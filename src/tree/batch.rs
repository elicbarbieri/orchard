//! Batched MerkleCRH^Orchard, used by [`super::MerkleHashOrchard::combine_pairs`]
//!
//! - Position-weighted Sinsemilla over precomputed tables, the spec's per-step checks kept exact
//! - Derivation and exception mapping: book `design/commitment-tree.md` ("Batched MerkleCRH")
//! - Variable time (Merkle nodes are public)

use alloc::vec::Vec;

use ff::{BatchInverter, Field, PrimeField};
use group::{Curve, CurveAffine as _, Group};
use lazy_static::lazy_static;
use pasta_curves::{
    arithmetic::{Coordinates, CurveAffine},
    pallas,
};
use sinsemilla::{K, SINSEMILLA_S};

use crate::constants::sinsemilla::{L_ORCHARD_MERKLE, Q_MERKLE_CRH};

/// Words of a MerkleCRH message: the layer, then each child's [`L_ORCHARD_MERKLE`] bits
pub(super) const WORDS: usize = (K + 2 * L_ORCHARD_MERKLE) / K;

const _: () = assert!(
    (K + 2 * L_ORCHARD_MERKLE).is_multiple_of(K),
    "no padding chunk"
);

/// S-table entries, one per `K`-bit word value
const TABLE: usize = 1 << K;

/// Batch size from which lockstep beats projective
///
/// - Measured µs/pair, lockstep vs projective: 32 → 15.3 vs 11.7, 64 → 11.4 vs 11.9
const LOCKSTEP_LANES: usize = 64;

/// Affine `(x, y)` of a point known not to be the identity
type Coords = (pallas::Base, pallas::Base);

lazy_static! {
    /// Tables for MerkleCRH's `Q` (`HashDomain::Q` is test-only upstream)
    pub(super) static ref WEIGHTED_MERKLE_CRH: Weighted = Weighted::new(
        pallas::Affine::from_xy(
            pallas::Base::from_repr(Q_MERKLE_CRH.0).expect("canonical Q_MERKLE_CRH.x"),
            pallas::Base::from_repr(Q_MERKLE_CRH.1).expect("canonical Q_MERKLE_CRH.y"),
        )
        .expect("Q_MERKLE_CRH is on the curve")
        .into(),
    );
}

/// Sinsemilla over `WORDS`-word messages as `B = [2^WORDS]Q + Σ [2^(WORDS−1−i)]S[m_i]`
///
/// - `rows[e * TABLE + j] = [2^e]S[j]` for `e < WORDS`
/// - `first[j]` = `B` after word 0 = `j`, `None` where the spec's step 0 is ⊥
pub(super) struct Weighted {
    rows: Vec<Coords>,
    first: Vec<Option<Coords>>,
}

impl Weighted {
    fn new(q: pallas::Point) -> Self {
        let s: Vec<pallas::Point> = SINSEMILLA_S
            .iter()
            .map(|(x, y)| {
                pallas::Affine::from_xy(*x, *y)
                    .expect("S is on the curve")
                    .into()
            })
            .collect();

        let mut rows = Vec::with_capacity(WORDS * TABLE);
        let mut row = s.clone();
        for _ in 0..WORDS {
            rows.extend(
                normalized(&row)
                    .into_iter()
                    .map(|p| p.expect("[2^e]S[j] ≠ 0")),
            );
            row = row.iter().map(Group::double).collect();
        }

        let weighted_q = (0..WORDS).fold(q, |q, _| q.double());
        let top = &rows[(WORDS - 1) * TABLE..];
        let first: Vec<pallas::Point> = s
            .iter()
            .zip(top)
            .map(|(s, &(x, y))| {
                let exceptional = q == *s || q == -s || (q.double() + s).is_identity().into();
                match exceptional {
                    true => pallas::Point::identity(),
                    false => weighted_q + pallas::Affine::from_xy(x, y).expect("table point"),
                }
            })
            .collect();

        Weighted {
            rows,
            first: normalized(&first),
        }
    }

    fn row(&self, exponent: usize, word: u16) -> &Coords {
        &self.rows[exponent * TABLE + usize::from(word)]
    }

    /// Each message's hash, `None` where a step is ⊥ or a doubling (caller recomputes by the spec)
    pub(super) fn hash(&self, messages: &[[u16; WORDS]]) -> Vec<Option<pallas::Base>> {
        match messages.len() >= LOCKSTEP_LANES {
            true => self.lockstep(messages),
            false => self.projective(messages),
        }
    }

    /// Each message on its own Jacobian accumulator, one batched inversion for every `x`
    fn projective(&self, messages: &[[u16; WORDS]]) -> Vec<Option<pallas::Base>> {
        let points: Vec<Option<Jacobian>> = messages.iter().map(|m| self.jacobian(m)).collect();
        let mut z: Vec<pallas::Base> = points.iter().flatten().map(|p| p.z).collect();
        let mut scratch = vec![pallas::Base::ONE; z.len()];
        BatchInverter::invert_with_external_scratch(&mut z, &mut scratch);

        let mut inverses = z.into_iter();
        points
            .into_iter()
            .map(|point| {
                point.map(|p| p.x * inverses.next().expect("one inverse per point").square())
            })
            .collect()
    }

