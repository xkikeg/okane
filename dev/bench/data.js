window.BENCHMARK_DATA = {
  "lastUpdate": 1737967590000,
  "repoUrl": "https://github.com/xkikeg/okane",
  "entries": {
    "Criterion.rs Benchmark": [
      {
        "commit": {
          "author": {
            "email": "kikeg@kikeg.com",
            "name": "kikeg",
            "username": "xkikeg"
          },
          "committer": {
            "email": "kikeg@kikeg.com",
            "name": "kikeg",
            "username": "xkikeg"
          },
          "distinct": true,
          "id": "74888809f1057d3574c97a92a9a5fe260ae05d6d",
          "message": "Corrected push target of benchmark CI.",
          "timestamp": "2025-01-27T09:44:00+01:00",
          "tree_id": "8dca12ceaf31fdb759867c76bba84b376b872720",
          "url": "https://github.com/xkikeg/okane/commit/74888809f1057d3574c97a92a9a5fe260ae05d6d"
        },
        "date": 1737967589692,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse comma",
            "value": 27,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load-with-counter",
            "value": 141729613,
            "range": "± 1458794",
            "unit": "ns/iter"
          },
          {
            "name": "process",
            "value": 213849868,
            "range": "± 973641",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3105149,
            "range": "± 20047",
            "unit": "ns/iter"
          },
          {
            "name": "query-balance-default",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}