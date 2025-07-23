window.BENCHMARK_DATA = {
  "lastUpdate": 1753272444285,
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 141729613,
            "range": "± 1458794",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
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
            "name": "query::balance/default/middle_10y16a500t",
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 141047079,
            "range": "± 2884112",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
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
            "name": "query::balance/default/middle_10y16a500t",
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 142335552,
            "range": "± 678354",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
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
            "name": "query::balance/default/middle_10y16a500t",
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 139808326,
            "range": "± 4190115",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
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
            "name": "query::balance/default/middle_10y16a500t",
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 141068684,
            "range": "± 5218111",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
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
            "name": "query::balance/default/middle_10y16a500t",
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 141774567,
            "range": "± 4080176",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
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
            "name": "query::balance/default/middle_10y16a500t",
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 134485789,
            "range": "± 1161262",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
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
            "name": "query::balance/default/middle_10y16a500t",
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 134294454,
            "range": "± 1226347",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
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
            "name": "query::balance/default/middle_10y16a500t",
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 129793058,
            "range": "± 651625",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
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
            "name": "query::balance/default/middle_10y16a500t",
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 145136185,
            "range": "± 1096647",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
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
            "name": "query::balance/default/middle_10y16a500t",
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 144082568,
            "range": "± 1255281",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
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
            "name": "query::balance/default/middle_10y16a500t",
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 144001622,
            "range": "± 1711031",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
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
            "name": "query::balance/default/middle_10y16a500t",
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 146033021,
            "range": "± 1402766",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
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
            "name": "query::balance/default/middle_10y16a500t",
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 137789843,
            "range": "± 1698106",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
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
            "name": "query::balance/default/middle_10y16a500t",
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 140390537,
            "range": "± 1147722",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
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
            "name": "query::balance/default/middle_10y16a500t",
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 137862329,
            "range": "± 1538841",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
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
            "name": "query::balance/default/middle_10y16a500t",
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
          "id": "44451d7e36814fe3451c2a30919b72791d8d1d36",
          "message": "Don't fail on benchmark regression for continuous run.\n\nOtherwise we won't track the main branch results.",
          "timestamp": "2025-02-25T10:37:19+01:00",
          "tree_id": "f0e4ecd32a9360dec71be58ed9a54b50086c041a",
          "url": "https://github.com/xkikeg/okane/commit/44451d7e36814fe3451c2a30919b72791d8d1d36"
        },
        "date": 1740476452904,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 27,
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 146923396,
            "range": "± 2864420",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 220434568,
            "range": "± 3599391",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3131764,
            "range": "± 110661",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1619654,
            "range": "± 23436",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9768,
            "range": "± 86",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 46370620,
            "range": "± 932701",
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
          "id": "f9df1410f438b812d539752f1306c35c8de3a3ec",
          "message": "Don't round the amount on each-posting basis.\n\nIf we have conversion at each posting, often we'll have remainder.\nHowever, we should not round those at each posting. Why?\nAssume the following situation.\n\n```\n   Expense        1 CHF @ 100.5 JPY\n   Expense        1 CHF @ 100.5 JPY\n   Asset       -201 JPY\n```\n\nThis is totally legit, however, if you apply banker's round,\ntwo expenses are evaluated as 100 JPY and there will be 1 JPY\nunbalanced.",
          "timestamp": "2025-03-03T19:02:33+01:00",
          "tree_id": "895c0db00a9001c2cb79ec9f3345e3b746079a90",
          "url": "https://github.com/xkikeg/okane/commit/f9df1410f438b812d539752f1306c35c8de3a3ec"
        },
        "date": 1741025113406,
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 138474455,
            "range": "± 2806341",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 216152934,
            "range": "± 1250903",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3182360,
            "range": "± 19925",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1595366,
            "range": "± 42818",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9938,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 46328040,
            "range": "± 303267",
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
          "id": "bc37f3b4c86656b01584a0cfbf9ce0adc54bddc0",
          "message": "Avoid including files starting with dot on glob.\n\nUnless the include glob pattern as explicit dot,\nwe won't read it.\nThis helps when Emacs create temprary files.",
          "timestamp": "2025-03-06T09:05:48+01:00",
          "tree_id": "5aaeb0e72bea2f5a9e26c2a0420c6d949fc85294",
          "url": "https://github.com/xkikeg/okane/commit/bc37f3b4c86656b01584a0cfbf9ce0adc54bddc0"
        },
        "date": 1741248511868,
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 139928315,
            "range": "± 1518750",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 214023467,
            "range": "± 938030",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3206785,
            "range": "± 69088",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1600617,
            "range": "± 25473",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9679,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 46503022,
            "range": "± 273976",
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
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "604f919787c283d1f3787aa64a17ee7ff6e97174",
          "message": "Add report_bench to test with on-memory files. (#225)\n\n* Add repot_bench to test with on-memory files.\n  This will enable running benches with different sized inputs.\n* Add varied parameter to test small to large instances.\n* Filter out InputParams by default to skip large set.",
          "timestamp": "2025-03-26T08:56:15+01:00",
          "tree_id": "66f57c0b84981204a6c3c940c7005f8b4a26a141",
          "url": "https://github.com/xkikeg/okane/commit/604f919787c283d1f3787aa64a17ee7ff6e97174"
        },
        "date": 1742976267909,
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 141709619,
            "range": "± 1365105",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 16790650,
            "range": "± 157181",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 133551815,
            "range": "± 1473752",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 134986689,
            "range": "± 1506482",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 25955221,
            "range": "± 482067",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 206666294,
            "range": "± 1913638",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 215681399,
            "range": "± 7709342",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3101709,
            "range": "± 13696",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 27995,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1608239,
            "range": "± 11853",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1692493,
            "range": "± 13854",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 7098,
            "range": "± 160",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9481,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 20547,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5814721,
            "range": "± 17496",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 46762955,
            "range": "± 953565",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 51175340,
            "range": "± 382397",
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
          "id": "419a355595ec79b337008ae39ef102cfe438b9e4",
          "message": "Release okane-golden 0.1.1.\n\nThis removes some internal dev-dependencies,\nalso added README.",
          "timestamp": "2025-04-06T11:22:00+02:00",
          "tree_id": "6e31ebc4aac99f753abe6f97cff5be7f054638f7",
          "url": "https://github.com/xkikeg/okane/commit/419a355595ec79b337008ae39ef102cfe438b9e4"
        },
        "date": 1743931815642,
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 143207683,
            "range": "± 2497822",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 17151650,
            "range": "± 106772",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 136499150,
            "range": "± 1354566",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 139359890,
            "range": "± 1460103",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 26145381,
            "range": "± 158445",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 208205531,
            "range": "± 2917558",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 219106017,
            "range": "± 3127523",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3106159,
            "range": "± 112919",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 28011,
            "range": "± 746",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1605433,
            "range": "± 10148",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1694992,
            "range": "± 19253",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 7003,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9506,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 20274,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5808506,
            "range": "± 35871",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 46495205,
            "range": "± 645356",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 50908250,
            "range": "± 2303198",
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
          "id": "c25a7f37f7e8f0e0faf15a6510e618f1c109d243",
          "message": "Add okane-golden coverage.",
          "timestamp": "2025-04-07T07:57:41+02:00",
          "tree_id": "3dc3dcbcedad32b499545b69e8d5f70eb68c1bbc",
          "url": "https://github.com/xkikeg/okane/commit/c25a7f37f7e8f0e0faf15a6510e618f1c109d243"
        },
        "date": 1744005919044,
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 145506910,
            "range": "± 2348182",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 17222164,
            "range": "± 70023",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 137126711,
            "range": "± 2193742",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 139483377,
            "range": "± 1350983",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 26767903,
            "range": "± 167589",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 212291497,
            "range": "± 1144297",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 222396339,
            "range": "± 893759",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3117519,
            "range": "± 51929",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 28010,
            "range": "± 174",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1621866,
            "range": "± 20448",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1700398,
            "range": "± 12813",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 7058,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9489,
            "range": "± 360",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 20076,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5810299,
            "range": "± 130210",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 46630961,
            "range": "± 125477",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 50837425,
            "range": "± 373506",
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
          "id": "fce174fc8213f1fa1db01b3e3a85ba179a232819",
          "message": "Update Rust version to 1.81.",
          "timestamp": "2025-04-08T09:25:39+02:00",
          "tree_id": "42b37f13a9bc3f8e6fec6c473a84aa5c2a634607",
          "url": "https://github.com/xkikeg/okane/commit/fce174fc8213f1fa1db01b3e3a85ba179a232819"
        },
        "date": 1744097618835,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 27,
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 146104903,
            "range": "± 549139",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 17326657,
            "range": "± 129366",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 137957987,
            "range": "± 934212",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 140836949,
            "range": "± 900095",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 26456707,
            "range": "± 124274",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 210751690,
            "range": "± 750265",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 221563366,
            "range": "± 992216",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3187953,
            "range": "± 15097",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 28019,
            "range": "± 191",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1589383,
            "range": "± 19011",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1682606,
            "range": "± 6190",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 7081,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9384,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 20364,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5743230,
            "range": "± 10075",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 46202540,
            "range": "± 148639",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 50847936,
            "range": "± 460006",
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
          "id": "807afd26490b089f4bf703843f9b7e4473e46a60",
          "message": "Updated CHANGELOG.",
          "timestamp": "2025-04-08T15:23:42+02:00",
          "tree_id": "fb2585cd7d7980e96b9e536209449d68809fda2f",
          "url": "https://github.com/xkikeg/okane/commit/807afd26490b089f4bf703843f9b7e4473e46a60"
        },
        "date": 1744119076992,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 27,
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 141901218,
            "range": "± 632567",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 16885510,
            "range": "± 124265",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 134346327,
            "range": "± 1655123",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 136080038,
            "range": "± 1372253",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 25897030,
            "range": "± 474919",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 205629396,
            "range": "± 1486702",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 214587952,
            "range": "± 987566",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3177232,
            "range": "± 17366",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 28012,
            "range": "± 252",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1600674,
            "range": "± 15559",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1692151,
            "range": "± 42601",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 6948,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9357,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 19993,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5786152,
            "range": "± 18305",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 46393039,
            "range": "± 185540",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 51508835,
            "range": "± 305549",
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
          "id": "8c5b222ea599bfcec2fd163260854c93d2dc7222",
          "message": "Fixed release date of the CHANGELOG on 0.14.0.",
          "timestamp": "2025-04-08T18:07:07+02:00",
          "tree_id": "2aeab034abecbfeeb62d49afcbc6bbdc52e80727",
          "url": "https://github.com/xkikeg/okane/commit/8c5b222ea599bfcec2fd163260854c93d2dc7222"
        },
        "date": 1744128883774,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 27,
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 142830510,
            "range": "± 611464",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 17134736,
            "range": "± 166981",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 136427182,
            "range": "± 1127324",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 137612372,
            "range": "± 4234158",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 26027300,
            "range": "± 173708",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 207199797,
            "range": "± 1541544",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 215799187,
            "range": "± 704863",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3154980,
            "range": "± 18039",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 28027,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1623371,
            "range": "± 16434",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1707163,
            "range": "± 11188",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 6944,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9364,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 19899,
            "range": "± 98",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5772630,
            "range": "± 19061",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 49765183,
            "range": "± 1322399",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 68460077,
            "range": "± 3452214",
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
          "id": "1a199d737f02dd074fb33c117a95e753dfcf8ec0",
          "message": "Add balance assertion to bench test data.",
          "timestamp": "2025-04-11T18:27:56+02:00",
          "tree_id": "94ad2f8109ddcadeec2c1e779abc352b74d5fb68",
          "url": "https://github.com/xkikeg/okane/commit/1a199d737f02dd074fb33c117a95e753dfcf8ec0"
        },
        "date": 1744389330400,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 25,
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 156674701,
            "range": "± 625943",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 19066155,
            "range": "± 170560",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 152177517,
            "range": "± 2013900",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 154785741,
            "range": "± 1441113",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 29379757,
            "range": "± 126423",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 233386336,
            "range": "± 1201710",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 243978164,
            "range": "± 1070629",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3089493,
            "range": "± 19323",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 28013,
            "range": "± 171",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1596271,
            "range": "± 32895",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1698036,
            "range": "± 9476",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 7051,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9581,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 20212,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5766761,
            "range": "± 16402",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 46221184,
            "range": "± 94285",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 50115836,
            "range": "± 373926",
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
          "id": "b9a6c6621f4fe753d7eda3ed95bad5b377a9fc72",
          "message": "Enhance error message for the balance assertion.",
          "timestamp": "2025-04-15T16:00:46+02:00",
          "tree_id": "615d8763daebb7c8a52712db0b77c965b8a04c04",
          "url": "https://github.com/xkikeg/okane/commit/b9a6c6621f4fe753d7eda3ed95bad5b377a9fc72"
        },
        "date": 1744726087324,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 25,
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 151182096,
            "range": "± 1791223",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 17990202,
            "range": "± 169421",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 143691495,
            "range": "± 1152985",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 145023221,
            "range": "± 1604154",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 28787448,
            "range": "± 348128",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 228863503,
            "range": "± 2326654",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 238430456,
            "range": "± 2950296",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3144869,
            "range": "± 15254",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 28009,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1619170,
            "range": "± 9687",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1708346,
            "range": "± 12970",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 6981,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9493,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 20012,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5772291,
            "range": "± 46059",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 46318063,
            "range": "± 271137",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 51063198,
            "range": "± 432167",
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
          "id": "7e2c3907beae43774f2a2324218fa23d6dbcf6d0",
          "message": "Change FakeFileSystem path to be always UNIX-like.\n\nThis allows testing files with expected output.\nNote the current implementation doesn't work with /root yet,\nso we should use relative path.",
          "timestamp": "2025-04-20T22:15:08+02:00",
          "tree_id": "29a50d7717adb3b1ef5eda6803ffbc68db249f64",
          "url": "https://github.com/xkikeg/okane/commit/7e2c3907beae43774f2a2324218fa23d6dbcf6d0"
        },
        "date": 1745180574471,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 27,
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
            "name": "load/on-file/middle_10y16a500t",
            "value": 157109259,
            "range": "± 2249263",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 18586018,
            "range": "± 184596",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 148702738,
            "range": "± 1245629",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 150819122,
            "range": "± 1698825",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 29083602,
            "range": "± 191672",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 230988923,
            "range": "± 3261106",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 241841927,
            "range": "± 3742564",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3078471,
            "range": "± 21183",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 27996,
            "range": "± 127",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1598385,
            "range": "± 20366",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1698729,
            "range": "± 37280",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 7154,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9398,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 20021,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5813943,
            "range": "± 11627",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 46387625,
            "range": "± 141701",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 51135929,
            "range": "± 363479",
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
          "id": "bc0e596cc260971617337c6140572462cdc88261",
          "message": "Added benchmark for PrettyDecimal::to_string.",
          "timestamp": "2025-05-16T18:36:35+02:00",
          "tree_id": "2724e52b5581ee89b6ff4c77ed2cef3dffc8d184",
          "url": "https://github.com/xkikeg/okane/commit/bc0e596cc260971617337c6140572462cdc88261"
        },
        "date": 1747413910141,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse comma",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string plain",
            "value": 85,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 142,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 151326154,
            "range": "± 973308",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 17837387,
            "range": "± 77446",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 142565625,
            "range": "± 1201212",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 144010103,
            "range": "± 1504685",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 28220112,
            "range": "± 430022",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 225271615,
            "range": "± 3137591",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 235005612,
            "range": "± 3323749",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2917141,
            "range": "± 14308",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 15,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 28009,
            "range": "± 1438",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1596977,
            "range": "± 14057",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1685933,
            "range": "± 7700",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 7107,
            "range": "± 109",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9460,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 20065,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5757344,
            "range": "± 39565",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 46172208,
            "range": "± 166687",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 50333806,
            "range": "± 294633",
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
          "id": "fe26de6e5cdf090c0601c8c3dc885634943b3651",
          "message": "Fix PrettyDecimal::fmt to work with comma3dot + fraction.",
          "timestamp": "2025-05-16T18:52:09+02:00",
          "tree_id": "b3e955b652c3fc9be07ee8a70928df39e0dd372a",
          "url": "https://github.com/xkikeg/okane/commit/fe26de6e5cdf090c0601c8c3dc885634943b3651"
        },
        "date": 1747414798601,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse comma",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string plain",
            "value": 84,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 196,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 154512445,
            "range": "± 516903",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 17874911,
            "range": "± 79491",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 143063323,
            "range": "± 1003315",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 144145203,
            "range": "± 2191181",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 28726401,
            "range": "± 146996",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 229925853,
            "range": "± 5002406",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 238928328,
            "range": "± 2775768",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3065639,
            "range": "± 13017",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 28015,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1594738,
            "range": "± 15975",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1689020,
            "range": "± 15656",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 7158,
            "range": "± 163",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9425,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 20328,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5792487,
            "range": "± 17859",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 46461131,
            "range": "± 159973",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 50910078,
            "range": "± 242106",
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
          "id": "3d709dc94a97881228f46c06c9dee2dab2433afd",
          "message": "Don't use write!(), instead use write_str, write_char.\n\nLet's see if this has any visible performance impact.",
          "timestamp": "2025-05-16T19:16:34+02:00",
          "tree_id": "02182c4b96f73134b13f4502a602208c2c8a853a",
          "url": "https://github.com/xkikeg/okane/commit/3d709dc94a97881228f46c06c9dee2dab2433afd"
        },
        "date": 1747416263719,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 29,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse comma",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string plain",
            "value": 84,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 164,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 152995150,
            "range": "± 858349",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 17924726,
            "range": "± 121612",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 141992016,
            "range": "± 1194406",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 143597861,
            "range": "± 1472626",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 28481948,
            "range": "± 99126",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 227216006,
            "range": "± 2134922",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 236161248,
            "range": "± 2525074",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3151572,
            "range": "± 17667",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 27995,
            "range": "± 206",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1599848,
            "range": "± 16184",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1696194,
            "range": "± 15099",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 7157,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9359,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 20249,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5690426,
            "range": "± 21030",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 45922089,
            "range": "± 126595",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 50087875,
            "range": "± 428753",
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
          "id": "658fce900deb6d1aa515edf7737551d651b3db7c",
          "message": "Remove number_of_integral_digits.\n\nThis is not needed as we gets the mantissa as str.\nThe problematic case in #249 is solved with <1000 check.",
          "timestamp": "2025-05-18T11:50:24+02:00",
          "tree_id": "b797cf8af70e14635032cecf6bdfb4c9fd165f39",
          "url": "https://github.com/xkikeg/okane/commit/658fce900deb6d1aa515edf7737551d651b3db7c"
        },
        "date": 1747562294522,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse comma",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string plain",
            "value": 84,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 96,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 151434670,
            "range": "± 431659",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 17801792,
            "range": "± 105695",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 141869838,
            "range": "± 1401798",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 143590718,
            "range": "± 1014379",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 28636394,
            "range": "± 172326",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 228419654,
            "range": "± 4208473",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 238493127,
            "range": "± 2836409",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3156708,
            "range": "± 72178",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 28008,
            "range": "± 165",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1606979,
            "range": "± 11463",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1704529,
            "range": "± 14736",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 7115,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9336,
            "range": "± 178",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 20314,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5755196,
            "range": "± 20771",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 46479786,
            "range": "± 1071989",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 52593014,
            "range": "± 1738790",
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
          "id": "24bf5d6903df2ede8db5d90f17bb84f5d9274b07",
          "message": "Add cross-platform test CI to Github actions.",
          "timestamp": "2025-05-21T10:00:35+02:00",
          "tree_id": "00c57364f7a721fb89f82b8eb416daada45243c2",
          "url": "https://github.com/xkikeg/okane/commit/24bf5d6903df2ede8db5d90f17bb84f5d9274b07"
        },
        "date": 1747814904282,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 28,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse comma",
            "value": 29,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string plain",
            "value": 84,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 94,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 151814588,
            "range": "± 749938",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 17883940,
            "range": "± 135128",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 142457509,
            "range": "± 965071",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 143933730,
            "range": "± 4953211",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 28928193,
            "range": "± 1485470",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 230567729,
            "range": "± 2935622",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 239344717,
            "range": "± 3159975",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3137101,
            "range": "± 20028",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 28003,
            "range": "± 133",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1605158,
            "range": "± 83208",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1701347,
            "range": "± 16984",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 7196,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9535,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 20408,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5768837,
            "range": "± 325666",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 46114994,
            "range": "± 148448",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 51287794,
            "range": "± 533904",
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
          "id": "bac76ef6a8a04860abcb82fcb2f4ae585eae1ce3",
          "message": "Support commodity renaming in the import command.\n\nThis helps especially when the given commodity is quite hard to read,\nsuch as numerical ID of the stock.",
          "timestamp": "2025-06-18T18:27:05+02:00",
          "tree_id": "585da982859a067191e872959ff00595072fccb7",
          "url": "https://github.com/xkikeg/okane/commit/bac76ef6a8a04860abcb82fcb2f4ae585eae1ce3"
        },
        "date": 1750264532489,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 29,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse comma",
            "value": 30,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string plain",
            "value": 84,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 93,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 151027690,
            "range": "± 2620667",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 17734321,
            "range": "± 300402",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 141282161,
            "range": "± 1427291",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 143014980,
            "range": "± 805669",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 28763503,
            "range": "± 179160",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 230104898,
            "range": "± 2386234",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 240602127,
            "range": "± 3091984",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3118803,
            "range": "± 18041",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 28009,
            "range": "± 142",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1604448,
            "range": "± 12228",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1694413,
            "range": "± 14630",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 7121,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9482,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 20326,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5749983,
            "range": "± 17370",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 46318497,
            "range": "± 373118",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 51509792,
            "range": "± 491182",
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
          "id": "28473ff3602312ead7200e32eb7e2ca1297dab17",
          "message": "Added 0.15.0 CHANGELOG.",
          "timestamp": "2025-06-19T17:56:07+02:00",
          "tree_id": "4a4d584bf09a54e84ecdb686628b75c32d0c3a95",
          "url": "https://github.com/xkikeg/okane/commit/28473ff3602312ead7200e32eb7e2ca1297dab17"
        },
        "date": 1750349076831,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse comma",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string plain",
            "value": 86,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 84,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 156296244,
            "range": "± 1083928",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 18033526,
            "range": "± 300100",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 142954947,
            "range": "± 1094654",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 144259015,
            "range": "± 1078939",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 29293498,
            "range": "± 341168",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 233706167,
            "range": "± 4437505",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 243132521,
            "range": "± 2887599",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3073293,
            "range": "± 26458",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 28007,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1676569,
            "range": "± 20258",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1769285,
            "range": "± 15076",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 7219,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9519,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 20281,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5759732,
            "range": "± 43481",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 46368022,
            "range": "± 546889",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 51742446,
            "range": "± 2532157",
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
          "id": "4bd79dce7280bb36df192a35fdcafec5830b5238",
          "message": "Bump version to v0.16.0-dev.",
          "timestamp": "2025-06-19T19:35:05+02:00",
          "tree_id": "b222a851f977653c14743de56b58fe611e8c1a62",
          "url": "https://github.com/xkikeg/okane/commit/4bd79dce7280bb36df192a35fdcafec5830b5238"
        },
        "date": 1750355004950,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 25,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "parse comma",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string plain",
            "value": 86,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 84,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 162245686,
            "range": "± 556849",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 18969329,
            "range": "± 109785",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 151297545,
            "range": "± 1023583",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 153920024,
            "range": "± 1062654",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 29753102,
            "range": "± 194704",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 238596106,
            "range": "± 3189590",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 249245748,
            "range": "± 2888689",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2973485,
            "range": "± 21202",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 27991,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1598695,
            "range": "± 17674",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1689791,
            "range": "± 9983",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 7126,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9475,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 20511,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5796564,
            "range": "± 16589",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 46490334,
            "range": "± 244977",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 51540840,
            "range": "± 248410",
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
          "id": "0c022563dab74406ca7b4715a9e966632e02f92f",
          "message": "Upgrade dependencies.",
          "timestamp": "2025-07-11T10:09:04+02:00",
          "tree_id": "66d2d2ce2d51933f92146f26b35cd45e8e00284d",
          "url": "https://github.com/xkikeg/okane/commit/0c022563dab74406ca7b4715a9e966632e02f92f"
        },
        "date": 1752221858080,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 25,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "parse comma",
            "value": 26,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string plain",
            "value": 86,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 83,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 154219253,
            "range": "± 3667466",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 18187016,
            "range": "± 603596",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 144173989,
            "range": "± 2272876",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 145899487,
            "range": "± 4605187",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 28960763,
            "range": "± 1692743",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 230296424,
            "range": "± 3970205",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 240193691,
            "range": "± 6149979",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3034482,
            "range": "± 38790",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 28001,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1598531,
            "range": "± 12275",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1694881,
            "range": "± 18628",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 7136,
            "range": "± 159",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9523,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 20454,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5729858,
            "range": "± 8226",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 45953168,
            "range": "± 279152",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 50918656,
            "range": "± 455994",
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
          "id": "7114980b00aac6b7267911440dd0eea7e5da9d0e",
          "message": "Upgrade quick_xml to 0.38.",
          "timestamp": "2025-07-11T14:05:11+02:00",
          "tree_id": "35a5d57c1ffb8b61c4f9e4d4c867923e69aeb6f8",
          "url": "https://github.com/xkikeg/okane/commit/7114980b00aac6b7267911440dd0eea7e5da9d0e"
        },
        "date": 1752235972584,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse comma",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string plain",
            "value": 86,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 84,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 153679083,
            "range": "± 1171338",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 18202615,
            "range": "± 132779",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 145408629,
            "range": "± 2174168",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 146805966,
            "range": "± 1961648",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 28863748,
            "range": "± 337463",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 230651398,
            "range": "± 2822389",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 240256880,
            "range": "± 2770389",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3032062,
            "range": "± 15555",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 28006,
            "range": "± 211",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1598554,
            "range": "± 22583",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1703920,
            "range": "± 15612",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 7099,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9511,
            "range": "± 83",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 20413,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5743700,
            "range": "± 15942",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 46206398,
            "range": "± 283671",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 51041479,
            "range": "± 378297",
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
          "id": "a261cb1e8cb393fbc1bacd1894fa2471214b5bf9",
          "message": "Enables LTO for release build.",
          "timestamp": "2025-07-21T23:28:09+02:00",
          "tree_id": "a5304b65faf4e0202b43703102d7308e19d1a1e2",
          "url": "https://github.com/xkikeg/okane/commit/a261cb1e8cb393fbc1bacd1894fa2471214b5bf9"
        },
        "date": 1753133816227,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse comma",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string plain",
            "value": 85,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 72,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 137832294,
            "range": "± 747076",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 16095754,
            "range": "± 78580",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 129596385,
            "range": "± 1009487",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 131306417,
            "range": "± 1016358",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 26345976,
            "range": "± 94043",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 210238721,
            "range": "± 3424798",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 220103718,
            "range": "± 2753028",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3036976,
            "range": "± 22280",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 13,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 26948,
            "range": "± 210",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1474487,
            "range": "± 6692",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1577726,
            "range": "± 14314",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 7088,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9409,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 20037,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5660093,
            "range": "± 12128",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 45548198,
            "range": "± 547708",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 51211264,
            "range": "± 1528012",
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
          "id": "d3c091d89fface0a49d682084e481887bbfee504",
          "message": "Add more tests to cover Merge logics.",
          "timestamp": "2025-07-23T13:58:55+02:00",
          "tree_id": "2d482f43ade238cb1846477d83bc7a813381484b",
          "url": "https://github.com/xkikeg/okane/commit/d3c091d89fface0a49d682084e481887bbfee504"
        },
        "date": 1753272443800,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse comma",
            "value": 26,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "to_string plain",
            "value": 87,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 86,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 139734164,
            "range": "± 598912",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 16289326,
            "range": "± 197859",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 129614664,
            "range": "± 1658945",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 131056578,
            "range": "± 1671629",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 26892142,
            "range": "± 73220",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 216584159,
            "range": "± 2552085",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 226409970,
            "range": "± 3254566",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2967906,
            "range": "± 15800",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 27477,
            "range": "± 239",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1500119,
            "range": "± 18056",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1598098,
            "range": "± 20941",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 7102,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9392,
            "range": "± 188",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 20103,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5722164,
            "range": "± 15030",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 46059208,
            "range": "± 319024",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 51325677,
            "range": "± 1538190",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}