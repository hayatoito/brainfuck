use criterion::Criterion;
use criterion::*;

use brainfuck;

fn run_brainfuck_1(s: &[u8]) {
    let mut out = Vec::new();
    brainfuck::run1(s, &[] as &[u8], &mut out).unwrap();
}

fn run_brainfuck_2(s: &[u8]) {
    let mut out = Vec::new();
    brainfuck::run2(s, &[] as &[u8], &mut out).unwrap();
}

// TODO: Use macro
macro_rules! bench_bf {
    ($c:ident, 1, $bf:expr) => {
        let bytes = include_bytes!(concat!("brainfuck/", $bf, ".bf"));
        $c.bench_function(concat!($bf, "-1"), move |b| {
            b.iter(|| run_brainfuck_1(bytes))
        });
    };
    ($c:ident, 2, $bf:expr) => {
        let bytes = include_bytes!(concat!("brainfuck/", $bf, ".bf"));
        $c.bench_function(concat!($bf, "-2"), move |b| {
            b.iter(|| run_brainfuck_2(bytes))
        });
    };
}

fn bench_trivial_loop_1(c: &mut Criterion) {
    // let bytes = include_bytes!("brainfuck/trivial-loop.bf");
    // c.bench_function("trivial-loop-1", move |b| b.iter(|| run_brainfuck_1(bytes)));
    bench_bf!(c, 1, "trivial-loop");
}

fn bench_trivial_loop_2(c: &mut Criterion) {
    bench_bf!(c, 2, "trivial-loop");
}

fn bench_sierpinski_1(c: &mut Criterion) {
    bench_bf!(c, 1, "sierpinski");
}

fn bench_sierpinski_2(c: &mut Criterion) {
    bench_bf!(c, 2, "sierpinski");
}

fn bench_nested_loop_1(c: &mut Criterion) {
    bench_bf!(c, 1, "nested-loop");
}

fn bench_nested_loop_2(c: &mut Criterion) {
    bench_bf!(c, 2, "nested-loop");
}

criterion_group!(
    benches,
    bench_trivial_loop_1,
    bench_trivial_loop_2,
    bench_sierpinski_1,
    bench_sierpinski_2,
    bench_nested_loop_1,
    bench_nested_loop_2,
);
// criterion_group!(benches, bench_trivial_loop);
criterion_main!(benches);
