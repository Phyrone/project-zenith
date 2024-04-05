use criterion::{criterion_group, criterion_main, Criterion};
criterion_group!(benches, common);

criterion_main!(benches);

fn common(criterion: &mut Criterion) {}
