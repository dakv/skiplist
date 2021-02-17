use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dakv_skiplist::SkipList;

fn criterion_benchmark(c: &mut Criterion) {
    let mut sl: SkipList<usize> = SkipList::default();
    c.bench_function("SkipList insert", |b| b.iter(|| sl.insert(black_box(&1))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
