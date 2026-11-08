//! Instruction-count benchmarks for the report engine, run with Gungraun
//! (Valgrind's Callgrind).
//!
//! The wall-time benches in `report_bench` depend on the CPU of the machine
//! they run on, and GitHub-hosted runners vary from job to job. Instruction
//! counts shouldn't, so CI also runs these (see `bench-gungraun.yml`).
//!
//! Each case runs once under Valgrind, which is ~50x slower than native, so
//! we only use the `small` dataset. Inputs are built in `setup` functions,
//! which Gungraun excludes from the counts. Setup and bench results are
//! leaked so that dropping them doesn't count either.

use std::borrow::Cow;
use std::hint::black_box;
use std::path::Path;

use bumpalo::Bump;
use chrono::NaiveDate;
use gungraun::{library_benchmark, library_benchmark_group, main};
use lender::FallibleLender;
use okane_core::{
    load::{self, LoadError},
    report::{
        self, ProcessOptions, ReportContext,
        query::{
            AccountFilter, BalanceQuery, Conversion, ConversionStrategy, DateRange, Ledger,
            RegisterQuery, Sort,
        },
    },
    syntax,
};
use testing::{ExampleInput, FakeFileSink, FileSink, InputParams};

pub mod testing;

fn params() -> InputParams {
    InputParams::small()
}

fn leak<T>(value: T) -> &'static mut T {
    Box::leak(Box::new(value))
}

fn example() -> &'static ExampleInput<FakeFileSink> {
    leak(
        FakeFileSink::new_example(Path::new("report_gungraun"), params())
            .expect("failed to create bench input"),
    )
}

fn arena() -> &'static Bump {
    leak(Bump::new())
}

/// Loaded and processed ledger, ready for queries.
struct Processed {
    ctx: ReportContext<'static>,
    ledger: Ledger<'static>,
}

fn processed(with_price_db: bool) -> &'static mut Processed {
    let input = example();
    let mut ctx = ReportContext::new(arena());
    let opts = ProcessOptions {
        price_db_path: with_price_db.then(|| input.pricedbpath().to_owned()),
    };
    let ledger =
        report::process(&mut ctx, input.new_loader(), &opts).expect("report::process must succeed");
    leak(Processed { ctx, ledger })
}

fn middle20() -> DateRange {
    let (start, end) = params().date_range_middle();
    DateRange {
        start: Some(start),
        end: Some(end),
    }
}

fn historical(ctx: &ReportContext<'static>) -> Option<Conversion<'static>> {
    Some(Conversion {
        strategy: ConversionStrategy::Historical,
        target: ctx.commodity("CHF").expect("CHF must exist in bench input"),
    })
}

// load / process

type Loader = load::Loader<load::FakeFileSystem>;

fn setup_loader() -> &'static Loader {
    leak(example().new_loader())
}

// Keep this closure out of the bench function: Callgrind toggles counting
// whenever it enters a function whose name matches the bench, and
// `load_impl::<{closure}>` would match through the closure's type name.
fn count_entries(loader: &Loader) -> usize {
    let mut count = 0;
    loader
        .load(|_, _, _: &syntax::tracked::LedgerEntry| {
            count += 1;
            Ok::<(), LoadError>(())
        })
        .expect("load must succeed");
    count
}

#[library_benchmark]
#[bench::small(setup = setup_loader)]
fn load_on_memory(loader: &'static Loader) -> usize {
    black_box(count_entries(loader))
}

fn setup_process() -> (&'static Loader, &'static Bump) {
    (setup_loader(), arena())
}

#[library_benchmark]
#[bench::small(setup = setup_process)]
fn process((loader, arena): (&'static Loader, &'static Bump)) -> &'static Processed {
    let mut ctx = ReportContext::new(arena);
    let ledger = report::process(&mut ctx, loader, &ProcessOptions::default())
        .expect("report::process must succeed");
    black_box(leak(Processed { ctx, ledger }))
}

library_benchmark_group!(name = load_process, benchmarks = [load_on_memory, process]);

// query::balance

enum BalanceCase {
    DateRangeMiddle20,
    ConversionUpToDate,
    ConversionHistorical,
    ConversionHistoricalPriceDb,
}

