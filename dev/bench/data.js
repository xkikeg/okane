window.BENCHMARK_DATA = {
  "lastUpdate": 1739395942019,
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
      },
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
          "id": "5aefeaf6c1be5be5b4b693f65993d63b56919df2",
          "message": "Prohibit putting the exchange with the same commodity on import.\n\nThis is error saying 1 EUR = 5 EUR (example).",
          "timestamp": "2025-01-27T15:57:41+01:00",
          "tree_id": "decb98ee2fc0434ad9b12b6be5474ef723ba00d6",
          "url": "https://github.com/xkikeg/okane/commit/5aefeaf6c1be5be5b4b693f65993d63b56919df2"
        },
        "date": 1737990009366,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 26,
            "range": "± 1",
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
            "value": 141047079,
            "range": "± 2884112",
            "unit": "ns/iter"
          },
          {
            "name": "process",
            "value": 206940667,
            "range": "± 2191703",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3104741,
            "range": "± 14669",
            "unit": "ns/iter"
          },
          {
            "name": "query-balance-default",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "30be292a0c904fa4602c9561e87b39564753007c",
          "message": "Uses parse::adaptor implementation for all parse users.",
          "timestamp": "2025-01-29T08:51:31+01:00",
          "tree_id": "c776871d0843384765210f44717f1be115396481",
          "url": "https://github.com/xkikeg/okane/commit/30be292a0c904fa4602c9561e87b39564753007c"
        },
        "date": 1738137242894,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 26,
            "range": "± 1",
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
            "value": 142335552,
            "range": "± 678354",
            "unit": "ns/iter"
          },
          {
            "name": "process",
            "value": 210736576,
            "range": "± 2930614",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3117359,
            "range": "± 13439",
            "unit": "ns/iter"
          },
          {
            "name": "query-balance-default",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "9a81535e0a1162dc6d4a55de3f5c30d5ea5a8ecc",
          "message": "Change Evaluable::eval to eval_mut, as it accepts mut context.\n\nLater we can prepare eval() method as well,\nwhich will treat context as immutable, better once report::process\ncompletes.",
          "timestamp": "2025-01-29T18:48:38+01:00",
          "tree_id": "49b8c8225a091047779a8ea440ff9ec3f8218773",
          "url": "https://github.com/xkikeg/okane/commit/9a81535e0a1162dc6d4a55de3f5c30d5ea5a8ecc"
        },
        "date": 1738173082385,
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
            "value": 139808326,
            "range": "± 4190115",
            "unit": "ns/iter"
          },
          {
            "name": "process",
            "value": 210447815,
            "range": "± 1785316",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3134187,
            "range": "± 15027",
            "unit": "ns/iter"
          },
          {
            "name": "query-balance-default",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "37e63741d3ba8a546ff496d69b956c021353f983",
          "message": "Provide eval command to resolve historical rate.\n\nWith --exchange and --price-db flags, users can convert the\ngiven value expression into the given commodity.",
          "timestamp": "2025-01-29T18:57:19+01:00",
          "tree_id": "ae844a7a3309b823bd84e34ea78d1c875ea8d46a",
          "url": "https://github.com/xkikeg/okane/commit/37e63741d3ba8a546ff496d69b956c021353f983"
        },
        "date": 1738173600974,
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
            "value": 141068684,
            "range": "± 5218111",
            "unit": "ns/iter"
          },
          {
            "name": "process",
            "value": 213774508,
            "range": "± 1250643",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2915148,
            "range": "± 23148",
            "unit": "ns/iter"
          },
          {
            "name": "query-balance-default",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "73477b3a312f77f98872eb1e8894aee3d0bea263",
          "message": "Added coverage CI workflow.\n\nThis also adds caching to cargo.",
          "timestamp": "2025-01-30T10:07:11+01:00",
          "tree_id": "14a07c10578b5a7d2b7ffdcddad1ea511a8e9fc6",
          "url": "https://github.com/xkikeg/okane/commit/73477b3a312f77f98872eb1e8894aee3d0bea263"
        },
        "date": 1738228191904,
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
            "value": 141774567,
            "range": "± 4080176",
            "unit": "ns/iter"
          },
          {
            "name": "process",
            "value": 211166695,
            "range": "± 1384006",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2911264,
            "range": "± 23346",
            "unit": "ns/iter"
          },
          {
            "name": "query-balance-default",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "bbbf06e60c9e97882ac88c5ee4d3a939b949f0d6",
          "message": "Fix post accounting parse to support multi-byte characters.\n\nPreviously it took the byte length of posting acount characters,\nand tries to take the same amount of \"char\".\nIt won't work for non-ASCII characters, and inefficient as\nit was 3-pass algorithm.\nchanged to use repeat_till with peek.",
          "timestamp": "2025-01-31T10:56:05+01:00",
          "tree_id": "4cd758d5578e104294046247b16c29a2b771c861",
          "url": "https://github.com/xkikeg/okane/commit/bbbf06e60c9e97882ac88c5ee4d3a939b949f0d6"
        },
        "date": 1738317525323,
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
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load-with-counter",
            "value": 134485789,
            "range": "± 1161262",
            "unit": "ns/iter"
          },
          {
            "name": "process",
            "value": 205044499,
            "range": "± 3144818",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2932200,
            "range": "± 107590",
            "unit": "ns/iter"
          },
          {
            "name": "query-balance-default",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "d5af8db2be1330212fbca5d4f3199184e3165f9d",
          "message": "Add tests for report::query methods.",
          "timestamp": "2025-01-31T12:25:27+01:00",
          "tree_id": "7a677d410852ef1554fb0345648c815cfe220aff",
          "url": "https://github.com/xkikeg/okane/commit/d5af8db2be1330212fbca5d4f3199184e3165f9d"
        },
        "date": 1738322849758,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 26,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse comma",
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load-with-counter",
            "value": 134294454,
            "range": "± 1226347",
            "unit": "ns/iter"
          },
          {
            "name": "process",
            "value": 205387522,
            "range": "± 1362404",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2901842,
            "range": "± 18306",
            "unit": "ns/iter"
          },
          {
            "name": "query-balance-default",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "2b47f2d05db08e951f964314cc63ede75d66990d",
          "message": "Add check about if the include path is the hitting glob.\n\nRaise an error if it hits no files.",
          "timestamp": "2025-01-31T20:50:38+01:00",
          "tree_id": "5de8bda411d2eff9bfd5d71deff4ffb6a36ba01c",
          "url": "https://github.com/xkikeg/okane/commit/2b47f2d05db08e951f964314cc63ede75d66990d"
        },
        "date": 1738353157011,
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
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load-with-counter",
            "value": 129793058,
            "range": "± 651625",
            "unit": "ns/iter"
          },
          {
            "name": "process",
            "value": 204562364,
            "range": "± 867186",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2944523,
            "range": "± 15724",
            "unit": "ns/iter"
          },
          {
            "name": "query-balance-default",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "5494389d64fc474d362ccf20666e93cab1adc9fa",
          "message": "Corrected clippy findings.",
          "timestamp": "2025-01-31T20:53:03+01:00",
          "tree_id": "af752216881ec8f7265d8ed254fd72dd7ef1532d",
          "url": "https://github.com/xkikeg/okane/commit/5494389d64fc474d362ccf20666e93cab1adc9fa"
        },
        "date": 1738353327524,
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
            "value": 145136185,
            "range": "± 1096647",
            "unit": "ns/iter"
          },
          {
            "name": "process",
            "value": 216972522,
            "range": "± 1530856",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3037949,
            "range": "± 70958",
            "unit": "ns/iter"
          },
          {
            "name": "query-balance-default",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "863bd4f001a65667d04198db0d3fc679fada6246",
          "message": "Added more unit tests to PostingAmount and SingleAmount.",
          "timestamp": "2025-02-06T19:26:19+01:00",
          "tree_id": "eb11a16c2ffd93f808bb92f4a7da8b9bdeeaf7b2",
          "url": "https://github.com/xkikeg/okane/commit/863bd4f001a65667d04198db0d3fc679fada6246"
        },
        "date": 1738866493655,
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
            "value": 144082568,
            "range": "± 1255281",
            "unit": "ns/iter"
          },
          {
            "name": "process",
            "value": 215684022,
            "range": "± 1287360",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2979576,
            "range": "± 27167",
            "unit": "ns/iter"
          },
          {
            "name": "query-balance-default",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "f7bae80216fc0cea15614cf49431ef1d90683878",
          "message": "In `import` CLI, put debits earlier than credits.\n\nUsually 借方 / debits comes earlier than 貸方 / credits.",
          "timestamp": "2025-02-10T15:02:45+01:00",
          "tree_id": "e5085029629684ae44de49d7a1f684cba0ee5b85",
          "url": "https://github.com/xkikeg/okane/commit/f7bae80216fc0cea15614cf49431ef1d90683878"
        },
        "date": 1739196288762,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 26,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse comma",
            "value": 27,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "load-with-counter",
            "value": 144001622,
            "range": "± 1711031",
            "unit": "ns/iter"
          },
          {
            "name": "process",
            "value": 215384251,
            "range": "± 3752542",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2988306,
            "range": "± 12820",
            "unit": "ns/iter"
          },
          {
            "name": "query-balance-default",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "62a41f079150910958b7c92c48f99bf259274146",
          "message": "Add test to cover balance failure.",
          "timestamp": "2025-02-10T19:05:44+01:00",
          "tree_id": "514bb288336fba79ad42e3ca273cb7d766919934",
          "url": "https://github.com/xkikeg/okane/commit/62a41f079150910958b7c92c48f99bf259274146"
        },
        "date": 1739210861562,
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
            "value": 146033021,
            "range": "± 1402766",
            "unit": "ns/iter"
          },
          {
            "name": "process",
            "value": 221390755,
            "range": "± 1089287",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3013497,
            "range": "± 22064",
            "unit": "ns/iter"
          },
          {
            "name": "query-balance-default",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "0d74ec043333d4910d566a028b29c4f432a54454",
          "message": "Updated dependencies.",
          "timestamp": "2025-02-11T17:46:18+01:00",
          "tree_id": "e59397f93eb0a2360c9db1bfae3637717479fff2",
          "url": "https://github.com/xkikeg/okane/commit/0d74ec043333d4910d566a028b29c4f432a54454"
        },
        "date": 1739292515990,
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
            "value": 137789843,
            "range": "± 1698106",
            "unit": "ns/iter"
          },
          {
            "name": "process",
            "value": 212805821,
            "range": "± 1188877",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3000167,
            "range": "± 31481",
            "unit": "ns/iter"
          },
          {
            "name": "query-balance-default",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "8706bcf3dba05e21509fadc21e98cacc2974be7c",
          "message": "Change test_import to use rstest fixture as well.",
          "timestamp": "2025-02-12T00:20:36+01:00",
          "tree_id": "c4182bcb5b592973f14d6862c08842723404556c",
          "url": "https://github.com/xkikeg/okane/commit/8706bcf3dba05e21509fadc21e98cacc2974be7c"
        },
        "date": 1739316163686,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 26,
            "range": "± 1",
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
            "value": 140390537,
            "range": "± 1147722",
            "unit": "ns/iter"
          },
          {
            "name": "process",
            "value": 215807347,
            "range": "± 590390",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2994311,
            "range": "± 22305",
            "unit": "ns/iter"
          },
          {
            "name": "query-balance-default",
            "value": 2,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "3e7d768d6906bb08d5c46d9bf572a76792f88d22",
          "message": "Add golden test for balance command.\n\nThis allows testing conversion feature for balance later.",
          "timestamp": "2025-02-12T22:30:20+01:00",
          "tree_id": "c964f1ec572eed5c3676892a11a996c1fea31784",
          "url": "https://github.com/xkikeg/okane/commit/3e7d768d6906bb08d5c46d9bf572a76792f88d22"
        },
        "date": 1739395941170,
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
            "value": 137862329,
            "range": "± 1538841",
            "unit": "ns/iter"
          },
          {
            "name": "process",
            "value": 216857062,
            "range": "± 758752",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2979417,
            "range": "± 24825",
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