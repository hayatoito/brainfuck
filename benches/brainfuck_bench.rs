use criterion::Criterion;
use criterion::*;

use brainfuck;

fn run_brainfuck(s: &[u8]) {
    let mut out = Vec::new();
    brainfuck::run(s, &[] as &[u8], &mut out).unwrap();
}

fn bench_trivial_loop(c: &mut Criterion) {
    let bytes = include_bytes!("brainfuck/trivial-loop.bf");
    c.bench_function("trivial-loop", move |b| b.iter(|| run_brainfuck(bytes)));
}

fn bench_sierpinski(c: &mut Criterion) {
    let bytes = include_bytes!("brainfuck/sierpinski.bf");
    c.bench_function("sierpinski", move |b| b.iter(|| run_brainfuck(bytes)));
}

criterion_group!(benches, bench_trivial_loop, bench_sierpinski);
// criterion_group!(benches, bench_trivial_loop);
criterion_main!(benches);
