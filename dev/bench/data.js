window.BENCHMARK_DATA = {
  "lastUpdate": 1769688773875,
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
          "id": "699cb8f4724a6c62139c76646dc6f51bca87a0b7",
          "message": "Add codecov config to itemize modules.",
          "timestamp": "2025-07-23T15:58:09+02:00",
          "tree_id": "ec4aa436586d8c45a08812653a8aa97ca010ee27",
          "url": "https://github.com/xkikeg/okane/commit/699cb8f4724a6c62139c76646dc6f51bca87a0b7"
        },
        "date": 1753279580785,
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
            "value": 26,
            "range": "± 1",
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
            "value": 84,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 139617837,
            "range": "± 1710954",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 16306572,
            "range": "± 148161",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 129980965,
            "range": "± 1247238",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 131557988,
            "range": "± 1103880",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 26851680,
            "range": "± 111302",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 214756465,
            "range": "± 2910450",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 224329712,
            "range": "± 2555024",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2955302,
            "range": "± 8887",
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
            "value": 27682,
            "range": "± 187",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1503265,
            "range": "± 10179",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1593317,
            "range": "± 7475",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 7251,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9461,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 20437,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5743406,
            "range": "± 7324",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 46019159,
            "range": "± 111948",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 50387027,
            "range": "± 209439",
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
          "id": "0df7d7e897b6dae2776affeb8ea613f22f55b3b3",
          "message": "Renamed coverage module.",
          "timestamp": "2025-07-23T20:10:45+02:00",
          "tree_id": "bd5b789626ce7638eae55e1b758152e9ecfba38e",
          "url": "https://github.com/xkikeg/okane/commit/0df7d7e897b6dae2776affeb8ea613f22f55b3b3"
        },
        "date": 1753294743388,
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
            "value": 85,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 138947540,
            "range": "± 1138800",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 16202096,
            "range": "± 402437",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 130113284,
            "range": "± 974596",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 131472229,
            "range": "± 2089000",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 26693286,
            "range": "± 162488",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 214585818,
            "range": "± 3079164",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 224028582,
            "range": "± 2534562",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3014328,
            "range": "± 9058",
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
            "value": 26869,
            "range": "± 333",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1490338,
            "range": "± 16842",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1592008,
            "range": "± 18341",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 7066,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9371,
            "range": "± 239",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 20240,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5728692,
            "range": "± 12977",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 45671551,
            "range": "± 1056536",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 50154959,
            "range": "± 226594",
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
          "id": "cf0c460122a276e5489bb4808106042936c68dea",
          "message": "Refactoring: Separate import::csv module into sub modules. (#264)\n\nThis refactoring allows having more fine-grained modules for csv import.\nThis also ensures that the test coverage increase given some lines has quite small coverage now.",
          "timestamp": "2025-07-29T09:26:33+02:00",
          "tree_id": "32ea1659b97395bc482fba5d374f9ae9488fde1c",
          "url": "https://github.com/xkikeg/okane/commit/cf0c460122a276e5489bb4808106042936c68dea"
        },
        "date": 1753774490921,
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
            "value": 84,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 138847618,
            "range": "± 486901",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 16140736,
            "range": "± 54553",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 128584544,
            "range": "± 1436541",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 130180255,
            "range": "± 1135101",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 26518795,
            "range": "± 63946",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 212047199,
            "range": "± 3091398",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 221772284,
            "range": "± 2554821",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3009685,
            "range": "± 24221",
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
            "value": 27435,
            "range": "± 298",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1480837,
            "range": "± 8413",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1579763,
            "range": "± 11272",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 7011,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9421,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 19927,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5655856,
            "range": "± 100956",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 45331726,
            "range": "± 526034",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 49406446,
            "range": "± 245117",
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
          "id": "a3ddf668d46205893f0b704f41982bd974cf9196",
          "message": "Use one-based crate instead of self defined OneBasedIndex.",
          "timestamp": "2025-07-30T18:48:14+02:00",
          "tree_id": "03cf30e28b3cc82111f6636463efa452f0b7eadd",
          "url": "https://github.com/xkikeg/okane/commit/a3ddf668d46205893f0b704f41982bd974cf9196"
        },
        "date": 1753894617859,
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
            "value": 26,
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
            "value": 85,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 139903126,
            "range": "± 2635927",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 16347181,
            "range": "± 111556",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 130510374,
            "range": "± 2480876",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 131316516,
            "range": "± 1818599",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 26861816,
            "range": "± 88486",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 216122764,
            "range": "± 3608592",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 225167865,
            "range": "± 3382929",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3054120,
            "range": "± 111536",
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
            "value": 27225,
            "range": "± 309",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1492606,
            "range": "± 14321",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1584031,
            "range": "± 9203",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 7026,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9327,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 20086,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5643896,
            "range": "± 47763",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 45442657,
            "range": "± 128639",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 50131737,
            "range": "± 370212",
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
          "id": "b4434a09db1afbac06721ae75cac5bc69d30a478",
          "message": "Update dependencies.",
          "timestamp": "2025-08-13T11:22:41+02:00",
          "tree_id": "17cbec7ceb9f249bd241083c6095a7de404098f6",
          "url": "https://github.com/xkikeg/okane/commit/b4434a09db1afbac06721ae75cac5bc69d30a478"
        },
        "date": 1755077482709,
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
            "name": "to_string plain",
            "value": 85,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 82,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 131719716,
            "range": "± 717194",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 15544625,
            "range": "± 204346",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 123772527,
            "range": "± 1069587",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 125140386,
            "range": "± 1115130",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 25504928,
            "range": "± 160835",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 204482505,
            "range": "± 2565119",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 213166512,
            "range": "± 2677647",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2940367,
            "range": "± 9498",
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
            "value": 26329,
            "range": "± 221",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1524711,
            "range": "± 19099",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1626021,
            "range": "± 9062",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 6971,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9156,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 19639,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5605672,
            "range": "± 7940",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 45082506,
            "range": "± 158626",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 50373588,
            "range": "± 258501",
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
          "id": "546f3d6a39d6ec30386167eb2141c288388dc27c",
          "message": "Support code column for CSV input.\n\nIn the CSV, it should be good to allow users to fill out transaction\ncode.",
          "timestamp": "2025-08-19T09:25:55+02:00",
          "tree_id": "304e1d290c868a193614189333624314d863d29b",
          "url": "https://github.com/xkikeg/okane/commit/546f3d6a39d6ec30386167eb2141c288388dc27c"
        },
        "date": 1755588858362,
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 82,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 133146721,
            "range": "± 729538",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 15413115,
            "range": "± 98302",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 123049680,
            "range": "± 791397",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 124413910,
            "range": "± 472773",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 25467790,
            "range": "± 141131",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 204482299,
            "range": "± 3488661",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 214485210,
            "range": "± 2918288",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2962933,
            "range": "± 24209",
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
            "value": 26351,
            "range": "± 248",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1490703,
            "range": "± 12871",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1579654,
            "range": "± 16474",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 6851,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9311,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 19638,
            "range": "± 154",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5566636,
            "range": "± 9157",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 44938439,
            "range": "± 262047",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 50310196,
            "range": "± 452708",
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
          "id": "1bb3018a6f047ff1b197bf2136eca2a949da0c1f",
          "message": "Add explicit lifetime in the type path, and use consistent lifetime.",
          "timestamp": "2025-09-22T21:29:17+02:00",
          "tree_id": "de74d9dcf07421a8c92d381954ca37bb61df0945",
          "url": "https://github.com/xkikeg/okane/commit/1bb3018a6f047ff1b197bf2136eca2a949da0c1f"
        },
        "date": 1758569878831,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 24,
            "range": "± 2",
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
            "value": 83,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 131449377,
            "range": "± 3852808",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 15332284,
            "range": "± 143177",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 122128436,
            "range": "± 1343659",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 123758236,
            "range": "± 1352840",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 25671944,
            "range": "± 78749",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 204222089,
            "range": "± 3359658",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 214385434,
            "range": "± 3062693",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3034362,
            "range": "± 59028",
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
            "value": 28018,
            "range": "± 71",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1519815,
            "range": "± 47116",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1616869,
            "range": "± 17875",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 7053,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9289,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 19773,
            "range": "± 78",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5616950,
            "range": "± 9478",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 45387384,
            "range": "± 250568",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 50564447,
            "range": "± 2274535",
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
          "id": "4454ba680a52bdc8880b5486bf77fcd61754dd79",
          "message": "Use dunce canonicalization instead of std::fs.\n\nhttps://crates.io/crates/dunce allows to canonicalize\nWindows file path without using UNC path which is not popular.",
          "timestamp": "2025-09-22T21:47:46+02:00",
          "tree_id": "5fbb1b72ac2df3e4dbeecbd8bf7cafec09591a0b",
          "url": "https://github.com/xkikeg/okane/commit/4454ba680a52bdc8880b5486bf77fcd61754dd79"
        },
        "date": 1758570956561,
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
            "value": 81,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 134340328,
            "range": "± 1371421",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 15580548,
            "range": "± 133375",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 123981261,
            "range": "± 1805194",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 125930251,
            "range": "± 1606127",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 25310081,
            "range": "± 109960",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 202484638,
            "range": "± 2722541",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 213188287,
            "range": "± 8621955",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2956226,
            "range": "± 97212",
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
            "value": 27986,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1513684,
            "range": "± 15598",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1594683,
            "range": "± 134166",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 6900,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9164,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 19683,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5566333,
            "range": "± 162847",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 44571103,
            "range": "± 299125",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 49070934,
            "range": "± 430899",
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
          "id": "511ed9b28d5ba8ec4dbb8317fa9fa27d4554886e",
          "message": "Upgrade annotate-snippets + other dependencies.\n\nannotate-snippets required some code change because of major update.\nAlso changed to present account not the entire posting,\nwhen BookKeepError::UndeduciblePostingAmount happened.",
          "timestamp": "2025-09-24T18:22:30+02:00",
          "tree_id": "4a997db292d6026b4232ff81570d1fdfc7c3d34c",
          "url": "https://github.com/xkikeg/okane/commit/511ed9b28d5ba8ec4dbb8317fa9fa27d4554886e"
        },
        "date": 1758731467087,
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
            "value": 85,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 143535640,
            "range": "± 523696",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 16769277,
            "range": "± 113693",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 134973760,
            "range": "± 1155202",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 137962594,
            "range": "± 1203809",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 26587884,
            "range": "± 59969",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 212534714,
            "range": "± 2948856",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 222887558,
            "range": "± 2997065",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3058127,
            "range": "± 11582",
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
            "value": 27986,
            "range": "± 111",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1504202,
            "range": "± 13929",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1594899,
            "range": "± 7209",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 7029,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9251,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 19807,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5595570,
            "range": "± 69803",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 44917849,
            "range": "± 92971",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 49729053,
            "range": "± 333350",
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
          "id": "a33c2d7f1d4855ce7d66bdf60a358a95a714f103",
          "message": "Allow defining alias of alias.\n\nLedger-CLI doesn't prohibit defining already defined account / commodity\nagain. okane needs to be feature parity for this bit.",
          "timestamp": "2025-10-06T15:47:25+02:00",
          "tree_id": "405cd41f855dca869f59158a64d138a89ba764d4",
          "url": "https://github.com/xkikeg/okane/commit/a33c2d7f1d4855ce7d66bdf60a358a95a714f103"
        },
        "date": 1759758962054,
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
            "value": 82,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 141087436,
            "range": "± 294262",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 16276241,
            "range": "± 142503",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 129660416,
            "range": "± 1221974",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 130900718,
            "range": "± 1821774",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 26256346,
            "range": "± 139620",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 209970648,
            "range": "± 3042688",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 219130076,
            "range": "± 2904841",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3042824,
            "range": "± 14301",
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
            "value": 28037,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1501388,
            "range": "± 13482",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1588965,
            "range": "± 14364",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 6949,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9201,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 19898,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5550803,
            "range": "± 16649",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 44519535,
            "range": "± 98019",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 49209191,
            "range": "± 320644",
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
          "id": "e89f737546b2a54b083a99e7255f9844ca5b46eb",
          "message": "Upgrade dependencies.\n\nAlso upgrade min Rust version to 1.82.",
          "timestamp": "2025-10-18T17:45:20+02:00",
          "tree_id": "35735ac83f061d096002989dd3972a0e9c0c67de",
          "url": "https://github.com/xkikeg/okane/commit/e89f737546b2a54b083a99e7255f9844ca5b46eb"
        },
        "date": 1760802847091,
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
            "value": 83,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 135866709,
            "range": "± 375585",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 15941614,
            "range": "± 69226",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 126904087,
            "range": "± 1166869",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 128672434,
            "range": "± 1107648",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 25960845,
            "range": "± 1263260",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 208006585,
            "range": "± 3919554",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 217168646,
            "range": "± 2363847",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3018488,
            "range": "± 12420",
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
            "value": 28002,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1499324,
            "range": "± 12293",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1590384,
            "range": "± 4877",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 6941,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9315,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 19998,
            "range": "± 104",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5571365,
            "range": "± 16656",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 44716838,
            "range": "± 81404",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 49126505,
            "range": "± 290915",
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
          "id": "b397ca7f30c71cef9c06b6f461bbb39025d9b0ce",
          "message": "Canonicalize Windows path on import matching to ensure separator is reverse slash.\n\nWe use PathBufExt::from_slash for the entry.path to use back-slash for separator.\nHowever, user given path might use slash and passed to select().\nthus we have to unify both path to use system separator with soft_canonicalize.\n(Note fs::canonicalize is safe to use, but chose to use soft_canonicalize for unit tests.)",
          "timestamp": "2025-10-20T00:22:01+02:00",
          "tree_id": "656e21be8377c34916dae4acfa6da18989fb29f8",
          "url": "https://github.com/xkikeg/okane/commit/b397ca7f30c71cef9c06b6f461bbb39025d9b0ce"
        },
        "date": 1760913014592,
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
            "value": 83,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 136810939,
            "range": "± 358128",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 15926218,
            "range": "± 271113",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 127779236,
            "range": "± 972580",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 128840236,
            "range": "± 951214",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 26042500,
            "range": "± 131470",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 208882422,
            "range": "± 2709012",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 217784526,
            "range": "± 4032592",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3019341,
            "range": "± 12988",
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
            "value": 28016,
            "range": "± 240",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1506944,
            "range": "± 13301",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1612862,
            "range": "± 17802",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 6897,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 9426,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 19874,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 5547985,
            "range": "± 10248",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 44579982,
            "range": "± 236355",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 48984136,
            "range": "± 348516",
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
          "id": "b852def624fa97322c89ef557b4178f284931347",
          "message": "Stop using unwrap for Display instance of commodities.",
          "timestamp": "2025-11-01T20:08:10+01:00",
          "tree_id": "feccb0ff4d5e2506b4e3cb521ebc0aedbfade726",
          "url": "https://github.com/xkikeg/okane/commit/b852def624fa97322c89ef557b4178f284931347"
        },
        "date": 1762024614702,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 24,
            "range": "± 1",
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 82,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 135292587,
            "range": "± 2281328",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 15792110,
            "range": "± 274229",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 127931140,
            "range": "± 2034824",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 128886496,
            "range": "± 2828104",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 27040540,
            "range": "± 370138",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 215806232,
            "range": "± 2914491",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 223727315,
            "range": "± 3067745",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3008096,
            "range": "± 18562",
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
            "value": 25169,
            "range": "± 189",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1487573,
            "range": "± 45543",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1586397,
            "range": "± 8809",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 6012,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 8295,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 16172,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4912423,
            "range": "± 10894",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 39610123,
            "range": "± 185131",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 44342329,
            "range": "± 228796",
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
          "id": "515e609f6fe526f7a48db82f2788aadd0c15d281",
          "message": "Remove HashMap from PriceDB computing path.\n\nThis might speed up conversion performance.",
          "timestamp": "2025-11-02T07:40:29+01:00",
          "tree_id": "b92c9bce936f9db7d7a67d28408326de16642cf2",
          "url": "https://github.com/xkikeg/okane/commit/515e609f6fe526f7a48db82f2788aadd0c15d281"
        },
        "date": 1762066120296,
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
            "value": 81,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 134505671,
            "range": "± 427729",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 15560404,
            "range": "± 57837",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 125586664,
            "range": "± 1660332",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 126983129,
            "range": "± 683419",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 25017906,
            "range": "± 133809",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 199989034,
            "range": "± 1930017",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 209225718,
            "range": "± 2592465",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2983095,
            "range": "± 8642",
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
            "value": 25791,
            "range": "± 318",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1474975,
            "range": "± 8830",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1570040,
            "range": "± 14345",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5680,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7580,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 13281,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4475013,
            "range": "± 14421",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 36291810,
            "range": "± 102844",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 38326119,
            "range": "± 215115",
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
          "id": "4f17e7064ea9da9132f92613a4bc5acd5250fd81",
          "message": "Use bumpalo-intern DirectInternStore.\n\nThis is a simple refactoring, factoring out the DirectInternStore\nout of okane-core module.",
          "timestamp": "2025-11-02T10:53:12+01:00",
          "tree_id": "7f1cdc11aadca772fafe6adbb5eed0d9ba14d68d",
          "url": "https://github.com/xkikeg/okane/commit/4f17e7064ea9da9132f92613a4bc5acd5250fd81"
        },
        "date": 1762077691766,
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
            "value": 81,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 141214147,
            "range": "± 1431554",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 16386800,
            "range": "± 244521",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 130969806,
            "range": "± 1121494",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 132531075,
            "range": "± 1917887",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 25959201,
            "range": "± 172788",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 206350806,
            "range": "± 2863434",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 216806000,
            "range": "± 3042673",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3010874,
            "range": "± 41359",
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
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 25422,
            "range": "± 91",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1479475,
            "range": "± 16094",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1572207,
            "range": "± 13333",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5572,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7860,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 13484,
            "range": "± 42",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4468944,
            "range": "± 14404",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 36395691,
            "range": "± 384887",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 39267532,
            "range": "± 886046",
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
          "id": "2e089445f559808b0e7c31ef612ac1110cd98280",
          "message": "Bump version to 0.16.0.",
          "timestamp": "2025-11-02T11:17:40+01:00",
          "tree_id": "91acb49af53efb6c202945f82943bd87d8f2d4fe",
          "url": "https://github.com/xkikeg/okane/commit/2e089445f559808b0e7c31ef612ac1110cd98280"
        },
        "date": 1762079182459,
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
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 81,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 142246284,
            "range": "± 703267",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 16921173,
            "range": "± 74060",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 134604533,
            "range": "± 1378732",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 137151386,
            "range": "± 780494",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 26602957,
            "range": "± 120142",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 212288067,
            "range": "± 2300128",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 223046470,
            "range": "± 2329053",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3046012,
            "range": "± 58987",
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
            "value": 28039,
            "range": "± 153",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1491496,
            "range": "± 12168",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1572536,
            "range": "± 17333",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5564,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7902,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 13574,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4482468,
            "range": "± 15479",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 36234891,
            "range": "± 200854",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 39539147,
            "range": "± 555778",
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
          "id": "0f44de2b20e010aa568cf602c21b606449c168ac",
          "message": "Bump cli deps to 0.16.0",
          "timestamp": "2025-11-02T11:25:58+01:00",
          "tree_id": "7c588ee94858cefdfda41dbe470d7d5f6ffc5540",
          "url": "https://github.com/xkikeg/okane/commit/0f44de2b20e010aa568cf602c21b606449c168ac"
        },
        "date": 1762079671544,
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
            "value": 83,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 146434563,
            "range": "± 592732",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 17046832,
            "range": "± 123704",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 136540284,
            "range": "± 834632",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 139189767,
            "range": "± 1228974",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 27124425,
            "range": "± 89995",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 216502443,
            "range": "± 1592681",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 226187951,
            "range": "± 1479820",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3052203,
            "range": "± 10821",
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
            "value": 27993,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1501721,
            "range": "± 16600",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1589054,
            "range": "± 20857",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5714,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7905,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 13360,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4467806,
            "range": "± 16502",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 36532861,
            "range": "± 177775",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 38618254,
            "range": "± 296740",
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
          "id": "d789c934fdfefef689d6a4f69978aa672a381c81",
          "message": "Fixed all Cargo clippy warnings.",
          "timestamp": "2025-11-13T10:59:42+01:00",
          "tree_id": "207ef4ff15b74c6e5c2a14cefbdb551edc9d61e6",
          "url": "https://github.com/xkikeg/okane/commit/d789c934fdfefef689d6a4f69978aa672a381c81"
        },
        "date": 1763028508232,
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
            "value": 83,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 134263767,
            "range": "± 426888",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 15652186,
            "range": "± 279952",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 125776957,
            "range": "± 3412548",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 127470735,
            "range": "± 3002393",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 26319249,
            "range": "± 549971",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 211181529,
            "range": "± 2678893",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 220005465,
            "range": "± 3769048",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3010431,
            "range": "± 10287",
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
            "value": 25352,
            "range": "± 86",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1477956,
            "range": "± 15646",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1558583,
            "range": "± 9683",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5786,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7924,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 14182,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4582321,
            "range": "± 15058",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 36908942,
            "range": "± 101064",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 38880736,
            "range": "± 173507",
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
          "id": "35f70eebbdcd2499f122533b5d6b9c1fd05a408d",
          "message": "Balance non-balanced posting with Equity:Adjustments.\n\nThis is useful and reduces wasteful work of fixing the balance.",
          "timestamp": "2025-11-30T22:54:37+01:00",
          "tree_id": "0199b0534b7fb91cd96cef11ef2fd86390dfe560",
          "url": "https://github.com/xkikeg/okane/commit/35f70eebbdcd2499f122533b5d6b9c1fd05a408d"
        },
        "date": 1764540173238,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse comma",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string plain",
            "value": 84,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 71,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 113704642,
            "range": "± 1628163",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 13853128,
            "range": "± 118348",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 111718048,
            "range": "± 398412",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 111868455,
            "range": "± 980514",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 22983548,
            "range": "± 112629",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 184346202,
            "range": "± 2145218",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 192474063,
            "range": "± 1596254",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3149607,
            "range": "± 90201",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 27037,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1401133,
            "range": "± 9208",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1490744,
            "range": "± 26596",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5061,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7162,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 11946,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 3955202,
            "range": "± 52806",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 33163304,
            "range": "± 125875",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 39673293,
            "range": "± 122021",
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
          "id": "7a9ab5eed5f7cf821db72e9c92c01c608ae63e04",
          "message": "Made InstdAmt and TxAmt in the AmtDtls optional.\n\nThese amounts are actually optional, not mandatory field.",
          "timestamp": "2025-12-04T23:09:26+01:00",
          "tree_id": "1f970d46e8cb210fe44b7f80ffac10967bcef891",
          "url": "https://github.com/xkikeg/okane/commit/7a9ab5eed5f7cf821db72e9c92c01c608ae63e04"
        },
        "date": 1764886637514,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse comma",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string plain",
            "value": 83,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 70,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 113960768,
            "range": "± 1250749",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 13763345,
            "range": "± 121217",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 111503211,
            "range": "± 889400",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 112977837,
            "range": "± 1010763",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 23377712,
            "range": "± 74204",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 186151022,
            "range": "± 1686773",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 194504378,
            "range": "± 1451056",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3723143,
            "range": "± 81592",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 11,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 26972,
            "range": "± 1358",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1383982,
            "range": "± 19307",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1482763,
            "range": "± 11510",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5083,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7096,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 11896,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 3978778,
            "range": "± 54724",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 33336316,
            "range": "± 493551",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 40362446,
            "range": "± 4070540",
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
          "id": "b569feb2100d85ea90c721fe9759f54635d64fe3",
          "message": "Remove library-level import integration test.\n\nThis made all library public exported,\nthis is not ideal as the binary crate.",
          "timestamp": "2025-12-06T11:53:10+01:00",
          "tree_id": "f8714c45c1e878bb295305895a73f978adaad95a",
          "url": "https://github.com/xkikeg/okane/commit/b569feb2100d85ea90c721fe9759f54635d64fe3"
        },
        "date": 1765018876655,
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
            "value": 83,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 132311265,
            "range": "± 556142",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 15322093,
            "range": "± 113123",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 121774341,
            "range": "± 1356981",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 123653396,
            "range": "± 1347201",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 25269735,
            "range": "± 141136",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 201750870,
            "range": "± 2363228",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 211082211,
            "range": "± 1916745",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3036427,
            "range": "± 15754",
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
            "value": 28028,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1501405,
            "range": "± 10992",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1582454,
            "range": "± 14263",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5650,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7822,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 13429,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4476064,
            "range": "± 20992",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 36153061,
            "range": "± 103710",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 38766326,
            "range": "± 174898",
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
          "id": "1d7100fded5c43dc7a274249fbb960895f1a6bd9",
          "message": "Factor out common logic constructing Txn from Fragment.\n\nThis addresses #291 issue.",
          "timestamp": "2025-12-06T13:59:52+01:00",
          "tree_id": "f0ffaeac3f5cf9b70bf76ae9f5e4ad322934b381",
          "url": "https://github.com/xkikeg/okane/commit/1d7100fded5c43dc7a274249fbb960895f1a6bd9"
        },
        "date": 1765026486029,
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
            "value": 82,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 134635432,
            "range": "± 1653305",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 15732675,
            "range": "± 130409",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 126304276,
            "range": "± 861292",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 127405877,
            "range": "± 1860647",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 25389522,
            "range": "± 83302",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 203521303,
            "range": "± 1916701",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 211900041,
            "range": "± 1777860",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2990648,
            "range": "± 19589",
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
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1491736,
            "range": "± 8883",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1590512,
            "range": "± 16163",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5582,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7719,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 13516,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4507463,
            "range": "± 17189",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 36433278,
            "range": "± 139846",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 38489396,
            "range": "± 264073",
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
          "id": "bfe0dfd458f707e3f87a8984becbe00b4ec12ab7",
          "message": "Refactored Extractor filter into single type.\n\nIt used to have an individual matcher for each format,\nbut that's costful and would prevent from extending it.",
          "timestamp": "2025-12-15T17:39:36+01:00",
          "tree_id": "97cf7560e0a9291a49073eaa9f1728ebee4a5f21",
          "url": "https://github.com/xkikeg/okane/commit/bfe0dfd458f707e3f87a8984becbe00b4ec12ab7"
        },
        "date": 1765817298521,
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
            "value": 84,
            "range": "± 0",
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
            "value": 127489457,
            "range": "± 309511",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 14892738,
            "range": "± 83319",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 117921743,
            "range": "± 528595",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 119034784,
            "range": "± 789842",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 24483464,
            "range": "± 134020",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 196312338,
            "range": "± 1847184",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 205544401,
            "range": "± 2105497",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2965180,
            "range": "± 10079",
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
            "value": 12464,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1370843,
            "range": "± 13415",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1458495,
            "range": "± 15574",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5596,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7841,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 13313,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4545873,
            "range": "± 34231",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 36820141,
            "range": "± 154504",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 39235261,
            "range": "± 214209",
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
          "id": "f5672125263d1a7b53c6291f7cc13d5d8ff6a78a",
          "message": "Added secondary_commodity to viseca and Camt053.",
          "timestamp": "2025-12-15T18:32:10+01:00",
          "tree_id": "d97278e1cf830233419e4b24474294175cf4173c",
          "url": "https://github.com/xkikeg/okane/commit/f5672125263d1a7b53c6291f7cc13d5d8ff6a78a"
        },
        "date": 1765820418148,
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
            "value": 128438102,
            "range": "± 3827185",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 14936010,
            "range": "± 183125",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 119957012,
            "range": "± 787184",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 121389319,
            "range": "± 1458881",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 24610614,
            "range": "± 416420",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 196710302,
            "range": "± 1012552",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 204996767,
            "range": "± 2059248",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2948954,
            "range": "± 12347",
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
            "value": 12463,
            "range": "± 70",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1373752,
            "range": "± 11731",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1460337,
            "range": "± 12768",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5587,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7953,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 13312,
            "range": "± 85",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4483844,
            "range": "± 18880",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 36175073,
            "range": "± 153611",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 37923995,
            "range": "± 168768",
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
          "id": "b95a0f230ac75a180ec1b28224faffa64cde09f7",
          "message": "Support Fragment provided secondary_commodity for matcher.\n\nThis allows one rule to apply secondary_commodity via conversion spec,\nand the following rule to match against the specified secondary_commodity.\n\nThis will be tested as part of #298 in end-to-end manner.",
          "timestamp": "2025-12-17T19:54:44+01:00",
          "tree_id": "39116fea84bf72414ea2ef0582dca1427d159d03",
          "url": "https://github.com/xkikeg/okane/commit/b95a0f230ac75a180ec1b28224faffa64cde09f7"
        },
        "date": 1765998172886,
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
            "range": "± 4",
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
            "value": 126916961,
            "range": "± 425466",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 14894802,
            "range": "± 87682",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 118499726,
            "range": "± 830499",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 119096791,
            "range": "± 880109",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 25040677,
            "range": "± 82718",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 199396205,
            "range": "± 2410378",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 208826065,
            "range": "± 4241336",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2945511,
            "range": "± 84052",
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
            "value": 12465,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1396534,
            "range": "± 26696",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1469114,
            "range": "± 88305",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5663,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7806,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 13400,
            "range": "± 333",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4506131,
            "range": "± 16495",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 36339766,
            "range": "± 859028",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 38663812,
            "range": "± 228663",
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
          "id": "f7dc39d4118fea94a08d8afc3b411e9164b1e816",
          "message": "Added Viseca entry so that we can check if reimburse is correct.\n\nWe need to have two entry, pay and pay-back, so to compare those two.",
          "timestamp": "2025-12-17T22:31:40+01:00",
          "tree_id": "bb1430f9b1794cc9f86c212f5065b10b294820b0",
          "url": "https://github.com/xkikeg/okane/commit/f7dc39d4118fea94a08d8afc3b411e9164b1e816"
        },
        "date": 1766007588090,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 24,
            "range": "± 1",
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
            "value": 84,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 129090164,
            "range": "± 396171",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 14998090,
            "range": "± 409138",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 120847305,
            "range": "± 1570363",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 122058630,
            "range": "± 786324",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 24864391,
            "range": "± 859589",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 198003986,
            "range": "± 3226230",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 207426698,
            "range": "± 3556527",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2956557,
            "range": "± 93679",
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
            "value": 12455,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1378613,
            "range": "± 13030",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1481321,
            "range": "± 17095",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5698,
            "range": "± 370",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7762,
            "range": "± 377",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 13487,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4708780,
            "range": "± 199686",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 38010650,
            "range": "± 1352188",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 39641531,
            "range": "± 843184",
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
          "id": "17cf60c516665c7421995e50223bfdacfcc67f76",
          "message": "Remove okane_golden::read_as_utf8.\n\nThis function is esssentially std::fs::read_to_string\nwith CRLF -> LF replacement, which is confusing.",
          "timestamp": "2025-12-18T09:38:38+01:00",
          "tree_id": "2646111f44a1d88a9f14a1ec48f8a3fc34074763",
          "url": "https://github.com/xkikeg/okane/commit/17cf60c516665c7421995e50223bfdacfcc67f76"
        },
        "date": 1766047725799,
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
            "range": "± 9",
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
            "value": 127683907,
            "range": "± 1294951",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 15211711,
            "range": "± 177956",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 121530254,
            "range": "± 1527839",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 122293892,
            "range": "± 1439887",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 24749693,
            "range": "± 97254",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 196685701,
            "range": "± 571526",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 202649705,
            "range": "± 3092865",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2911118,
            "range": "± 55304",
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
            "value": 12280,
            "range": "± 273",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1352319,
            "range": "± 33436",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1457389,
            "range": "± 32016",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5481,
            "range": "± 130",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7638,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 13294,
            "range": "± 36",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4455195,
            "range": "± 13766",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 36161069,
            "range": "± 176379",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 37632136,
            "range": "± 184394",
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
          "id": "0bdcf5835b9b0148c4227304063ee326fe1bb1aa",
          "message": "Add commodity matcher, which is natural extension to secondary_commodity.",
          "timestamp": "2025-12-18T18:36:23+01:00",
          "tree_id": "ac6a8c4a1e87876096c6befce59b6a63a2fe596b",
          "url": "https://github.com/xkikeg/okane/commit/0bdcf5835b9b0148c4227304063ee326fe1bb1aa"
        },
        "date": 1766079862547,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 24,
            "range": "± 3",
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
            "range": "± 0",
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
            "value": 130402185,
            "range": "± 514698",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 15449002,
            "range": "± 95401",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 123270543,
            "range": "± 909290",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 124721638,
            "range": "± 923731",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 24785957,
            "range": "± 76262",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 198882229,
            "range": "± 1625546",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 206986252,
            "range": "± 1669922",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2952000,
            "range": "± 12002",
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
            "value": 12463,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1387528,
            "range": "± 28717",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1476365,
            "range": "± 12957",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5600,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7774,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 13342,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4491376,
            "range": "± 16600",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 36377156,
            "range": "± 133940",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 38312781,
            "range": "± 194236",
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
          "id": "a6a931913b5cdc7ff825cfea17f61f5b6f41b294",
          "message": "Fixup: Add commodity matcher field.\n\nThis was left over in PR #301.",
          "timestamp": "2025-12-19T09:10:10+01:00",
          "tree_id": "ece10fac4194965b0ed064353c456660ea2cd38f",
          "url": "https://github.com/xkikeg/okane/commit/a6a931913b5cdc7ff825cfea17f61f5b6f41b294"
        },
        "date": 1766132293923,
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 82,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 131394279,
            "range": "± 765515",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 14938473,
            "range": "± 77845",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 119548801,
            "range": "± 830361",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 121005650,
            "range": "± 1138727",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 24536757,
            "range": "± 116161",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 196545080,
            "range": "± 2297746",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 205333216,
            "range": "± 2203811",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2939424,
            "range": "± 17443",
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
            "value": 12459,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1377800,
            "range": "± 14461",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1466447,
            "range": "± 11305",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5681,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7684,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 13300,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4483729,
            "range": "± 13254",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 36212638,
            "range": "± 156014",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 38594081,
            "range": "± 268380",
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
          "id": "34a971e990a3232d1ebbf32def6aa8ac1e3c6fae",
          "message": "Add hidden_fee feature: guess original rate and put commissions.\n\nWith this feature, the tool can deduce the hidden fee added to the\ncommodity rate, and guess the cost in automated way.",
          "timestamp": "2025-12-19T13:16:50+01:00",
          "tree_id": "2ab720de18558210b83295375f7f2082ab7b85a4",
          "url": "https://github.com/xkikeg/okane/commit/34a971e990a3232d1ebbf32def6aa8ac1e3c6fae"
        },
        "date": 1766147096874,
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
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string plain",
            "value": 82,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 73,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 108902649,
            "range": "± 256271",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 13259350,
            "range": "± 111439",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 106741547,
            "range": "± 1338871",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 108602166,
            "range": "± 964653",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 23074510,
            "range": "± 158280",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 184028354,
            "range": "± 1972364",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 192732111,
            "range": "± 2945997",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3419296,
            "range": "± 110990",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 11589,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1261723,
            "range": "± 16282",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1368007,
            "range": "± 22490",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 4962,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 6787,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 11616,
            "range": "± 66",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 3942210,
            "range": "± 21112",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 33253330,
            "range": "± 117618",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 40633202,
            "range": "± 217969",
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
          "id": "54727ca7c742c60717c5dc99eba0523b1fbd528e",
          "message": "Fixed all clippy warnings, also added Commodity::is_empty.",
          "timestamp": "2025-12-23T17:53:50+01:00",
          "tree_id": "ad2cfaf605e220bb4e274a5f8588f9c9113780dc",
          "url": "https://github.com/xkikeg/okane/commit/54727ca7c742c60717c5dc99eba0523b1fbd528e"
        },
        "date": 1766509311577,
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
            "value": 84,
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
            "value": 127181754,
            "range": "± 969611",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 14752541,
            "range": "± 72985",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 118292077,
            "range": "± 2347685",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 119055738,
            "range": "± 1041104",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 24640866,
            "range": "± 116977",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 197653276,
            "range": "± 2367668",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 206993435,
            "range": "± 2267179",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2954093,
            "range": "± 21948",
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
            "value": 12457,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1385860,
            "range": "± 18915",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1471970,
            "range": "± 17584",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5558,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7644,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 13289,
            "range": "± 56",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4470235,
            "range": "± 15264",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 36075118,
            "range": "± 152584",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 37958612,
            "range": "± 184977",
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
          "id": "990d85dda5c590b415115f9bacab42e691a0b9ac",
          "message": "Refactored single_entry to handle hidden fee better.\n\nNow this to_double_entry handles transaction generation in 4 waves,\n1. add posting amount as-is (without rate),\n2. convert rate with hidden_fee\n3. append rate, add hidden fee commission if exists\n4. append excess if exists.",
          "timestamp": "2025-12-23T18:02:50+01:00",
          "tree_id": "4ccc0f3410cbb507df008f04b549bb37c044a7c6",
          "url": "https://github.com/xkikeg/okane/commit/990d85dda5c590b415115f9bacab42e691a0b9ac"
        },
        "date": 1766509846348,
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
            "value": 127755373,
            "range": "± 588733",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 14848339,
            "range": "± 80157",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 118155748,
            "range": "± 797579",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 118412085,
            "range": "± 1478094",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 25231132,
            "range": "± 78712",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 201505670,
            "range": "± 1733804",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 210095226,
            "range": "± 2833775",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2944600,
            "range": "± 13745",
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
            "value": 12476,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1374857,
            "range": "± 4945",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1464286,
            "range": "± 14802",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5624,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7686,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 13239,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4449799,
            "range": "± 13944",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 36152244,
            "range": "± 146002",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 37820715,
            "range": "± 245094",
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
          "id": "cdcc97b3e31048a4f182b2f7ddd93a7eb87d05c7",
          "message": "Release 0.17.0 (also golden 0.2.0)",
          "timestamp": "2025-12-24T23:38:31+01:00",
          "tree_id": "976e7007b9d4497734db443df6e2e9a87c8c6ec7",
          "url": "https://github.com/xkikeg/okane/commit/cdcc97b3e31048a4f182b2f7ddd93a7eb87d05c7"
        },
        "date": 1766616382748,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "parse comma",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string plain",
            "value": 83,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 75,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 114610620,
            "range": "± 1114802",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 14104113,
            "range": "± 390294",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 112757995,
            "range": "± 717076",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 115402902,
            "range": "± 1851133",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 23432263,
            "range": "± 75089",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 187822790,
            "range": "± 1370522",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 196782317,
            "range": "± 4312689",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3121075,
            "range": "± 91973",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 11589,
            "range": "± 240",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1272130,
            "range": "± 46481",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1362071,
            "range": "± 11209",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5014,
            "range": "± 61",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 6780,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 11568,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 3868892,
            "range": "± 9151",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 32643233,
            "range": "± 171232",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 39790847,
            "range": "± 216903",
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
          "id": "6ee03552f5e0e4b51de91f374238022b9cc64698",
          "message": "Fixed a clippy warning.",
          "timestamp": "2025-12-25T12:28:53+01:00",
          "tree_id": "849bceec0de732e6d893ea64b98f1d4abf1d8240",
          "url": "https://github.com/xkikeg/okane/commit/6ee03552f5e0e4b51de91f374238022b9cc64698"
        },
        "date": 1766662633490,
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
            "value": 91,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 82,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 129885969,
            "range": "± 1481809",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 15209488,
            "range": "± 204035",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 121005462,
            "range": "± 2235288",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 122525191,
            "range": "± 1498570",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 24236544,
            "range": "± 135420",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 194529527,
            "range": "± 2106615",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 202813836,
            "range": "± 2216703",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3045590,
            "range": "± 10811",
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
            "value": 12463,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1375495,
            "range": "± 15856",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1471063,
            "range": "± 8156",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5630,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7733,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 13343,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4484156,
            "range": "± 12210",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 36249863,
            "range": "± 201952",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 38448834,
            "range": "± 918052",
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
          "id": "571eac80c28a9ad87584266d200f1bea26274700",
          "message": "Added 2 PRs into CHANGELOG.",
          "timestamp": "2025-12-25T14:40:22+01:00",
          "tree_id": "3ed4467da915fd55f6c47ed6321f1353f6a47fbf",
          "url": "https://github.com/xkikeg/okane/commit/571eac80c28a9ad87584266d200f1bea26274700"
        },
        "date": 1766670531346,
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 82,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 134973890,
            "range": "± 304173",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 15848373,
            "range": "± 106368",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 126881462,
            "range": "± 701103",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 129335366,
            "range": "± 746163",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 25886171,
            "range": "± 158282",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 207147043,
            "range": "± 2104685",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 217091772,
            "range": "± 1885799",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3000973,
            "range": "± 13099",
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
            "value": 12638,
            "range": "± 178",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1378283,
            "range": "± 6297",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1472695,
            "range": "± 13127",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5750,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7831,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 13499,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4514459,
            "range": "± 8998",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 36413198,
            "range": "± 109989",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 38926334,
            "range": "± 217030",
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
          "id": "9c9c83b85c06376a56c5199f457dfdca0ca04a13",
          "message": "Implemented --current option to balance conversion.\n\nThis allows easy query checking the current balance of assets /\nliabilities.\nAlso renamed --now to --today given it's just 'day' precision.",
          "timestamp": "2026-01-02T00:25:55+01:00",
          "tree_id": "d7bedf98c43de0536aff1ed34bae363af1d57318",
          "url": "https://github.com/xkikeg/okane/commit/9c9c83b85c06376a56c5199f457dfdca0ca04a13"
        },
        "date": 1767310453405,
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
            "value": 85,
            "range": "± 0",
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
            "value": 137040898,
            "range": "± 1819894",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 15963440,
            "range": "± 210601",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 127559533,
            "range": "± 1198432",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 130193889,
            "range": "± 1069211",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 25804506,
            "range": "± 128356",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 206272015,
            "range": "± 2497564",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 216663896,
            "range": "± 1883108",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3000947,
            "range": "± 15135",
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
            "value": 12517,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1382107,
            "range": "± 9260",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1473518,
            "range": "± 13884",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5650,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7726,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 13362,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4561370,
            "range": "± 23109",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 36998086,
            "range": "± 344227",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 39221837,
            "range": "± 279960",
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
          "id": "79627633696b159d7d5ae508c8d0c2c30c507397",
          "message": "Renamed UpToDate conversion strategy field to today.\n\nIt's not `now` (timestamp), but `today` (just a yyyy/mm/dd tuple).",
          "timestamp": "2026-01-02T01:59:16+01:00",
          "tree_id": "07e4fa40885d6808797d333d20d9bae80cd21519",
          "url": "https://github.com/xkikeg/okane/commit/79627633696b159d7d5ae508c8d0c2c30c507397"
        },
        "date": 1767316051865,
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
            "value": 85,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 81,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 136558391,
            "range": "± 1756705",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 15844719,
            "range": "± 130439",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 127339421,
            "range": "± 1031440",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 130596855,
            "range": "± 2097734",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 25748257,
            "range": "± 162893",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 205525680,
            "range": "± 2402146",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 216495463,
            "range": "± 2108466",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2996903,
            "range": "± 16783",
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
            "value": 12447,
            "range": "± 53",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1394907,
            "range": "± 18967",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1482728,
            "range": "± 16657",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5699,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7880,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 13370,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4492869,
            "range": "± 26056",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 36360380,
            "range": "± 202846",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 38558947,
            "range": "± 362361",
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
          "id": "c8b99f016ab3d1a2f1155c8a459f8fd15d2ffc91",
          "message": "Use BTreeMap instead of HashMap for Amount.\n\n* CommodityTag is now just a single usize. Cheap to compare.\n* HashMap gives undeterministic order, not good for as_inline_display().\n* Let's see performance regression.",
          "timestamp": "2026-01-02T19:59:04+01:00",
          "tree_id": "9868d740df7f2e3f5e2c96410f0f35126f4f2ab3",
          "url": "https://github.com/xkikeg/okane/commit/c8b99f016ab3d1a2f1155c8a459f8fd15d2ffc91"
        },
        "date": 1767380825767,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 24,
            "range": "± 1",
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 81,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 126339883,
            "range": "± 2332169",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 14636027,
            "range": "± 153244",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 119635543,
            "range": "± 1594669",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 120894189,
            "range": "± 1176145",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 23974261,
            "range": "± 117044",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 191876454,
            "range": "± 1153667",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 202102191,
            "range": "± 1079740",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2992262,
            "range": "± 16044",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 12457,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1307866,
            "range": "± 28088",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1458630,
            "range": "± 36569",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 4971,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 6832,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 11051,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4123731,
            "range": "± 22441",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 33426520,
            "range": "± 240569",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 35792456,
            "range": "± 205840",
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
          "id": "6d5135140bd2c19db3b6191160f0e859e1eca90a",
          "message": "Added detailed error for BookKeepError::EvalFailure.\n\nThis is a part of https://github.com/xkikeg/okane/issues/267.",
          "timestamp": "2026-01-03T02:24:03+01:00",
          "tree_id": "a719c8d552fbcd10e782fa43ab5a39bfd23cb06d",
          "url": "https://github.com/xkikeg/okane/commit/6d5135140bd2c19db3b6191160f0e859e1eca90a"
        },
        "date": 1767403936120,
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
            "value": 81,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 130463961,
            "range": "± 964623",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 15186842,
            "range": "± 79232",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 123583244,
            "range": "± 693070",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 125389373,
            "range": "± 1031232",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 23873260,
            "range": "± 749966",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 190523887,
            "range": "± 634077",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 201359011,
            "range": "± 2703914",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2983510,
            "range": "± 19563",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 12457,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1262111,
            "range": "± 17836",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1411080,
            "range": "± 11547",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5030,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 6909,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 11277,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4141009,
            "range": "± 26055",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 33510403,
            "range": "± 163214",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 36196990,
            "range": "± 633343",
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
          "id": "1439c88a5249aba5108e46bab7dd169680565abb",
          "message": "Gives detailed error message for balance assertion failure.",
          "timestamp": "2026-01-04T17:32:52+01:00",
          "tree_id": "7d3579fc961a60ce25a6e1c567a16e7b93567eff",
          "url": "https://github.com/xkikeg/okane/commit/1439c88a5249aba5108e46bab7dd169680565abb"
        },
        "date": 1767544885660,
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
            "value": 85,
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
            "value": 126171911,
            "range": "± 849909",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 14754368,
            "range": "± 241522",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 119466685,
            "range": "± 545600",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 121013172,
            "range": "± 607795",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 23132435,
            "range": "± 172677",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 185645343,
            "range": "± 2099226",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 197053657,
            "range": "± 1987880",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2956659,
            "range": "± 7676",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 12460,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1431533,
            "range": "± 55141",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1570847,
            "range": "± 53879",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5279,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7120,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 11342,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4345727,
            "range": "± 29324",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 35187129,
            "range": "± 209782",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 36947012,
            "range": "± 345872",
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
          "id": "6d8a72a4441fdf4b983d71e98ae68d17e7befc9e",
          "message": "Recover logging on unspecified account import.\n\nThe log was accidentally removed in:\nhttps://github.com/xkikeg/okane/pull/292\nhttps://github.com/xkikeg/okane/commit/1d7100fded5c43dc7a274249fbb960895f1a6bd9",
          "timestamp": "2026-01-08T10:11:26+01:00",
          "tree_id": "a40fb1101ca8f2c5e58f91528adb664b1ba53942",
          "url": "https://github.com/xkikeg/okane/commit/6d8a72a4441fdf4b983d71e98ae68d17e7befc9e"
        },
        "date": 1767864012198,
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
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 82,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 126726432,
            "range": "± 922901",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 14620517,
            "range": "± 89259",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 119337438,
            "range": "± 1775031",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 120572792,
            "range": "± 1876204",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 22885125,
            "range": "± 153009",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 183235251,
            "range": "± 2468698",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 193963993,
            "range": "± 2043903",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2966211,
            "range": "± 10850",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 12462,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1437328,
            "range": "± 27170",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1566213,
            "range": "± 28681",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5176,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7104,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 11299,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4357086,
            "range": "± 25747",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 35083935,
            "range": "± 212700",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 36877795,
            "range": "± 295980",
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
          "id": "abcf155c112a40e9501d5135d678077cdbc4e594",
          "message": "Allow importing tsv file as well.\n\nThe input can be TSV (tab separated values) instead of CSV.\nWith this change we also refactored import module / cmd module,\nmoved responsibility into import side.",
          "timestamp": "2026-01-08T22:53:40+01:00",
          "tree_id": "86948551bf603cdb9a5020c655d50b5dd3a187be",
          "url": "https://github.com/xkikeg/okane/commit/abcf155c112a40e9501d5135d678077cdbc4e594"
        },
        "date": 1767909705884,
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
            "value": 126211394,
            "range": "± 1376739",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 14706638,
            "range": "± 109956",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 119309847,
            "range": "± 1560036",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 121580329,
            "range": "± 899700",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 23053775,
            "range": "± 169967",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 184097347,
            "range": "± 2036093",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 195059083,
            "range": "± 3041878",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2951933,
            "range": "± 15704",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 12462,
            "range": "± 299",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1437915,
            "range": "± 65993",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1569245,
            "range": "± 70800",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5230,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7124,
            "range": "± 138",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 11318,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4379125,
            "range": "± 78204",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 35364212,
            "range": "± 612617",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 38199688,
            "range": "± 653787",
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
          "id": "3fb9d5a4905c22e5ffffd940cfd69fef66363695",
          "message": "Corrected syntax documentation regarding metadata.\n\nHere metadata is defined without spaces / newlines.\nSo we need to append them in some cases.\n\nhttps://github.com/xkikeg/okane/issues/320",
          "timestamp": "2026-01-08T23:08:31+01:00",
          "tree_id": "f8348b2d5b6be5e6b9b792012c123f79f7cff28f",
          "url": "https://github.com/xkikeg/okane/commit/3fb9d5a4905c22e5ffffd940cfd69fef66363695"
        },
        "date": 1767910592516,
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
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 82,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 127314945,
            "range": "± 2456814",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 14760682,
            "range": "± 143261",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 120372936,
            "range": "± 2134369",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 119479135,
            "range": "± 745969",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 23193758,
            "range": "± 302077",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 185042377,
            "range": "± 1336641",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 193994031,
            "range": "± 1275674",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2946009,
            "range": "± 118670",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 12491,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1446245,
            "range": "± 9920",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1583073,
            "range": "± 14482",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5285,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7212,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 11490,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4373547,
            "range": "± 26763",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 35414820,
            "range": "± 418011",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 37780300,
            "range": "± 3332596",
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
          "id": "cba0ce38f6cb6d18f5d4579681bfaecc1c4887b6",
          "message": "Added test case exercising CSV custom delimiter.",
          "timestamp": "2026-01-08T23:08:46+01:00",
          "tree_id": "e3898c2d35c80eab37980da303c07e555be61b5a",
          "url": "https://github.com/xkikeg/okane/commit/cba0ce38f6cb6d18f5d4579681bfaecc1c4887b6"
        },
        "date": 1767910622226,
        "tool": "cargo",
        "benches": [
          {
            "name": "parse plain",
            "value": 24,
            "range": "± 1",
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
            "value": 84,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 85,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 127343038,
            "range": "± 1687188",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 14784714,
            "range": "± 214249",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 120571312,
            "range": "± 1947876",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 122133964,
            "range": "± 1016933",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 23200607,
            "range": "± 475905",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 184632911,
            "range": "± 2017053",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 195897797,
            "range": "± 1401358",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2973499,
            "range": "± 39103",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 12472,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1456097,
            "range": "± 35782",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1586242,
            "range": "± 39212",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5209,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7139,
            "range": "± 59",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 11446,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4380795,
            "range": "± 44120",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 35732540,
            "range": "± 645538",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 38126236,
            "range": "± 428912",
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
          "id": "370587923e27d55f0be87e6c75d72bb13d8c243c",
          "message": "Added `template: true` to allow declaring a config just a template.\n\nIf user adds catch-all blanket config, it's problematic as\nall files would have essentially matched config and led some error\nmissing fields.\nTo avoid that, introduced `template` field so to declare a config isn't\na leaf config, and thus needs to be merged later.\n\nSome alternative considered:\n\n1. Considered `leaf: true`, but `template` must be rare and explicit.\n1. Considered not allowing template back to leaf.\nThis might be indeed good as we're currently ordering the config merge\norder depending on the matched path length and the last one must be\nnon-template, others must be template.\nHowever, eventually I might introduce more complicated (like mixin).\n(Of course, mixin can be `mixin: true` instead of template...)",
          "timestamp": "2026-01-08T23:29:20+01:00",
          "tree_id": "887a06e08eff536c735b7229a8eab220c82e4f35",
          "url": "https://github.com/xkikeg/okane/commit/370587923e27d55f0be87e6c75d72bb13d8c243c"
        },
        "date": 1767911860915,
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
            "value": 85,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 125805366,
            "range": "± 890440",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 14559466,
            "range": "± 75184",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 118697995,
            "range": "± 947681",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 120262294,
            "range": "± 1275999",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 23237744,
            "range": "± 261418",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 185638028,
            "range": "± 1405049",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 195972512,
            "range": "± 1489955",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2945037,
            "range": "± 11197",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 12462,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1434679,
            "range": "± 14449",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1566654,
            "range": "± 6972",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5229,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7060,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 11412,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4368894,
            "range": "± 28240",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 35261343,
            "range": "± 208628",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 37585313,
            "range": "± 336240",
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
          "id": "aab8632a672c3316ba8fce188922e10330793909",
          "message": "Use String instead of &'static str for ImportError::InvalidConfig.\n\nThis is more flexible, and nobody cares about the regression for just\none String allocation on error path.",
          "timestamp": "2026-01-09T17:12:19+01:00",
          "tree_id": "780dc65280ca5c87fa062859ee744115617a9fbb",
          "url": "https://github.com/xkikeg/okane/commit/aab8632a672c3316ba8fce188922e10330793909"
        },
        "date": 1767975619550,
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 81,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 127365937,
            "range": "± 1115309",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 14751392,
            "range": "± 187245",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 118941470,
            "range": "± 1253732",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 121670871,
            "range": "± 3019429",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 23888101,
            "range": "± 141353",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 190910421,
            "range": "± 822757",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 200927289,
            "range": "± 1678287",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2948403,
            "range": "± 19126",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 12462,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1435146,
            "range": "± 34617",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1569869,
            "range": "± 13701",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5173,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7073,
            "range": "± 54",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 11470,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4353731,
            "range": "± 43345",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 38192989,
            "range": "± 318849",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 40811589,
            "range": "± 467392",
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
          "id": "c8dcd42b64e0ea9002b9c4a5a2ff5b8d189a7475",
          "message": "Fixed tsv import.\n\nIt was wrongly referred delimiter from the config\neven though it's provided by the program code.",
          "timestamp": "2026-01-09T17:12:31+01:00",
          "tree_id": "644b3d56731dce4a03ef344afe1c74272c618164",
          "url": "https://github.com/xkikeg/okane/commit/c8dcd42b64e0ea9002b9c4a5a2ff5b8d189a7475"
        },
        "date": 1767975660633,
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
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 81,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 129252217,
            "range": "± 1476482",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 14774204,
            "range": "± 339217",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 120121537,
            "range": "± 1655579",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 121556386,
            "range": "± 2202972",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 23241915,
            "range": "± 152894",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 185499064,
            "range": "± 3443662",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 196185063,
            "range": "± 1749026",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2961418,
            "range": "± 18091",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 12455,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1445563,
            "range": "± 7379",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1583417,
            "range": "± 13512",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5305,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7153,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 11551,
            "range": "± 235",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4472708,
            "range": "± 61121",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 36120017,
            "range": "± 750334",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 38368452,
            "range": "± 485533",
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
          "id": "2dc259ee00310caefe6f2ce24538b5f31bdf270a",
          "message": "Allow specifying file_type in the config.\n\nImplements https://github.com/xkikeg/okane/issues/319.\n\nPrecisely, this PR **forces** to set file_type for ISO Camt053 and\nViseca format as well.",
          "timestamp": "2026-01-09T17:40:07+01:00",
          "tree_id": "55984356910c53fa99856a605a1f4b1cd71436d6",
          "url": "https://github.com/xkikeg/okane/commit/2dc259ee00310caefe6f2ce24538b5f31bdf270a"
        },
        "date": 1767977282704,
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 87,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 126243494,
            "range": "± 1067074",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 14723054,
            "range": "± 118621",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 120522616,
            "range": "± 1890772",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 122246834,
            "range": "± 776116",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 23149253,
            "range": "± 164619",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 184525653,
            "range": "± 2738671",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 195571873,
            "range": "± 1402715",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2961299,
            "range": "± 60718",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 12469,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1438837,
            "range": "± 6468",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1570785,
            "range": "± 23756",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5244,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7116,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 11510,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4362008,
            "range": "± 95903",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 35216629,
            "range": "± 289329",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 37515084,
            "range": "± 231690",
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
          "id": "a1fe7bba65411095ad0b0c775c64b0c5ebd32d68",
          "message": "Added changelog since 0.17.0 release.",
          "timestamp": "2026-01-09T17:48:33+01:00",
          "tree_id": "e79e2490035ee9276e29a34774d391b6cf8df9e7",
          "url": "https://github.com/xkikeg/okane/commit/a1fe7bba65411095ad0b0c775c64b0c5ebd32d68"
        },
        "date": 1767977797699,
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
            "value": 85,
            "range": "± 1",
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
            "value": 129447880,
            "range": "± 2785550",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 14947405,
            "range": "± 414001",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 122195411,
            "range": "± 2087066",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 123276086,
            "range": "± 1565685",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 23479415,
            "range": "± 315587",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 186913794,
            "range": "± 2213791",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 197582754,
            "range": "± 1468363",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2946954,
            "range": "± 19895",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 12465,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1437309,
            "range": "± 52719",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1566889,
            "range": "± 16845",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5246,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7281,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 11694,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4372578,
            "range": "± 26754",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 35172935,
            "range": "± 211962",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 37278084,
            "range": "± 647377",
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
          "id": "60579e62ff2d72bd1f52ecf1f88114e8df0dcce4",
          "message": "Corrected a cargo clippy warning.",
          "timestamp": "2026-01-10T22:09:25+01:00",
          "tree_id": "90dd9b1df996f0162d97b3d6c784ea7473126ef7",
          "url": "https://github.com/xkikeg/okane/commit/60579e62ff2d72bd1f52ecf1f88114e8df0dcce4"
        },
        "date": 1768079853841,
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
            "value": 85,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 133213043,
            "range": "± 4858130",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 15490201,
            "range": "± 961288",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 123360119,
            "range": "± 3060751",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 125045697,
            "range": "± 3165998",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 23996145,
            "range": "± 674991",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 192142641,
            "range": "± 3320628",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 206454113,
            "range": "± 5009158",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3013988,
            "range": "± 110792",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 12964,
            "range": "± 844",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1486900,
            "range": "± 70598",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1622912,
            "range": "± 60933",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5436,
            "range": "± 246",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7358,
            "range": "± 279",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 11510,
            "range": "± 365",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4456729,
            "range": "± 139428",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 36290363,
            "range": "± 1453440",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 39624984,
            "range": "± 2198937",
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
          "id": "282737d93456455b9b7aa259708af063d84e3805",
          "message": "Move okane.rs into main.rs.\n\nThis enables us to make all lib modules private.",
          "timestamp": "2026-01-10T22:18:05+01:00",
          "tree_id": "b44147d36f45de532b36f5df535c9a4046e3e0e7",
          "url": "https://github.com/xkikeg/okane/commit/282737d93456455b9b7aa259708af063d84e3805"
        },
        "date": 1768080365592,
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
            "range": "± 2",
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
            "value": 125759553,
            "range": "± 1646693",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 14450811,
            "range": "± 143161",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 118796057,
            "range": "± 1808948",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 118780852,
            "range": "± 1486312",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 23204257,
            "range": "± 142018",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 186052972,
            "range": "± 1137137",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 195616606,
            "range": "± 1203690",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2970344,
            "range": "± 11806",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 12453,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1437256,
            "range": "± 20105",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1566156,
            "range": "± 13581",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5229,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7149,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 11392,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4351844,
            "range": "± 44515",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 35237858,
            "range": "± 222188",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 37100119,
            "range": "± 296973",
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
          "id": "8fe8c2bc18db2490c7501d69c0d06d48d1b66cdd",
          "message": "Added multi_commodity balance integration test.\n\nThis was disabled because the Amount display was non-deterministic.\nNow it's deterministic and ok to have tests.",
          "timestamp": "2026-01-10T22:36:46+01:00",
          "tree_id": "9debd3a464c575b0fadaae70778f624033aced49",
          "url": "https://github.com/xkikeg/okane/commit/8fe8c2bc18db2490c7501d69c0d06d48d1b66cdd"
        },
        "date": 1768081476636,
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
            "range": "± 0",
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
            "value": 126211688,
            "range": "± 1902449",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 14619161,
            "range": "± 140247",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 118808324,
            "range": "± 1167345",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 120228803,
            "range": "± 2510392",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 23253276,
            "range": "± 464320",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 186486790,
            "range": "± 4197745",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 195447624,
            "range": "± 5703440",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2958550,
            "range": "± 10893",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 12468,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1435127,
            "range": "± 6807",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1574400,
            "range": "± 13094",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5369,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7240,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 11778,
            "range": "± 169",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4357667,
            "range": "± 25026",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 35371204,
            "range": "± 257879",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 37423746,
            "range": "± 455131",
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
          "id": "58ec666d9ffc5a5f82d85ed5576b59279c34cdd8",
          "message": "Improved Amount printing on negative amount.\n\nBefore: `(1 USD + -2 CHF)`\nAfter: `(1 USD - 2 CHF)`",
          "timestamp": "2026-01-10T23:56:05+01:00",
          "tree_id": "ac44b82b05b94267d685be35ba911b1e071195c6",
          "url": "https://github.com/xkikeg/okane/commit/58ec666d9ffc5a5f82d85ed5576b59279c34cdd8"
        },
        "date": 1768086251624,
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
            "value": 25,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string plain",
            "value": 86,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 85,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 131255533,
            "range": "± 1662285",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 15622189,
            "range": "± 214855",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 125425040,
            "range": "± 959254",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 127194530,
            "range": "± 849063",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 23875831,
            "range": "± 152484",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 190385134,
            "range": "± 932974",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 200358931,
            "range": "± 1491436",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2936187,
            "range": "± 15545",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 12459,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1435618,
            "range": "± 28785",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1578032,
            "range": "± 21829",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5178,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7016,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 11509,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4394167,
            "range": "± 32695",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 35526294,
            "range": "± 465028",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 38110872,
            "range": "± 230934",
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
          "id": "972dbf2b92cdcd7127126feeee0ad087d7dbe7b8",
          "message": "Use OneBased for Field::ColumnIndex.\n\nAlso uses OneBased v1.",
          "timestamp": "2026-01-13T20:23:32+01:00",
          "tree_id": "863be2a35c593ab87b700e88e213a3c72796a9b2",
          "url": "https://github.com/xkikeg/okane/commit/972dbf2b92cdcd7127126feeee0ad087d7dbe7b8"
        },
        "date": 1768332690139,
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
            "value": 87,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 85,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 125147338,
            "range": "± 719228",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 14612626,
            "range": "± 138473",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 118922154,
            "range": "± 1605780",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 120675173,
            "range": "± 1291309",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 23357332,
            "range": "± 108760",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 187185176,
            "range": "± 812642",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 197265428,
            "range": "± 1897881",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 2954315,
            "range": "± 14025",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 12486,
            "range": "± 241",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1464761,
            "range": "± 9733",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1602173,
            "range": "± 11959",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5281,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7124,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 11503,
            "range": "± 62",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4373431,
            "range": "± 27079",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 35446135,
            "range": "± 311476",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 37725413,
            "range": "± 339091",
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
          "id": "6b1975967d623c2028fbff73cbd6c1cbfd8af934",
          "message": "Reordered Amount / SingleAmount / PostingAmount from_value(s)? args.\n\nI found Amount::into_values().into_iter() gives the iterator of\n(commodity, value), not the (value, commodity).\nThen, from_values should take the iterator in this order instead for\nmake API symmetrical. For the alignment, from_value should have the same\norder.",
          "timestamp": "2026-01-15T21:53:15+01:00",
          "tree_id": "6610ce4b8100a46d6a3e5cbbfa90874ce8258913",
          "url": "https://github.com/xkikeg/okane/commit/6b1975967d623c2028fbff73cbd6c1cbfd8af934"
        },
        "date": 1768510870344,
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
            "value": 85,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 81,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 126627021,
            "range": "± 852551",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 14692063,
            "range": "± 242500",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 118676497,
            "range": "± 1473867",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 121297811,
            "range": "± 1422976",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 23451404,
            "range": "± 293286",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 187354988,
            "range": "± 1016750",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 197197638,
            "range": "± 2032352",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3029981,
            "range": "± 8493",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 12585,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1459662,
            "range": "± 19926",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1592859,
            "range": "± 25339",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5177,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7117,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 11257,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4342536,
            "range": "± 22585",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 35270774,
            "range": "± 370540",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 37894575,
            "range": "± 299559",
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
          "id": "979f952c35025f0a031309e417b9e6737f5a802e",
          "message": "Refactored the ImportError.\n\n* Now ImportError is a simple kind + message + source struct.\n* Removed ImportError::Other that was just escape hatch.",
          "timestamp": "2026-01-16T21:04:01+01:00",
          "tree_id": "24f2020b24457a7bee97c72596c649080871a5b4",
          "url": "https://github.com/xkikeg/okane/commit/979f952c35025f0a031309e417b9e6737f5a802e"
        },
        "date": 1768594329247,
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 81,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 126381576,
            "range": "± 411811",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 15087710,
            "range": "± 365416",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 120543177,
            "range": "± 1843052",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 121921799,
            "range": "± 2153283",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 23136243,
            "range": "± 247521",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 185814508,
            "range": "± 2137599",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 196385194,
            "range": "± 1677034",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3016234,
            "range": "± 8494",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 12456,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1435439,
            "range": "± 38452",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1571035,
            "range": "± 9360",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 5246,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 7278,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 11374,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4329063,
            "range": "± 19404",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 35974632,
            "range": "± 1363886",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 40924545,
            "range": "± 977936",
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
          "id": "a2ede08b40b7bff460dbe8189716a955a6bac7fc",
          "message": "Simplified Viseca LineReader interface.\n\nIt used to refer its internal state too often.\nCoupled line information into LineReader results.",
          "timestamp": "2026-01-23T09:18:21+01:00",
          "tree_id": "7fff1fd476088d0a5493cbda5186a3c09c6be1fa",
          "url": "https://github.com/xkikeg/okane/commit/a2ede08b40b7bff460dbe8189716a955a6bac7fc"
        },
        "date": 1769156830953,
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 81,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 138760658,
            "range": "± 1166867",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 16311503,
            "range": "± 100913",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 132988824,
            "range": "± 1130387",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 133167102,
            "range": "± 1075779",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 25832008,
            "range": "± 538778",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 206284194,
            "range": "± 2845739",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 219187095,
            "range": "± 4187333",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3136715,
            "range": "± 20615",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 12464,
            "range": "± 868",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1194415,
            "range": "± 15492",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1317986,
            "range": "± 20249",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 4766,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 6505,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 10786,
            "range": "± 757",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4062402,
            "range": "± 20107",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 32796630,
            "range": "± 383171",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 35545601,
            "range": "± 476828",
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
          "id": "e171b2c7ed9217688aedf8ae551e6a560294a3b5",
          "message": "Supported viseca transaction line with missing category.\n\nThis PR will fix https://github.com/xkikeg/okane/issues/340.",
          "timestamp": "2026-01-25T12:38:34+01:00",
          "tree_id": "93ddae352d2333614bf03c641bf5b64c0da9f146",
          "url": "https://github.com/xkikeg/okane/commit/e171b2c7ed9217688aedf8ae551e6a560294a3b5"
        },
        "date": 1769341612983,
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string plain",
            "value": 91,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 82,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 146834478,
            "range": "± 1535474",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 17225774,
            "range": "± 150101",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 139874966,
            "range": "± 1399934",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 140916537,
            "range": "± 1168412",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 27961328,
            "range": "± 275771",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 223607498,
            "range": "± 2609854",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 236076397,
            "range": "± 2734185",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3369082,
            "range": "± 6487",
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
            "value": 11497,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1266835,
            "range": "± 17187",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1364888,
            "range": "± 14896",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 4938,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 6667,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 10772,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4117858,
            "range": "± 9608",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 33958626,
            "range": "± 163402",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 37416434,
            "range": "± 215718",
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
          "id": "432bfa0b8e664025d2fa0e1870cb619fb892c9f5",
          "message": "Added Zlib as allowed license.\n\nThis is causing cargo deny failure.",
          "timestamp": "2026-01-26T16:42:27+01:00",
          "tree_id": "8e9e295e4a895134c2f080c4f465fce59ca1d68e",
          "url": "https://github.com/xkikeg/okane/commit/432bfa0b8e664025d2fa0e1870cb619fb892c9f5"
        },
        "date": 1769442642025,
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 81,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 137015071,
            "range": "± 1033941",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 16100089,
            "range": "± 120316",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 129092596,
            "range": "± 1097839",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 131015431,
            "range": "± 1005697",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 24640178,
            "range": "± 352004",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 197660882,
            "range": "± 2471972",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 207844186,
            "range": "± 1016684",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3099226,
            "range": "± 12533",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 12452,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1202383,
            "range": "± 52637",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1337295,
            "range": "± 54895",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 4764,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 6551,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 10737,
            "range": "± 43",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4064366,
            "range": "± 32310",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 32557691,
            "range": "± 239643",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 34817460,
            "range": "± 295717",
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
          "id": "900594cd1721e26e422184253186c2f41796c707",
          "message": "Corrected clippy warnings.",
          "timestamp": "2026-01-26T17:52:26+01:00",
          "tree_id": "5b03eb709f105679e26de319261669a467dd209a",
          "url": "https://github.com/xkikeg/okane/commit/900594cd1721e26e422184253186c2f41796c707"
        },
        "date": 1769446840659,
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
            "value": 84,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 80,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 139854354,
            "range": "± 2590314",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 16159735,
            "range": "± 375454",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 134657444,
            "range": "± 3037491",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 137548108,
            "range": "± 2440819",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 25029501,
            "range": "± 538927",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 200883073,
            "range": "± 2744925",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 213790185,
            "range": "± 2278954",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3118503,
            "range": "± 33318",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 12337,
            "range": "± 268",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1189213,
            "range": "± 26683",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1324965,
            "range": "± 24298",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 4819,
            "range": "± 115",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 6681,
            "range": "± 157",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 10887,
            "range": "± 234",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4026885,
            "range": "± 45128",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 32486783,
            "range": "± 382425",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 34163808,
            "range": "± 514831",
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
          "id": "58ef7925c8511e9332419bd9ab7fcf98f3857f3b",
          "message": "Fixed Balance::get to accept Account, not &Account.\n\nAccount is essentially &str, quite lightweight to copy.",
          "timestamp": "2026-01-29T13:04:39+01:00",
          "tree_id": "38566a157f23cc2b2263dfb1155db3f5ed44b40a",
          "url": "https://github.com/xkikeg/okane/commit/58ef7925c8511e9332419bd9ab7fcf98f3857f3b"
        },
        "date": 1769688773533,
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "to_string comma",
            "value": 82,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-file/middle_10y16a500t",
            "value": 141396048,
            "range": "± 1393810",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/small_5y10a200t",
            "value": 16587006,
            "range": "± 86915",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_10y16a500t",
            "value": 135127416,
            "range": "± 1008216",
            "unit": "ns/iter"
          },
          {
            "name": "load/on-memory/middle_more_commodity_10y16a500t",
            "value": 135713382,
            "range": "± 1788716",
            "unit": "ns/iter"
          },
          {
            "name": "process/small_5y10a200t",
            "value": 25325380,
            "range": "± 1030765",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_10y16a500t",
            "value": 202477871,
            "range": "± 1278337",
            "unit": "ns/iter"
          },
          {
            "name": "process/middle_more_commodity_10y16a500t",
            "value": 214148626,
            "range": "± 1377338",
            "unit": "ns/iter"
          },
          {
            "name": "query-posting-one-account",
            "value": 3100872,
            "range": "± 10235",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/small_5y10a200t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/default/middle_more_commodity_10y16a500t",
            "value": 12,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/small_5y10a200t",
            "value": 12832,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_10y16a500t",
            "value": 1196657,
            "range": "± 17451",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/date-range/middle_more_commodity_10y16a500t",
            "value": 1320535,
            "range": "± 9695",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/small_5y10a200t",
            "value": 4831,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_10y16a500t",
            "value": 6598,
            "range": "± 125",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-up-to-date/middle_more_commodity_10y16a500t",
            "value": 10916,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/small_5y10a200t",
            "value": 4051770,
            "range": "± 22699",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_10y16a500t",
            "value": 34351853,
            "range": "± 539500",
            "unit": "ns/iter"
          },
          {
            "name": "query::balance/conversion-historical/middle_more_commodity_10y16a500t",
            "value": 36525488,
            "range": "± 719824",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}