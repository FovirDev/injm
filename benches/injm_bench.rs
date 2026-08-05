use std::path::Path;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use injm::injector::inject;
use injm::parser::marker::extract_marker_blocks;
use injm::types::{BlockRole, MarkerBlock, MarkerConfig, SourceSpan};

fn rust_source(blocks: usize) -> String {
    let mut s = String::with_capacity(blocks * 64);
    for i in 0..blocks {
        s.push_str("fn f");
        s.push_str(&i.to_string());
        s.push_str("() {\n    // injm begin <id_");
        s.push_str(&i.to_string());
        s.push_str(">\n    let x = ");
        s.push_str(&i.to_string());
        s.push_str(";\n    // injm end\n}\n\n");
    }
    s
}

fn markdown_source(blocks: usize) -> String {
    let mut s = String::with_capacity(blocks * 64);
    for i in 0..blocks {
        s.push_str("<!-- injm begin <id_");
        s.push_str(&i.to_string());
        s.push_str(" -->\n\nsome content ");
        s.push_str(&i.to_string());
        s.push_str("\n\n<!-- injm end -->\n\n");
    }
    s
}

fn bench_parse(c: &mut Criterion) {
    let rust = rust_source(500);
    c.bench_function("extract_marker_blocks/rust_500_blocks", |b| {
        b.iter(|| extract_marker_blocks(black_box(&rust), Path::new("x.rs"), "rust").unwrap())
    });

    let markdown = markdown_source(500);
    c.bench_function("extract_marker_blocks/markdown_500_blocks", |b| {
        b.iter(|| {
            extract_marker_blocks(black_box(&markdown), Path::new("x.md"), "markdown").unwrap()
        })
    });
}

fn bench_inject(c: &mut Criterion) {
    let content = rust_source(1000);
    let blocks = extract_marker_blocks(&content, Path::new("x.rs"), "rust").unwrap();
    let outputs: Vec<MarkerBlock> = blocks
        .into_iter()
        .filter(|b| matches!(b.role, BlockRole::Output { .. }))
        .collect();
    let input = MarkerBlock {
        span: SourceSpan::new(0, 0),
        content: "let y = 1;\nlet z = 2;".to_string(),
        role: BlockRole::Input { ids: vec![] },
        config: MarkerConfig::default(),
    };
    let inputs = vec![input];

    c.bench_function("inject/1000_blocks", |b| {
        b.iter(|| inject(black_box(&content), black_box(&outputs), black_box(&inputs)))
    });
}

criterion_group!(benches, bench_parse, bench_inject);
criterion_main!(benches);
