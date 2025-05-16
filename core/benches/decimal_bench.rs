use std::str::FromStr;

use okane_core::syntax::pretty_decimal::PrettyDecimal;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_decimal_macros::dec;

fn parse_benchmark(c: &mut Criterion) {
    c.bench_function("parse plain", |b| {
        b.iter(|| black_box(PrettyDecimal::from_str("12345.678").unwrap()))
    });
    c.bench_function("parse comma", |b| {
        b.iter(|| black_box(PrettyDecimal::from_str("12,345.678").unwrap()))
    });
}

fn to_string_benchmark(c: &mut Criterion) {
    c.bench_function("to_string plain", |b| {
        b.iter(|| black_box(PrettyDecimal::plain(dec!(12_345_678.90)).to_string()))
    });
    c.bench_function("to_string comma", |b| {
        b.iter(|| black_box(PrettyDecimal::comma3dot(dec!(12_345_678.90)).to_string()))
    });
}

#[ctor::ctor]
fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

criterion_group!(benches, parse_benchmark, to_string_benchmark);
criterion_main!(benches);