    fn jacobian(&self, message: &[u16; WORDS]) -> Option<Jacobian> {
        let (x, y) = self.first[usize::from(message[0])]?;
        let mut acc = Jacobian {
            x,
            y,
            z: pallas::Base::ONE,
        };
        for (k, &word) in message.iter().enumerate().skip(1) {
            let zz = acc.z.square();
            let (x_d, _) = self.row(WORDS - k, word);
            if acc.x == *x_d * zz {
                return None;
            }
            acc = acc.add_affine(self.row(WORDS - 1 - k, word), zz)?;
        }
        Some(acc)
    }

    /// Every message on an affine accumulator, stepped together (one batched inversion per step)
    fn lockstep(&self, messages: &[[u16; WORDS]]) -> Vec<Option<pallas::Base>> {
        let mut acc: Vec<Option<Coords>> = messages
            .iter()
            .map(|m| self.first[usize::from(m[0])])
            .collect();
        let mut dx = vec![pallas::Base::ONE; messages.len()];
        let mut scratch = dx.clone();

        for k in 1..WORDS {
            for ((acc, dx), m) in acc.iter_mut().zip(&mut dx).zip(messages) {
                *dx = pallas::Base::ONE;
                let Some((x, _)) = *acc else { continue };
                let (x_w, _) = self.row(WORDS - 1 - k, m[k]);
                let (x_d, _) = self.row(WORDS - k, m[k]);
                let denominator = *x_w - x;
                match x == *x_d || bool::from(denominator.is_zero()) {
                    true => *acc = None,
                    false => *dx = denominator,
                }
            }
            BatchInverter::invert_with_external_scratch(&mut dx, &mut scratch);
            for ((acc, inverse), m) in acc.iter_mut().zip(&dx).zip(messages) {
                let Some((x, y)) = *acc else { continue };
                let (x_w, y_w) = self.row(WORDS - 1 - k, m[k]);
                let lambda = (*y_w - y) * inverse;
                let x3 = lambda.square() - x - x_w;
                *acc = Some((x3, lambda * (x - x3) - y));
            }
        }
        acc.into_iter().map(|p| p.map(|(x, _)| x)).collect()
    }
}

/// Affine coordinates, `None` for the identity (one inversion for the whole slice)
fn normalized(points: &[pallas::Point]) -> Vec<Option<Coords>> {
    let mut affine = vec![pallas::Affine::identity(); points.len()];
    pallas::Point::batch_normalize(points, &mut affine);
    affine
        .iter()
        .map(|p| {
            let coordinates: Option<Coordinates<pallas::Affine>> = p.coordinates().into();
            coordinates.map(|c| (*c.x(), *c.y()))
        })
        .collect()
}

/// `(X, Y, Z)` for the affine `(X/Z², Y/Z³)`
#[derive(Clone, Copy)]
struct Jacobian {
    x: pallas::Base,
    y: pallas::Base,
    z: pallas::Base,
}

impl Jacobian {
    /// `self + (x2, y2)` by madd-2007-bl (7M + 3S given `zz = self.z²`)
    ///
    /// `None` where the xs coincide (sum = 0 or a doubling)
    fn add_affine(&self, (x2, y2): &Coords, zz: pallas::Base) -> Option<Self> {
        let h = *x2 * zz - self.x;
        if bool::from(h.is_zero()) {
            return None;
        }
        let hh = h.square();
        let i = hh.double().double();
        let j = h * i;
        let r = (*y2 * self.z * zz - self.y).double();
        let v = self.x * i;
        let x = r.square() - j - v.double();
        let y = r * (v - x) - (self.y * j).double();
        let z = (self.z + h).square() - zz - hh;
        Some(Jacobian { x, y, z })
    }
}

/// The `K`-bit words of `layer ‖ left ‖ right`
pub(super) fn words(layer: u8, left: &pallas::Base, right: &pallas::Base) -> [u16; WORDS] {
    let mut bits = [0u64; (WORDS * K).div_ceil(64)];
    bits[0] = u64::from(layer);
    place(&mut bits, left, K);
    place(&mut bits, right, K + L_ORCHARD_MERKLE);
    core::array::from_fn(|i| {
        let (at, shift) = (i * K / 64, i * K % 64);
        let low = bits[at] >> shift;
        let high = match shift + K > 64 {
            true => bits[at + 1] << (64 - shift),
            false => 0,
        };
        ((low | high) & ((1 << K) - 1)) as u16
    })
}

