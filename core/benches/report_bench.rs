use std::path::Path;

use bumpalo::Bump;
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
                .load_repl(|_, _, _: &syntax::tracked::LedgerEntry| {
                    count += 1;
                    Ok::<(), LoadError>(())
                })
                .unwrap();
            black_box(());
            black_box(count)
        })
    });
}

fn report_benchmark(c: &mut Criterion) {
    let input = testing::ExampleInput::new(Path::new("report_bench")).unwrap();
    c.bench_function("process", |b| {
        b.iter(|| {
            let arena = Bump::new();
            let mut ctx = report::ReportContext::new(&arena);
            let ret = report::process(&mut ctx, load::new_loader(input.rootpath().to_owned()))
                .expect("report::process must succeed");
            black_box(ret);
        })
    });
}

#[ctor::ctor]
fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

criterion_group!(benches, load_benchmark, report_benchmark);
criterion_main!(benches);
