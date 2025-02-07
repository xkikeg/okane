use std::path::Path;

use bumpalo::Bump;
use chrono::NaiveDate;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use okane_core::{
    load::{self, LoadError},
    report, syntax,
};

pub mod testing;

fn load_benchmark(c: &mut Criterion) {
    let input = testing::ExampleInput::new(Path::new("report_bench")).unwrap();
    c.bench_function("load-with-counter", |b| {
        b.iter(|| {
            let mut count = 0;
            load::new_loader(input.rootpath().to_owned())
                .load(|_, _, _: &syntax::tracked::LedgerEntry| {
                    count += 1;
                    Ok::<(), LoadError>(())
                })
                .unwrap();
            black_box(());
            black_box(count)
        })
    });
}

fn report_process_benchmark(c: &mut Criterion) {
    let opts = report::ProcessOptions::default();
    let input = testing::ExampleInput::new(Path::new("report_bench")).unwrap();
    c.bench_function("process", |b| {
        b.iter(|| {
            let arena = Bump::new();
            let mut ctx = report::ReportContext::new(&arena);
            let ret = report::process(
                &mut ctx,
                load::new_loader(input.rootpath().to_owned()),
                &opts,
            )
            .expect("report::process must succeed");
            black_box(ret);
        })
    });
}

fn query_postings(c: &mut Criterion) {
    let input = testing::ExampleInput::new(Path::new("report_bench")).unwrap();
    let arena = Bump::new();
    let mut ctx = report::ReportContext::new(&arena);
    let opts = report::ProcessOptions::default();
    let ledger = report::process(
        &mut ctx,
        load::new_loader(input.rootpath().to_owned()),
        &opts,
    )
    .expect("report::process must succeed");

    c.bench_function("query-posting-one-account", |b| {
        b.iter(|| {
            let query = report::query::PostingQuery {
                account: Some("Assets:Account02".to_string()),
            };
            black_box(ledger.postings(&ctx, &query));
        })
    });
}
fn query_balance(c: &mut Criterion) {
    let input = testing::ExampleInput::new(Path::new("report_bench")).unwrap();
    let arena = Bump::new();
    let mut ctx = report::ReportContext::new(&arena);
    let opts = report::ProcessOptions::default();
    let mut ledger = report::process(
        &mut ctx,
        load::new_loader(input.rootpath().to_owned()),
        &opts,
    )
    .expect("report::process must succeed");

    c.bench_function("query-balance-default", |b| {
        b.iter(|| {
            black_box(
                ledger
                    .balance(&ctx, &report::query::BalanceQuery::default())
                    .unwrap(),
            );
        })
    });

    let query = report::query::BalanceQuery {
        date_range: report::query::DateRange {
            start: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            end: Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
        },
        conversion: None,
    };
    c.bench_function("query-balance-conversion-date", |b| {
        b.iter(|| {
            black_box(ledger.balance(&ctx, &query).unwrap());
        })
    });

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
    c.bench_function("query-balance-conversion-up-to-date", |b| {
        b.iter(|| {
            black_box(ledger.balance(&ctx, &query).unwrap());
        })
    });

    let chf = ctx.commodity("CHF").unwrap();

    let query = report::query::BalanceQuery {
        date_range: report::query::DateRange::default(),
        conversion: Some(report::query::Conversion {
            strategy: report::query::ConversionStrategy::Historical,
            target: chf,
        }),
    };
    c.bench_function("query-balance-conversion-historical", |b| {
        b.iter(|| {
            black_box(ledger.balance(&ctx, &query).unwrap());
        })
    });
}

#[ctor::ctor]
fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

criterion_group!(
    benches,
    load_benchmark,
    report_process_benchmark,
    query_postings,
    query_balance
);
criterion_main!(benches);
