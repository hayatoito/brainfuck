use criterion::Criterion;
use criterion::*;

use brainfuck;

macro_rules! bench_bf {
    ($c:ident, $bf:expr, $opt:expr) => {
        let bytes = include_bytes!(concat!("brainfuck/", $bf, ".bf"));
        $c.bench_function(&format!("{}{}", $bf, $opt), move |b| {
            b.iter(|| {
                let mut out = Vec::new();
                brainfuck::run(bytes, &[] as &[u8], &mut out, Some($opt), false).unwrap();
            })
        });
    };
}

fn bench_trivial_loop_1(c: &mut Criterion) {
    bench_bf!(c, "trivial-loop", 1);
}

fn bench_trivial_loop_2(c: &mut Criterion) {
    bench_bf!(c, "trivial-loop", 2);
}

fn bench_trivial_loop_3(c: &mut Criterion) {
    bench_bf!(c, "trivial-loop", 3);
}

fn bench_nested_loop_1(c: &mut Criterion) {
    bench_bf!(c, "nested-loop", 1);
}

fn bench_nested_loop_2(c: &mut Criterion) {
    bench_bf!(c, "nested-loop", 2);
}

fn bench_nested_loop_3(c: &mut Criterion) {
    bench_bf!(c, "nested-loop", 3);
}

fn bench_sierpinski_1(c: &mut Criterion) {
    bench_bf!(c, "sierpinski", 1);
}

fn bench_sierpinski_2(c: &mut Criterion) {
    bench_bf!(c, "sierpinski", 2);
}

fn bench_sierpinski_3(c: &mut Criterion) {
    bench_bf!(c, "sierpinski", 3);
}

criterion_group!(
    benches,
    bench_trivial_loop_1,
    bench_trivial_loop_2,
    bench_trivial_loop_3,
    bench_nested_loop_1,
    bench_nested_loop_2,
    bench_nested_loop_3,
    bench_sierpinski_1,
    bench_sierpinski_2,
    bench_sierpinski_3,
);
// criterion_group!(benches, bench_trivial_loop);
criterion_main!(benches);
