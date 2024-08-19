use std::fmt::Write as _;
use std::sync::atomic::{self, AtomicI64};
use std::{
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use bumpalo::Bump;
use maplit::hashmap;
use okane_core::{
    load::{self, LoadError},
    report::{self, Evaluable},
};
use okane_core::{parse, repl};

use criterion::{black_box, criterion_group, Criterion};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub mod testing;

fn example_input() -> &'static Mutex<Option<testing::ExampleInput>> {
    static BENCH_INPUT: OnceLock<Mutex<Option<testing::ExampleInput>>> = OnceLock::new();
    BENCH_INPUT.get_or_init(|| {
        Mutex::new(Some(
            testing::ExampleInput::new(Path::new("report_bench"))
                .expect("ExampleInput creation failed"),
        ))
    })
}

struct JustDrop<'a>(&'a Path);

static JUST_DROP_COUNT: AtomicI64 = AtomicI64::new(0);

impl<'a> Drop for JustDrop<'a> {
    fn drop(&mut self) {
        JUST_DROP_COUNT.fetch_add(1, atomic::Ordering::SeqCst);
    }
}

fn load_benchmark(c: &mut Criterion) {
    // for i in 0..1000 {
    //     if (i + 1) % 1000 == 0 {
    //         log::info!("loop at {}, pause 1 minute", i + 1);
    //         std::thread::sleep(std::time::Duration::from_secs(10));
    //         log::info!("resume");
    //     }
    //     let arena = Bump::new();
    //     let mut ctx = report::ReportContext::new(&arena);
    //     let mut b = report::Balance::default();
    //     let mut input = String::new();
    //     write!(&mut input, "{} JPY", Decimal::from(i) / dec!(100)).expect("must succeed");
    //     let jpy = ctx.ensure_commodity("JPY");
    //     for _j in 0..10000 {
    //         b.increment(
    //             ctx.ensure_account("Account X"),
    //             report::Amount::from_value(dec!(1), jpy),
    //         );
    //         let expr: repl::expr::ValueExpr = black_box(&input)
    //             .as_str()
    //             .try_into()
    //             .expect("must be valid");
    //         use okane_core::report::Evaluable;
    //         b.set_balance(
    //             ctx.ensure_account("Account X"),
    //             expr.eval(&mut ctx)
    //                 .expect("must succeed")
    //                 .try_into()
    //                 .expect("this must be a single amount"),
    //         );
    //     }
    //     black_box(b);
    // }
    {
        for i in 0..200 {
            if (i + 1) % 100 == 0 {
                log::info!("loop at {}, pause 1 second", i + 1);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            let arena = Bump::new();
            let mut ctx = report::ReportContext::new(&arena);
            let input = "2024/01/01 Foo\n Assets    10 JPY\n Income    -10 JPY\n".to_string();
            let mut bal = report::Balance::default();
            for _j in 0..10000 {
                let (_, txn): (_, repl::LedgerEntry) = parse::parse_ledger(&input)
                    .next()
                    .expect("parse must have one element")
                    .expect("parse must succeed");
                let txn = match txn {
                    repl::LedgerEntry::Txn(txn) => Some(txn),
                    _ => None,
                }
                .expect("must be txn");
                report::add_transaction(&mut ctx, &mut bal, &txn)
                    .expect("add_transaction must succeed");
                // black_box(txn);
                // let expr: repl::expr::ValueExpr = "10 JPY".try_into().expect("must succeed");
                // let amount: report::Amount = expr
                //     .eval(&mut ctx)
                //     .expect("eval must succeed")
                //     .try_into()
                //     .expect("evaluated must be an Amount");
                // bal.increment(ctx.ensure_account("Foo"), amount);
            }
            black_box(bal);
        }
        // assert_eq!(
        //     report::AMOUNT_DEFAULT_COUNT.load(atomic::Ordering::SeqCst),
        //     6000400
        // );
        // assert_eq!(
        //     report::AMOUNT_DROP_COUNT.load(atomic::Ordering::SeqCst),
        //     14000400
        // );
    }

    let input = example_input().lock().expect("Mutex::lock must succeed");
    let rootpath = input.as_ref().expect("test input must exist").rootpath();
    c.bench_function("load simple", |b| {
        b.iter(|| {
            let mut count = 0;
            load::new_loader(rootpath.to_owned())
                .load_repl(|_, _, _| {
                    count += 1;
                    Ok::<(), LoadError>(())
                })
                .unwrap();
            black_box(count)
        })
    });
}

// fn report_benchmark(c: &mut Criterion) {
//     let input = example_input().lock().expect("Mutex::lock must succeed");
//     let rootpath = input.as_ref().expect("test input must exist").rootpath();
//     let mut group = c.benchmark_group("report");
//     group.sample_size(10);
//     group.bench_function("report simple", |b| {
//         b.iter(|| {
//             let arena = Bump::new();
//             let mut ctx = report::ReportContext::new(&arena);
//             let ret = report::process(&mut ctx, load::new_loader(rootpath.to_owned())).unwrap();
//             black_box(ret);
//             drop(ctx);
//             drop(arena)
//         })
//     });
// }

#[ctor::ctor]
fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

// criterion_group!(benches, load_benchmark, report_benchmark);
criterion_group!(benches, load_benchmark);

fn main() {
    benches();
    example_input().lock().unwrap().take();
    Criterion::default().configure_from_args().final_summary();
}
