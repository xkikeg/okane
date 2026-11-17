window.BENCHMARK_DATA = {
  "lastUpdate": 1794906325238,
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
      }
    ]
  }
}