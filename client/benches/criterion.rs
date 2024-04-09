use criterion::{criterion_group, criterion_main, Criterion};

criterion_group!(benches, common, compression);
criterion_main!(benches);

fn common(criterion: &mut Criterion) {}

fn compression(criterion: &mut Criterion) {}
