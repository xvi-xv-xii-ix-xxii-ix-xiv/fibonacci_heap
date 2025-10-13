// benches/benchmarks.rs
use criterion::{criterion_group, criterion_main, Criterion};
use fibonacci_heap::FibonacciHeap;
use std::hint::black_box;

fn bench_insert(c: &mut Criterion) {
    c.bench_function("insert", |b| {
        b.iter(|| {
            let mut heap = FibonacciHeap::new();
            for i in 0..1000 {
                heap.insert(black_box(i)).unwrap();
            }
        })
    });
}

fn bench_extract_min(c: &mut Criterion) {
    c.bench_function("extract_min", |b| {
        b.iter(|| {
            let mut heap = FibonacciHeap::new();
            for i in 0..1000 {
                heap.insert(i).unwrap();
            }
            for _ in 0..1000 {
                heap.extract_min();
            }
        })
    });
}

fn bench_decrease_key(c: &mut Criterion) {
    c.bench_function("decrease_key", |b| {
        b.iter(|| {
            let mut heap = FibonacciHeap::new();
            let nodes: Vec<_> = (0..1000).map(|i| heap.insert(i).unwrap()).collect();
            for node in &nodes {
                let key = node.borrow().key;
                heap.decrease_key(node, black_box(key / 2)).unwrap();
            }
        })
    });
}

fn bench_merge(c: &mut Criterion) {
    c.bench_function("merge", |b| {
        b.iter(|| {
            let mut heap1 = FibonacciHeap::new();
            for i in 0..500 {
                heap1.insert(i).unwrap();
            }

            let mut heap2 = FibonacciHeap::new();
            for i in 500..1000 {
                heap2.insert(i).unwrap();
            }

            heap1.merge(heap2);
        })
    });
}

criterion_group!(
    benches,
    bench_insert,
    bench_extract_min,
    bench_decrease_key,
    bench_merge
);
criterion_main!(benches);