/// ORs `child`'s little-endian bits into `bits` from bit `at` (canonical, so under
/// [`L_ORCHARD_MERKLE`])
fn place(bits: &mut [u64], child: &pallas::Base, at: usize) {
    for (i, limb) in child.to_repr().chunks_exact(8).enumerate() {
        let limb = u64::from_le_bytes(limb.try_into().expect("8-byte limb"));
        let (word, shift) = ((at + 64 * i) / 64, (at + 64 * i) % 64);
        bits[word] |= limb << shift;
        if shift > 0 {
            bits[word + 1] |= limb >> (64 - shift);
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use ff::{Field, PrimeFieldBits};
    use group::Group;
    use pasta_curves::{arithmetic::CurveAffine, pallas};
    use proptest::prelude::*;
    use sinsemilla::{HashDomain, K, SINSEMILLA_S};

    use super::{words, Weighted, LOCKSTEP_LANES, WEIGHTED_MERKLE_CRH, WORDS};
    use crate::constants::{
        sinsemilla::{i2lebsp_k, L_ORCHARD_MERKLE},
        MERKLE_DEPTH_ORCHARD,
    };
    use crate::tree::{testing::arb_merkle_hash, MERKLE_CRH};

    /// Sinsemilla's chunk order: each word's bits, least significant first
    fn bits(message: &[u16; WORDS]) -> impl Iterator<Item = bool> + '_ {
        message
            .iter()
            .flat_map(|word| (0..K).map(move |i| (word >> i) & 1 == 1))
    }

    fn arb_message() -> impl Strategy<Value = [u16; WORDS]> {
        prop::collection::vec(0..(1u16 << K), WORDS)
            .prop_map(|words| words.try_into().expect("WORDS words"))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(16))]

        #[test]
        fn words_are_the_merkle_crh_message(
            layer in 0..MERKLE_DEPTH_ORCHARD as u8,
            left in arb_merkle_hash(),
            right in arb_merkle_hash(),
        ) {
            let expected: Vec<bool> = i2lebsp_k(layer.into())
                .into_iter()
                .chain(left.0.to_le_bits().iter().by_vals().take(L_ORCHARD_MERKLE))
                .chain(right.0.to_le_bits().iter().by_vals().take(L_ORCHARD_MERKLE))
                .collect();
            let packed: Vec<bool> = bits(&words(layer, &left.0, &right.0)).collect();
            prop_assert_eq!(packed, expected);
        }

        /// Batch sizes either side of [`LOCKSTEP_LANES`]
        #[test]
        fn both_strategies_equal_sinsemilla_hash(
            messages in prop::collection::vec(arb_message(), 0..=LOCKSTEP_LANES + 4),
        ) {
            let expected: Vec<Option<pallas::Base>> =
                messages.iter().map(|m| MERKLE_CRH.hash(bits(m)).into()).collect();
            prop_assert_eq!(&WEIGHTED_MERKLE_CRH.projective(&messages), &expected);
            prop_assert_eq!(&WEIGHTED_MERKLE_CRH.lockstep(&messages), &expected);
        }
    }

    /// `[2^exponent]S[word]` from the S table alone (not from the rows under test)
    fn weighted_s(exponent: usize, word: u16) -> pallas::Point {
        let (x, y) = SINSEMILLA_S[usize::from(word)];
        let s = pallas::Point::from(pallas::Affine::from_xy(x, y).expect("S is on the curve"));
        (0..exponent).fold(s, |p, _| p.double())
    }

    /// The `Q` whose weighted accumulator after word `first` is `target`
    fn q_placing(first: u16, target: pallas::Point) -> pallas::Point {
        let weight = pallas::Scalar::from(2).pow_vartime([WORDS as u64]);
        (target - weighted_s(WORDS - 1, first)) * weight.invert().expect("2^WORDS ≠ 0")
    }

    /// Crafted `Q`s put each exception on words 0 and 1: both strategies hand every one back, and
    /// the spec's ⊥ is exactly the c1/c2 cases (book "Exceptions stay exact")
    #[test]
    fn crafted_exceptions_are_handed_back() {
        // any two distinct word values
        const FIRST: u16 = 3;
        const SECOND: u16 = 700;
        let d = weighted_s(WORDS - 1, SECOND);
        let w = weighted_s(WORDS - 2, SECOND);
        let cases = [
            ("step 0: Q = S[first]", weighted_s(0, FIRST), true),
            ("c1: B = D", q_placing(FIRST, d), true),
            ("c1: B = -D", q_placing(FIRST, -d), true),
            ("c2: B = -W", q_placing(FIRST, -w), true),
            ("doubling: B = W", q_placing(FIRST, w), false),
        ];
        let mut message = [0u16; WORDS];
        (message[0], message[1]) = (FIRST, SECOND);

        for (case, q, bottom) in cases {
            let weighted = Weighted::new(q);
            assert_eq!(weighted.projective(&[message]), [None], "{case}");
            assert_eq!(weighted.lockstep(&[message]), [None], "{case}");
            let spec: Option<pallas::Base> = HashDomain::from_Q(q).hash(bits(&message)).into();
            assert_eq!(spec.is_none(), bottom, "{case}");
        }
    }
}
