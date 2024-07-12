use std::path::Path;

use okane_core::load::{self, LoadError};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub mod testing;

fn load_benchmark(c: &mut Criterion) {
    let input = testing::ExampleInput::new(Path::new("load_bench")).unwrap();
    c.bench_function("load simple", |b| {
        b.iter(|| {
            let mut count = 0;
            load::new_loader(input.rootpath().to_owned())
                .load_repl(|_, _| {
                    count += 1;
                    Ok::<(), LoadError>(())
                })
                .unwrap();
            black_box(());
            black_box(count)
        })
    });
}

#[ctor::ctor]
fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

criterion_group!(benches, load_benchmark);
criterion_main!(benches);
