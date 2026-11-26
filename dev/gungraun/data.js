window.BENCHMARK_DATA = {
  "lastUpdate": 1795655892835,
  "repoUrl": "https://github.com/xkikeg/okane",
  "entries": {
    "Gungraun Benchmark": [
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
          "id": "e00b5a3ba1579806e35eed73f52782ae12f28dd4",
          "message": "bench: add gungraun instruction-count benchmark\n\nWall-time benches on GitHub-hosted runners mostly measure which CPU the\njob landed on. Add report_gungraun, which counts instructions with\nValgrind's Callgrind for the same report operations on the small\ndataset, and run it in a separate workflow.\n\nThe workflow only reports for now: on PRs it compares against the base\ncommit and writes the result to the job summary, without failing. The\nexisting wall-time job is unchanged, so we can compare how noisy each one\nis before deciding what PRs should fail on.\n\nResults also go to github-action-benchmark: on main they are stored in\ngh-pages under dev/gungraun, separate from the wall-time chart; on PRs\nthe job summary compares them against the latest main result.\n\nLocally, two runs differ by at most 0.03% per case, except the ~49k\ninstruction conversion_up_to_date balance case at 0.5%.\n\nRefs #625.\n\nCo-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>",
          "timestamp": "2026-11-09T09:38:08+01:00",
          "tree_id": "016776c88a0defd1a385ed89cc49ce7e7e0c667f",
          "url": "https://github.com/xkikeg/okane/commit/e00b5a3ba1579806e35eed73f52782ae12f28dd4"
        },
        "date": 1794213584833,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_process::load_on_memory/small",
            "value": 120644589,
            "unit": "instructions"
          },
          {
            "name": "load_process::process/small",
            "value": 174122163,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical",
            "value": 37590885,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical_pricedb",
            "value": 35527440,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_up_to_date",
            "value": 50297,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/date_range_middle20",
            "value": 11031907,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/account_filter",
            "value": 758050,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical",
            "value": 28338788,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical_pricedb",
            "value": 30473729,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/date_range_middle20",
            "value": 1184216,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/default",
            "value": 4838802,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_cold",
            "value": 12873790,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_warm",
            "value": 4758877,
            "unit": "instructions"
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
          "id": "39dc968ddeb2cfa4c7cdeb83b8cc0288a28deb7a",
          "message": "Use lz4 package instead of liblz4-tool.\n\nhttps://packages.ubuntu.com/noble/liblz4-tool\nThe package was already transitional in 24.04,\nand 26.04 doesn't support it anymore.",
          "timestamp": "2026-11-12T10:50:34+01:00",
          "tree_id": "317e0cdf6e845bbcfcfcbaaeb7fb954ee09a600b",
          "url": "https://github.com/xkikeg/okane/commit/39dc968ddeb2cfa4c7cdeb83b8cc0288a28deb7a"
        },
        "date": 1794477147960,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_process::load_on_memory/small",
            "value": 119245691,
            "unit": "instructions"
          },
          {
            "name": "load_process::process/small",
            "value": 169709217,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical",
            "value": 36176244,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical_pricedb",
            "value": 34110515,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_up_to_date",
            "value": 47540,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/date_range_middle20",
            "value": 10422187,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/account_filter",
            "value": 757983,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical",
            "value": 26913670,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical_pricedb",
            "value": 29055944,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/date_range_middle20",
            "value": 1184149,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/default",
            "value": 4838735,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_cold",
            "value": 12531776,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_warm",
            "value": 4758810,
            "unit": "instructions"
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
          "id": "0c148ac1046e68dfa85e26238e8dfafff9070cd6",
          "message": "Create MergeOwned trait for structs.\n\nWe used to implement Merge directly, with implementing\nmerge_from with clone().\nBut for T: Default, we can use mem::take() which is cheaper.\nSo we split MergeOwned for these types,\nand provided a macro to implement Merge for MergeOwned types.\nAvoided the blanket implementation, but it might be possible\nto adopt it instead later.",
          "timestamp": "2026-11-12T11:19:10+01:00",
          "tree_id": "4c39a5b9aad1dde86d61bcb0c8d077b549e3adb5",
          "url": "https://github.com/xkikeg/okane/commit/0c148ac1046e68dfa85e26238e8dfafff9070cd6"
        },
        "date": 1794478859691,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_process::load_on_memory/small",
            "value": 119250962,
            "unit": "instructions"
          },
          {
            "name": "load_process::process/small",
            "value": 169693696,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical",
            "value": 36184164,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical_pricedb",
            "value": 34108598,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_up_to_date",
            "value": 47312,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/date_range_middle20",
            "value": 10422329,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/account_filter",
            "value": 757983,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical",
            "value": 26918417,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical_pricedb",
            "value": 29051913,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/date_range_middle20",
            "value": 1184149,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/default",
            "value": 4838735,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_cold",
            "value": 12532261,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_warm",
            "value": 4758810,
            "unit": "instructions"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "kikeg@kikeg.com",
            "name": "kikeg",
            "username": "xkikeg"
          },
          "distinct": true,
          "id": "3ea94ec2f080ddba7251783b57485feaad8b05bf",
          "message": "build(deps): bump taiki-e/install-action from 2.89.0 to 2.90.3\n\nBumps [taiki-e/install-action](https://github.com/taiki-e/install-action) from 2.89.0 to 2.90.3.\n- [Release notes](https://github.com/taiki-e/install-action/releases)\n- [Changelog](https://github.com/taiki-e/install-action/blob/main/CHANGELOG.md)\n- [Commits](https://github.com/taiki-e/install-action/compare/1919c4a95903d2c33e4a221102edca87309fc14b...cfdb7ed4ccc7c7dce37759eae89dc5674b469b39)\n\n---\nupdated-dependencies:\n- dependency-name: taiki-e/install-action\n  dependency-version: 2.90.3\n  dependency-type: direct:production\n  update-type: version-update:semver-minor\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>",
          "timestamp": "2026-11-13T08:22:08+01:00",
          "tree_id": "790582e79b6836da19c88acefcc68879e2babecb",
          "url": "https://github.com/xkikeg/okane/commit/3ea94ec2f080ddba7251783b57485feaad8b05bf"
        },
        "date": 1794554671007,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_process::load_on_memory/small",
            "value": 119223990,
            "unit": "instructions"
          },
          {
            "name": "load_process::process/small",
            "value": 169665248,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical",
            "value": 36221458,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical_pricedb",
            "value": 34151182,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_up_to_date",
            "value": 47039,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/date_range_middle20",
            "value": 10422153,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/account_filter",
            "value": 757983,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical",
            "value": 26967439,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical_pricedb",
            "value": 29097420,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/date_range_middle20",
            "value": 1184149,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/default",
            "value": 4838735,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_cold",
            "value": 12531612,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_warm",
            "value": 4838812,
            "unit": "instructions"
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
          "id": "e844e336c3dc082365d240a6445d710ce64cf5f6",
          "message": "parse: accept prefix commodity only through a named function\n\n`Amount::try_from(&str)` accepted the commodity on either side, though\nLedger files only take it after the value. Its only caller relying on\nthe prefix form was the CSV importer, which reads values such as\n`$6.96` and drops the commodity.\n\nMake `TryFrom` take the posting syntax only, and expose the old\nbehaviour as `parse::amount_with_prefix_commodity` for the importer.\n\nCo-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>",
          "timestamp": "2026-11-17T07:38:37+01:00",
          "tree_id": "d127a1784d5ceb3e28169f8ae5196f3843c58da6",
          "url": "https://github.com/xkikeg/okane/commit/e844e336c3dc082365d240a6445d710ce64cf5f6"
        },
        "date": 1794897641909,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_process::load_on_memory/small",
            "value": 119224413,
            "unit": "instructions"
          },
          {
            "name": "load_process::process/small",
            "value": 169658713,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical",
            "value": 36225500,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical_pricedb",
            "value": 34154917,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_up_to_date",
            "value": 47065,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/date_range_middle20",
            "value": 10422620,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/account_filter",
            "value": 757983,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical",
            "value": 26965183,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical_pricedb",
            "value": 29099199,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/date_range_middle20",
            "value": 1184149,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/default",
            "value": 4838735,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_cold",
            "value": 12531859,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_warm",
            "value": 4838812,
            "unit": "instructions"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "kikeg@kikeg.com",
            "name": "kikeg",
            "username": "xkikeg"
          },
          "distinct": true,
          "id": "a0b691a0fd5c2eedc51d2639ad0157c31be180fb",
          "message": "build(deps): bump softprops/action-gh-release from 3.0.3 to 3.0.4\n\nBumps [softprops/action-gh-release](https://github.com/softprops/action-gh-release) from 3.0.3 to 3.0.4.\n- [Release notes](https://github.com/softprops/action-gh-release/releases)\n- [Changelog](https://github.com/softprops/action-gh-release/blob/master/CHANGELOG.md)\n- [Commits](https://github.com/softprops/action-gh-release/compare/efb35369e0ad2afab669f228072c1b0d510eae64...8a0cd651e5b2365bd640854a3fa5dfc70e4875f3)\n\n---\nupdated-dependencies:\n- dependency-name: softprops/action-gh-release\n  dependency-version: 3.0.4\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>",
          "timestamp": "2026-11-17T07:45:08+01:00",
          "tree_id": "a3df601fae2cd5f973832842b3e987cd92678e1c",
          "url": "https://github.com/xkikeg/okane/commit/a0b691a0fd5c2eedc51d2639ad0157c31be180fb"
        },
        "date": 1794897995291,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_process::load_on_memory/small",
            "value": 119225557,
            "unit": "instructions"
          },
          {
            "name": "load_process::process/small",
            "value": 169687434,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical",
            "value": 36230544,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical_pricedb",
            "value": 34150812,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_up_to_date",
            "value": 46839,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/date_range_middle20",
            "value": 10422114,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/account_filter",
            "value": 757983,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical",
            "value": 26967473,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical_pricedb",
            "value": 29102832,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/date_range_middle20",
            "value": 1184149,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/default",
            "value": 4838735,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_cold",
            "value": 12531926,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_warm",
            "value": 4838812,
            "unit": "instructions"
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
          "id": "acbdf29bb2b51726f1e4bfd8790782eb797600f8",
          "message": "Bump pretty_decimal to 0.3\n\nCo-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>",
          "timestamp": "2026-11-17T10:03:38+01:00",
          "tree_id": "aad2e0dfed6700ef90810414f3702d4998ae7c80",
          "url": "https://github.com/xkikeg/okane/commit/acbdf29bb2b51726f1e4bfd8790782eb797600f8"
        },
        "date": 1794906323842,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_process::load_on_memory/small",
            "value": 119737733,
            "unit": "instructions"
          },
          {
            "name": "load_process::process/small",
            "value": 170176051,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical",
            "value": 36227456,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical_pricedb",
            "value": 34174237,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_up_to_date",
            "value": 46751,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/date_range_middle20",
            "value": 10421548,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/account_filter",
            "value": 757983,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical",
            "value": 26967091,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical_pricedb",
            "value": 29101550,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/date_range_middle20",
            "value": 1184149,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/default",
            "value": 4838735,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_cold",
            "value": 12531501,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_warm",
            "value": 4838812,
            "unit": "instructions"
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
          "id": "17a99c4ae67a6dd3d7d3c5cc9e012045abc84523",
          "message": "parse: fix hang on a parse error at the end of input\n\nParseError::new looked for the end of the error span with an unbounded\n`(offset + 1..)` range. `is_char_boundary` is false past the end of\nthe string, so an error at the end of the input, such as\n`ValueExpr::try_from(\"\")`, walked up usize forever.\n\nBound the range by the input length, so the span is empty at the end.\n\nCo-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>",
          "timestamp": "2026-11-17T10:05:33+01:00",
          "tree_id": "46667932cacbbd9774ed0e65d0b4ece2ceee950f",
          "url": "https://github.com/xkikeg/okane/commit/17a99c4ae67a6dd3d7d3c5cc9e012045abc84523"
        },
        "date": 1794906446701,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_process::load_on_memory/small",
            "value": 119735860,
            "unit": "instructions"
          },
          {
            "name": "load_process::process/small",
            "value": 170217069,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical",
            "value": 36238096,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical_pricedb",
            "value": 34150930,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_up_to_date",
            "value": 47068,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/date_range_middle20",
            "value": 10422055,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/account_filter",
            "value": 757983,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical",
            "value": 26966516,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical_pricedb",
            "value": 29102836,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/date_range_middle20",
            "value": 1184149,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/default",
            "value": 4838735,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_cold",
            "value": 12531721,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_warm",
            "value": 4838812,
            "unit": "instructions"
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
          "id": "512c28503fe99e5f410a2392e15ffa00d48320bf",
          "message": "Make --sort=date default, insteaad of --sort=original.\n\n--sort=original is useful only when user want to track down per-file\ninconsistency.\nOTOH --sort=date should be the default choice given most of the users \nbenefit from the option.\n\nThis is breaking change.",
          "timestamp": "2026-11-17T12:50:54+01:00",
          "tree_id": "c499425bf83615bf66b7bfa631c382e6913168a7",
          "url": "https://github.com/xkikeg/okane/commit/512c28503fe99e5f410a2392e15ffa00d48320bf"
        },
        "date": 1794916385510,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_process::load_on_memory/small",
            "value": 119735196,
            "unit": "instructions"
          },
          {
            "name": "load_process::process/small",
            "value": 170203321,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical",
            "value": 36229145,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical_pricedb",
            "value": 34159273,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_up_to_date",
            "value": 47045,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/date_range_middle20",
            "value": 10427866,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/account_filter",
            "value": 757983,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical",
            "value": 26961204,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical_pricedb",
            "value": 29102689,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/date_range_middle20",
            "value": 1184149,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/default",
            "value": 4838735,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_cold",
            "value": 12532589,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_warm",
            "value": 4838812,
            "unit": "instructions"
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
          "id": "d9a84a1ff13676cc011b753001308dd9525737e7",
          "message": "chore: cargo update\n\nCo-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>",
          "timestamp": "2026-11-17T14:08:55+01:00",
          "tree_id": "e95979281907b0c57d30427ee3987e326857fb56",
          "url": "https://github.com/xkikeg/okane/commit/d9a84a1ff13676cc011b753001308dd9525737e7"
        },
        "date": 1794921075548,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_process::load_on_memory/small",
            "value": 119735148,
            "unit": "instructions"
          },
          {
            "name": "load_process::process/small",
            "value": 170130172,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical",
            "value": 36226314,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical_pricedb",
            "value": 34151374,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_up_to_date",
            "value": 47145,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/date_range_middle20",
            "value": 10421669,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/account_filter",
            "value": 757983,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical",
            "value": 26967282,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical_pricedb",
            "value": 29097210,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/date_range_middle20",
            "value": 1184149,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/default",
            "value": 4838735,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_cold",
            "value": 12532116,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_warm",
            "value": 4838812,
            "unit": "instructions"
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
          "id": "c7b260b70a4dc0e037cf40b18d3c0becfb45efe6",
          "message": "ci: allow unmaintained chrono in cargo-deny\n\nRUSTSEC-2026-0543 marks chrono as unmaintained, which fails cargo-deny.\nIgnore it until chrono is replaced by jiff.\n\nCo-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>",
          "timestamp": "2026-11-17T20:27:44+01:00",
          "tree_id": "cafa5a1572a6c12e5ec36200e347e15379749013",
          "url": "https://github.com/xkikeg/okane/commit/c7b260b70a4dc0e037cf40b18d3c0becfb45efe6"
        },
        "date": 1794943806618,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_process::load_on_memory/small",
            "value": 119736168,
            "unit": "instructions"
          },
          {
            "name": "load_process::process/small",
            "value": 170126889,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical",
            "value": 36226784,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical_pricedb",
            "value": 34176459,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_up_to_date",
            "value": 46889,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/date_range_middle20",
            "value": 10423693,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/account_filter",
            "value": 757983,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical",
            "value": 26970143,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical_pricedb",
            "value": 29102000,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/date_range_middle20",
            "value": 1184149,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/default",
            "value": 4838735,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_cold",
            "value": 12531590,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_warm",
            "value": 4838812,
            "unit": "instructions"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "kikeg@kikeg.com",
            "name": "kikeg",
            "username": "xkikeg"
          },
          "distinct": true,
          "id": "707b907982880b8b177360da20aa84d679027ef2",
          "message": "build(deps): bump taiki-e/install-action from 2.90.3 to 2.90.5\n\nBumps [taiki-e/install-action](https://github.com/taiki-e/install-action) from 2.90.3 to 2.90.5.\n- [Release notes](https://github.com/taiki-e/install-action/releases)\n- [Changelog](https://github.com/taiki-e/install-action/blob/main/CHANGELOG.md)\n- [Commits](https://github.com/taiki-e/install-action/compare/cfdb7ed4ccc7c7dce37759eae89dc5674b469b39...ab49d3c64680143e184118bed66a46419fc15519)\n\n---\nupdated-dependencies:\n- dependency-name: taiki-e/install-action\n  dependency-version: 2.90.5\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>",
          "timestamp": "2026-11-18T11:12:34+01:00",
          "tree_id": "a89986dc0b51dd695b5927fa5f78af625ce25b6f",
          "url": "https://github.com/xkikeg/okane/commit/707b907982880b8b177360da20aa84d679027ef2"
        },
        "date": 1794997140929,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_process::load_on_memory/small",
            "value": 119734152,
            "unit": "instructions"
          },
          {
            "name": "load_process::process/small",
            "value": 170136795,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical",
            "value": 36226862,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical_pricedb",
            "value": 34185599,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_up_to_date",
            "value": 47020,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/date_range_middle20",
            "value": 10422567,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/account_filter",
            "value": 757983,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical",
            "value": 26967405,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical_pricedb",
            "value": 29100198,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/date_range_middle20",
            "value": 1184149,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/default",
            "value": 4838735,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_cold",
            "value": 12532009,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_warm",
            "value": 4838812,
            "unit": "instructions"
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
          "id": "851e63c78bbc9c438cbdc7eec491706eec09d3de",
          "message": "chore: Upgrade dependencies.",
          "timestamp": "2026-11-25T22:11:35+01:00",
          "tree_id": "9d3bb01ffb14243f7f68c0e553747dd4c3496714",
          "url": "https://github.com/xkikeg/okane/commit/851e63c78bbc9c438cbdc7eec491706eec09d3de"
        },
        "date": 1795641237370,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_process::load_on_memory/small",
            "value": 119734072,
            "unit": "instructions"
          },
          {
            "name": "load_process::process/small",
            "value": 170137488,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical",
            "value": 36240726,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical_pricedb",
            "value": 34154627,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_up_to_date",
            "value": 47052,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/date_range_middle20",
            "value": 10421702,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/account_filter",
            "value": 757983,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical",
            "value": 26958080,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical_pricedb",
            "value": 29100961,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/date_range_middle20",
            "value": 1184149,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/default",
            "value": 4838735,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_cold",
            "value": 12532350,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_warm",
            "value": 4838812,
            "unit": "instructions"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "kikeg@kikeg.com",
            "name": "kikeg",
            "username": "xkikeg"
          },
          "distinct": true,
          "id": "fb1370e1c1263f4839910e26cb1e36f94b4258b1",
          "message": "build(deps): bump EmbarkStudios/cargo-deny-action from 2.1.1 to 2.1.3\n\nBumps [EmbarkStudios/cargo-deny-action](https://github.com/embarkstudios/cargo-deny-action) from 2.1.1 to 2.1.3.\n- [Release notes](https://github.com/embarkstudios/cargo-deny-action/releases)\n- [Commits](https://github.com/embarkstudios/cargo-deny-action/compare/3c6349835b2b7b196a839186cb8b78e02f7b5f25...17aeb3c839e42274e2b8adf721c8c65edfdc74dd)\n\n---\nupdated-dependencies:\n- dependency-name: EmbarkStudios/cargo-deny-action\n  dependency-version: 2.1.3\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>",
          "timestamp": "2026-11-25T22:17:58+01:00",
          "tree_id": "2075ce2879c9d256e3fbdac459594b837ff6c8e1",
          "url": "https://github.com/xkikeg/okane/commit/fb1370e1c1263f4839910e26cb1e36f94b4258b1"
        },
        "date": 1795641570718,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_process::load_on_memory/small",
            "value": 119736084,
            "unit": "instructions"
          },
          {
            "name": "load_process::process/small",
            "value": 170126303,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical",
            "value": 36227496,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical_pricedb",
            "value": 34152118,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_up_to_date",
            "value": 47113,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/date_range_middle20",
            "value": 10422119,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/account_filter",
            "value": 757983,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical",
            "value": 26962515,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical_pricedb",
            "value": 29096005,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/date_range_middle20",
            "value": 1184149,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/default",
            "value": 4838735,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_cold",
            "value": 12531972,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_warm",
            "value": 4838812,
            "unit": "instructions"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "kikeg@kikeg.com",
            "name": "kikeg",
            "username": "xkikeg"
          },
          "distinct": true,
          "id": "d7abb01fa2041d8141e9a8e5ade1570730840bbf",
          "message": "build(deps): bump taiki-e/install-action from 2.90.5 to 2.90.8\n\nBumps [taiki-e/install-action](https://github.com/taiki-e/install-action) from 2.90.5 to 2.90.8.\n- [Release notes](https://github.com/taiki-e/install-action/releases)\n- [Changelog](https://github.com/taiki-e/install-action/blob/main/CHANGELOG.md)\n- [Commits](https://github.com/taiki-e/install-action/compare/ab49d3c64680143e184118bed66a46419fc15519...fe3acb53a98b62c772db35de75ce0e4595e7ca8c)\n\n---\nupdated-dependencies:\n- dependency-name: taiki-e/install-action\n  dependency-version: 2.90.8\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>",
          "timestamp": "2026-11-26T02:17:01+01:00",
          "tree_id": "13fd5a24d2bca932ecaf1e337d8e1522c745264f",
          "url": "https://github.com/xkikeg/okane/commit/d7abb01fa2041d8141e9a8e5ade1570730840bbf"
        },
        "date": 1795655891490,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_process::load_on_memory/small",
            "value": 119735956,
            "unit": "instructions"
          },
          {
            "name": "load_process::process/small",
            "value": 170127171,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical",
            "value": 36229626,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_historical_pricedb",
            "value": 34154677,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/conversion_up_to_date",
            "value": 46953,
            "unit": "instructions"
          },
          {
            "name": "query_balance::balance/date_range_middle20",
            "value": 10422310,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/account_filter",
            "value": 757983,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical",
            "value": 26960445,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/conversion_historical_pricedb",
            "value": 29097615,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/date_range_middle20",
            "value": 1184149,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/default",
            "value": 4838735,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_cold",
            "value": 12532093,
            "unit": "instructions"
          },
          {
            "name": "query_register::register/sort_date_warm",
            "value": 4838812,
            "unit": "instructions"
          }
        ]
      }
    ]
  }
}