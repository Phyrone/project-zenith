use criterion::{Criterion, criterion_group, criterion_main};

criterion_group!(benches, common,compression);
criterion_main!(benches);

fn common(criterion: &mut Criterion) {}


fn compression(criterion: &mut Criterion) {}