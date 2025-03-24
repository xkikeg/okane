use std::path::Path;

use bumpalo::Bump;
use chrono::NaiveDate;
use criterion::measurement::Measurement as _;
use criterion::{
    black_box, criterion_group, criterion_main, measurement::WallTime, BatchSize, BenchmarkId,
    Criterion,
};
use log::LevelFilter;
use okane_core::{
    load::LoadError,
    report::{self},
    syntax,
};
use pretty_assertions::assert_eq;
use testing::{ExampleInput, FakeFileSink, FileSink, InputParams, RealFileSink};

pub mod testing;

fn load_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("load");
    group.warm_up_time(std::time::Duration::from_secs(7));

    let params = InputParams::middle();
    let input = RealFileSink::new_example(Path::new("report_bench"), params).unwrap();

    basic_asserts(&input);

    group.bench_with_input(
        BenchmarkId::new("on-file", params),
        &params,
        |b, _params| {
            b.iter(|| {
                let mut count = 0;
                input
                    .new_loader()
                    .load(|_, _, _: &syntax::tracked::LedgerEntry| {
                        count += 1;
                        Ok::<(), LoadError>(())
                    })
                    .unwrap();
                black_box(count)
            })
        },
    );

    for params in InputParams::params_from_env() {
        if let Some(samples) = params.sample_size {
            group.sample_size(samples);
        }
        group.bench_with_input(
            BenchmarkId::new("on-memory", params),
            &params,
            |b, params| {
                let input = FakeFileSink::new_example(Path::new("report_bench"), *params).unwrap();

                basic_asserts(&input);

                b.iter_batched(
                    || input.new_loader(),
                    |loader| {
                        let mut count = 0;
                        loader
                            .load(|_, _, _: &syntax::tracked::LedgerEntry| {
                                count += 1;
                                Ok::<(), LoadError>(())
                            })
                            .unwrap();
                        black_box(count)
                    },
                    BatchSize::LargeInput,
                );
            },
        );
    }
    group.finish();
}

fn report_process_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("process");
    group.warm_up_time(std::time::Duration::from_secs(7));
    for params in InputParams::params_from_env() {
        if let Some(samples) = params.sample_size {
            group.sample_size(samples);
        }
        group.bench_with_input(BenchmarkId::from_parameter(params), &params, |b, params| {
            let input = FakeFileSink::new_example(Path::new("report_bench"), *params).unwrap();
            let opts = report::ProcessOptions::default();

            b.iter_custom(|iters| {
                let walltime = WallTime;
                let mut total = walltime.zero();
                for _i in 0..iters {
                    let arena = Bump::new();
                    let loader = input.new_loader();
                    let mut ctx = report::ReportContext::new(&arena);
                    let start = walltime.start();
                    report::process(&mut ctx, loader, &opts).expect("report::process must succeed");
                    total = walltime.add(&total, &walltime.end(start));
                }
                total
            });
        });
    }
    group.finish();
}

fn query_postings(c: &mut Criterion) {
    let input =
        FakeFileSink::new_example(Path::new("report_bench"), InputParams::middle()).unwrap();
    let arena = Bump::new();
    let mut ctx = report::ReportContext::new(&arena);
    let opts = report::ProcessOptions::default();
    let ledger =
        report::process(&mut ctx, input.new_loader(), &opts).expect("report::process must succeed");

    c.bench_function("query-posting-one-account", |b| {
        b.iter_with_large_drop(|| {
            let query = report::query::PostingQuery {
                account: Some("Assets:Account02".to_string()),
            };
            black_box(ledger.postings(&ctx, &query));
        })
    });
}
fn query_balance(c: &mut Criterion) {
    let mut group = c.benchmark_group("query::balance");

    for params in InputParams::params_from_env() {
        let input = FakeFileSink::new_example(Path::new("report_bench"), params).unwrap();
        let arena = Bump::new();
        let mut ctx = report::ReportContext::new(&arena);
        let opts = report::ProcessOptions::default();
        let mut ledger = report::process(&mut ctx, input.new_loader(), &opts)
            .expect("report::process must succeed");

        group.bench_with_input(
            BenchmarkId::new("default", &params),
            &params,
            |b, _params| {
                b.iter_with_large_drop(|| {
                    black_box(
                        ledger
                            .balance(&ctx, &report::query::BalanceQuery::default())
                            .unwrap(),
                    );
                })
            },
        );

        let query = report::query::BalanceQuery {
            date_range: report::query::DateRange {
                start: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
                end: Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
            },
            conversion: None,
        };
        group.bench_with_input(
            BenchmarkId::new("date-range", &params),
            &params,
            |b, _params| {
                b.iter_with_large_drop(|| {
                    black_box(ledger.balance(&ctx, &query).unwrap());
                })
            },
        );

        let usd = ctx.commodity("USD").unwrap();

        let query = report::query::BalanceQuery {
            date_range: report::query::DateRange::default(),
            conversion: Some(report::query::Conversion {
                strategy: report::query::ConversionStrategy::UpToDate {
                    now: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
                },
                target: usd,
            }),
        };
        group.bench_with_input(
            BenchmarkId::new("conversion-up-to-date", &params),
            &params,
            |b, _params| {
                b.iter_with_large_drop(|| {
                    black_box(ledger.balance(&ctx, &query).unwrap());
                })
            },
        );

        let chf = ctx.commodity("CHF").unwrap();

        let query = report::query::BalanceQuery {
            date_range: report::query::DateRange::default(),
            conversion: Some(report::query::Conversion {
                strategy: report::query::ConversionStrategy::Historical,
                target: chf,
            }),
        };
        group.bench_with_input(
            BenchmarkId::new("conversion-historical", &params),
            &params,
            |b, _params| {
                b.iter_with_large_drop(|| {
                    black_box(ledger.balance(&ctx, &query).unwrap());
                })
            },
        );
    }
    group.finish();
}

fn basic_asserts<T: FileSink>(input: &ExampleInput<T>) {
    let arena = Bump::new();
    let mut ctx = report::ReportContext::new(&arena);
    let opts = report::ProcessOptions::default();
    let ledger =
        report::process(&mut ctx, input.new_loader(), &opts).expect("report::process must succeed");
    let num_txns = ledger.transactions().count();

    assert_eq!(input.num_transactions(), num_txns as u64);
}

#[ctor::ctor]
fn init() {
    // print INFO level logs by default, unless overridden by env.
    let mut builder = env_logger::builder();
    builder
        .is_test(true)
        .filter_level(LevelFilter::Info)
        .parse_default_env();
    let _ = builder.try_init();
}

criterion_group!(
    benches,
    load_benchmark,
    report_process_benchmark,
    query_postings,
    query_balance
);
criterion_main!(benches);
