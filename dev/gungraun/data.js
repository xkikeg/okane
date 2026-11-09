window.BENCHMARK_DATA = {
  "lastUpdate": 1794213586485,
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
      }
    ]
  }
}