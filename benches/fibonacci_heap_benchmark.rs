// benches/fibonacci_heap_benchmark.rs
use criterion::{Criterion, criterion_group, criterion_main};
use fibonacci_heap::{FibonacciHeapI32, GenericFibonacciHeap, NodeRef};
use std::hint::black_box;

fn bench_insert(c: &mut Criterion) {
    c.bench_function("insert_i32", |b| {
        b.iter(|| {
            let mut heap = FibonacciHeapI32::new();
            for i in 0..1000 {
                heap.insert(black_box(i)).unwrap();
            }
        })
    });
}

fn bench_extract_min(c: &mut Criterion) {
    c.bench_function("extract_min_i32", |b| {
        b.iter(|| {
            let mut heap = FibonacciHeapI32::new();
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
    c.bench_function("decrease_key_i32", |b| {
        b.iter(|| {
            let mut heap = FibonacciHeapI32::new();
            let nodes: Vec<_> = (1..=1000).map(|i| heap.insert(i * 10).unwrap()).collect();
            for node in &nodes {
                let key = node.get_key();
                heap.decrease_key(node, black_box(key - 5)).unwrap();
            }
        })
    });
}

fn bench_merge(c: &mut Criterion) {
    c.bench_function("merge_i32", |b| {
        b.iter(|| {
            let mut heap1 = FibonacciHeapI32::new();
            for i in 0..500 {
                heap1.insert(i).unwrap();
            }

            let mut heap2 = FibonacciHeapI32::new();
            for i in 500..1000 {
                heap2.insert(i).unwrap();
            }

            heap1.merge(heap2);
        })
    });
}

// Benchmarks for other types
fn bench_insert_f64(c: &mut Criterion) {
    c.bench_function("insert_f64", |b| {
        b.iter(|| {
            let mut heap: GenericFibonacciHeap<f64> = GenericFibonacciHeap::new();
            for i in 1..=1000 {
                heap.insert(black_box(i as f64)).unwrap();
            }
        })
    });
}

fn bench_decrease_key_f64(c: &mut Criterion) {
    c.bench_function("decrease_key_f64", |b| {
        b.iter(|| {
            let mut heap: GenericFibonacciHeap<f64> = GenericFibonacciHeap::new();
            let nodes: Vec<_> = (1..=100)
                .map(|i| heap.insert(i as f64 * 10.0).unwrap())
                .collect();
            for node in &nodes {
                let key = node.get_key();
                heap.decrease_key(node, black_box(key - 2.5)).unwrap();
            }
        })
    });
}

// This stresses the consolidate() function and previously triggered an index-out-of-bounds panic.
fn bench_1m_insert_1k_search(c: &mut Criterion) {
    c.bench_function("1m_insert_1k_search", |b| {
        b.iter(|| {
            let mut heap = FibonacciHeapI32::new();
            // Insert 1,000,000 elements with a pattern to create varied trees
            for i in 0..1_000_000 {
                heap.insert(black_box(i % 10_000)).unwrap();
            }
            // Extract 1,000 minima – this triggers consolidate many times
            for _ in 0..1000 {
                black_box(heap.extract_min());
            }
        })
    });
}

criterion_group!(
    benches,
    bench_insert,
    bench_extract_min,
    bench_decrease_key,
    bench_decrease_key_f64,
    bench_merge,
    bench_insert_f64,
    bench_1m_insert_1k_search
);
criterion_main!(benches);
