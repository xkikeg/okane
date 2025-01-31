use std::path::Path;

use bumpalo::Bump;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use log::LevelFilter;
use okane_core::{
    load::LoadError,
    report::{self, query::PostingQuery},
    syntax,
};
use pretty_assertions::assert_eq;
use testing::{new_example, ExampleInput, FakeFileSink, FileSink, RealFileSink};

pub mod testing;

fn load_benchmark(c: &mut Criterion) {
    let input = new_example::<FakeFileSink>(Path::new("report_bench")).unwrap();

    basic_asserts(&input);

    c.bench_function("load-on-memory", |b| {
        b.iter(|| {
            let mut count = 0;
            input
                .new_loader()
                .load(|_, _, _: &syntax::tracked::LedgerEntry| {
                    count += 1;
                    Ok::<(), LoadError>(())
                })
                .unwrap();
            black_box(());
            black_box(count)
        })
    });

    let input = new_example::<RealFileSink>(Path::new("report_bench")).unwrap();

    basic_asserts(&input);

    c.bench_function("load-on-file", |b| {
        b.iter(|| {
            let mut count = 0;
            input
                .new_loader()
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
    let input = new_example::<FakeFileSink>(Path::new("report_bench")).unwrap();
    let opts = report::ProcessOptions::default();

    c.bench_function("process", |b| {
        b.iter(|| {
            let arena = Bump::new();
            let mut ctx = report::ReportContext::new(&arena);
            let ret = report::process(&mut ctx, input.new_loader(), &opts)
                .expect("report::process must succeed");
            black_box(ret);
        })
    });
}

fn query_postings(c: &mut Criterion) {
    let input = new_example::<FakeFileSink>(Path::new("report_bench")).unwrap();
    let arena = Bump::new();
    let mut ctx = report::ReportContext::new(&arena);
    let opts = report::ProcessOptions::default();
    let ledger =
        report::process(&mut ctx, input.new_loader(), &opts).expect("report::process must succeed");

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
    let input = new_example::<FakeFileSink>(Path::new("report_bench")).unwrap();
    let arena = Bump::new();
    let mut ctx = report::ReportContext::new(&arena);
    let opts = report::ProcessOptions::default();
    let ledger =
        report::process(&mut ctx, input.new_loader(), &opts).expect("report::process must succeed");

    c.bench_function("query-balance-default", |b| {
        b.iter(|| {
            black_box(ledger.balance());
        })
    });
}

fn basic_asserts<T: FileSink>(input: &ExampleInput<T>) {
    let arena = Bump::new();
    let mut ctx = report::ReportContext::new(&arena);
    let opts = report::ProcessOptions::default();
    let ledger =
        report::process(&mut ctx, input.new_loader(), &opts).expect("report::process must succeed");
    let num_txns = ledger.transactions().count();

    assert_eq!(testing::num_transactions(), num_txns as u64);
}

#[ctor::ctor]
fn init() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(LevelFilter::max())
        .try_init();
}

criterion_group!(
    benches,
    load_benchmark,
    report_process_benchmark,
    query_postings,
    query_balance
);
criterion_main!(benches);
