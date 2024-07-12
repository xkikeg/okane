use std::str::FromStr;

use okane_core::repl::pretty_decimal::PrettyDecimal;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn parse_benchmark(c: &mut Criterion) {
    c.bench_function("parse plain", |b| {
        b.iter(|| black_box(PrettyDecimal::from_str("12345.678").unwrap()))
    });
    c.bench_function("parse comma", |b| {
        b.iter(|| black_box(PrettyDecimal::from_str("12,345.678").unwrap()))
    });
}

#[ctor::ctor]
fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

criterion_group!(benches, parse_benchmark);
criterion_main!(benches);
