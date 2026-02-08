use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use itertools::Itertools;
use std::fs;

fn tiny_markdown_string(c: &mut Criterion) {
    let doc = "A *single* [markdown](/path) string!".to_owned();
    c.bench_with_input(BenchmarkId::new("tiny", "markdown_string"), &doc, |b, s| {
        b.iter(|| markdown::to_html(s));
    });
}

fn readme(c: &mut Criterion) {
    let doc = fs::read_to_string("readme.md").unwrap();

    c.bench_with_input(BenchmarkId::new("medium", "readme"), &doc, |b, s| {
        b.iter(|| markdown::to_html(s));
    });
}

fn large_jsx_expressions(c: &mut Criterion) {
    let num_components = 3;
    let num_jsx_lines_per_component = 5000;
    fn dummy_component(code: String) -> String {
        return format!("<DummyComponent code={{`{code}`}} />");
    }
    fn uuids(len: usize) -> String {
        (0..len)
            .map(|_| "770f93e8-b4ee-4ce8-ab0f-4ece7d8c1090")
            .join("\n")
    }
    let doc = (0..num_components)
        .map(|_| dummy_component(uuids(num_jsx_lines_per_component)))
        .join("\n\n");

    c.bench_with_input(BenchmarkId::new("large", "jsx_expression"), &doc, |b, s| {
        b.iter(|| markdown::to_html(s));
    });
}

criterion_group!(benches, tiny_markdown_string, readme, large_jsx_expressions);
criterion_main!(benches);
