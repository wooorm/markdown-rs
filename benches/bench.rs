use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::fs;

fn readme(c: &mut Criterion) {
    let doc = fs::read_to_string("readme.md").unwrap();

    c.bench_with_input(BenchmarkId::new("readme", "readme"), &doc, |b, s| {
        b.iter(|| markdown::to_html(s));
    });
}

// fn one_and_a_half_mb(c: &mut Criterion) {
//     let doc = fs::read_to_string("../a-dump-of-markdown/markdown.md").unwrap();
//     let mut group = c.benchmark_group("giant");
//     group.sample_size(10);
//     group.bench_with_input(BenchmarkId::new("giant", "1.5 mb"), &doc, |b, s| {
//         b.iter(|| markdown::to_html(s));
//     });
//     group.finish();
// }
// , one_and_a_half_mb

criterion_group!(benches, readme);
criterion_main!(benches);
