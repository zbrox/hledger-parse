use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hledger_parse::parse_journal;

const SIMPLE_JOURNAL: &str = include_str!("../examples/simple.journal");

fn bench_journal_parse(c: &mut Criterion) {
    let mut journal = SIMPLE_JOURNAL;
    c.bench_function("simple journal parse", |b| b.iter(|| parse_journal(black_box(&mut journal), None)));
}

criterion_group!(benches, bench_journal_parse);
criterion_main!(benches);