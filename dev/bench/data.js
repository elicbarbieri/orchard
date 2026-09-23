window.BENCHMARK_DATA = {
  "lastUpdate": 1790198214250,
  "repoUrl": "https://github.com/elicbarbieri/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "john@coldnoise.net",
            "name": "John",
            "username": "nullcopy"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "2929d3b7114b4eb3ae36f1d705531f124ac450ef",
          "message": "Merge pull request #560 from nullcopy/pczt-action-decryption\n\npczt: add trial decryption and output recovery to Action",
          "timestamp": "2026-09-21T10:23:14-05:00",
          "tree_id": "8f1f0a75b86804fed486cb501ade28fc6a247fcc",
          "url": "https://github.com/elicbarbieri/orchard/commit/2929d3b7114b4eb3ae36f1d705531f124ac450ef"
        },
        "date": 1790198213114,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2532507838,
            "range": "± 10936580",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2517867667,
            "range": "± 40439920",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 3603074831,
            "range": "± 13059208",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 4705370775,
            "range": "± 14083368",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 19174464,
            "range": "± 169661",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 19235587,
            "range": "± 557679",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 22217928,
            "range": "± 197281",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 25201626,
            "range": "± 163820",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1259925,
            "range": "± 12890",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 103707,
            "range": "± 316",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1256192,
            "range": "± 17304",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1089396023,
            "range": "± 1387426",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 12682543,
            "range": "± 33917",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 1119731,
            "range": "± 22488",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 12636446,
            "range": "± 43672",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 1084979,
            "range": "± 12209",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 63290627,
            "range": "± 514560",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 5512754,
            "range": "± 5940",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 63098403,
            "range": "± 114797",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 5343195,
            "range": "± 10092",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 126634276,
            "range": "± 329112",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 11008988,
            "range": "± 127549",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 126244596,
            "range": "± 498939",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 10667785,
            "range": "± 43186",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 379984,
            "range": "± 4150",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 410508,
            "range": "± 641",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}