fn setup_balance(case: BalanceCase) -> (&'static mut Processed, BalanceQuery<'static>) {
    let p = processed(matches!(case, BalanceCase::ConversionHistoricalPriceDb));
    let mut query = BalanceQuery::default();
    match case {
        BalanceCase::DateRangeMiddle20 => query.date_range = middle20(),
        BalanceCase::ConversionUpToDate => {
            query.conversion = Some(Conversion {
                strategy: ConversionStrategy::UpToDate {
                    today: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
                },
                target: p
                    .ctx
                    .commodity("USD")
                    .expect("USD must exist in bench input"),
            })
        }
        BalanceCase::ConversionHistorical | BalanceCase::ConversionHistoricalPriceDb => {
            query.conversion = historical(&p.ctx)
        }
    }
    (p, query)
}

#[library_benchmark]
#[bench::date_range_middle20(args = (BalanceCase::DateRangeMiddle20), setup = setup_balance)]
#[bench::conversion_up_to_date(args = (BalanceCase::ConversionUpToDate), setup = setup_balance)]
#[bench::conversion_historical(args = (BalanceCase::ConversionHistorical), setup = setup_balance)]
#[bench::conversion_historical_pricedb(args = (BalanceCase::ConversionHistoricalPriceDb), setup = setup_balance)]
fn balance(
    (p, query): (&'static mut Processed, BalanceQuery<'static>),
) -> &'static Cow<'static, report::Balance<'static>> {
    black_box(leak(
        p.ledger
            .balance(&p.ctx, &query)
            .expect("balance must succeed"),
    ))
}

library_benchmark_group!(name = query_balance, benchmarks = balance);

// query::register

enum RegisterCase {
    Default,
    AccountFilter,
    DateRangeMiddle20,
    ConversionHistorical,
    ConversionHistoricalPriceDb,
    SortDateCold,
    SortDateWarm,
}

fn setup_register(case: RegisterCase) -> (&'static mut Processed, RegisterQuery<'static>) {
    let p = processed(matches!(case, RegisterCase::ConversionHistoricalPriceDb));
    let mut query = RegisterQuery::default();
    match case {
        RegisterCase::Default => (),
        RegisterCase::AccountFilter => {
            query.account = AccountFilter::single(
                p.ctx
                    .account("Assets:Account02")
                    .expect("Assets:Account02 must exist in bench input"),
            )
        }
        RegisterCase::DateRangeMiddle20 => query.date_range = middle20(),
        RegisterCase::ConversionHistorical | RegisterCase::ConversionHistoricalPriceDb => {
            query.conversion = historical(&p.ctx)
        }
        // The first `Sort::Date` query sorts the transactions and caches the
        // order; later ones reuse it.
        RegisterCase::SortDateCold => query.sort = Sort::Date,
        RegisterCase::SortDateWarm => {
            query.sort = Sort::Date;
            count_register(p, &query);
        }
    }
    (p, query)
}

fn count_register(p: &mut Processed, query: &RegisterQuery<'static>) -> u64 {
    let mut entries = p
        .ledger
        .register_entries(&p.ctx, query)
        .expect("register_entries must succeed");
    let mut count = 0;
    while entries
        .next()
        .expect("register entry must succeed")
        .is_some()
    {
        count += 1;
    }
    count
}

#[library_benchmark]
#[bench::default(args = (RegisterCase::Default), setup = setup_register)]
#[bench::account_filter(args = (RegisterCase::AccountFilter), setup = setup_register)]
#[bench::date_range_middle20(args = (RegisterCase::DateRangeMiddle20), setup = setup_register)]
#[bench::conversion_historical(args = (RegisterCase::ConversionHistorical), setup = setup_register)]
#[bench::conversion_historical_pricedb(args = (RegisterCase::ConversionHistoricalPriceDb), setup = setup_register)]
#[bench::sort_date_cold(args = (RegisterCase::SortDateCold), setup = setup_register)]
#[bench::sort_date_warm(args = (RegisterCase::SortDateWarm), setup = setup_register)]
fn register((p, query): (&'static mut Processed, RegisterQuery<'static>)) -> u64 {
    black_box(count_register(p, &query))
}

library_benchmark_group!(name = query_register, benchmarks = register);

main!(
    library_benchmark_groups = load_process,
    query_balance,
    query_register
);
