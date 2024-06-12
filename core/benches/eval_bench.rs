use std::path::Path;

use okane_core::load::load_repl;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

mod testing;

fn load_benchmark(c: &mut Criterion) {
    let input = testing::ExampleInput::new(Path::new("load_bench")).unwrap();
    c.bench_function("load simple", |b| {
        b.iter(|| black_box(load_repl(&input.rootpath())))
    });
}

#[ctor::ctor]
fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

criterion_group!(benches, load_benchmark);
criterion_main!(benches);
