[![Crates.io](https://img.shields.io/crates/v/fibonacci_heap.svg)](https://crates.io/crates/fibonacci_heap)
![Rust](https://img.shields.io/badge/Rust-1.80+-orange)
![License](https://img.shields.io/badge/License-MIT-blue)
![Edition](https://img.shields.io/badge/Edition-2024-purple)

# Fibonacci Heap in Rust

A high-performance **Fibonacci Heap** implementation in Rust with **generic type support**. The **Fibonacci Heap** is a heap data structure consisting of a collection of trees, which is used to implement priority queues. It offers improved amortized time complexities for many operations compared to other heap structures, such as binary heaps. Its primary advantages are its efficient **decrease-key** and **merge** operations, making it particularly useful in algorithms like Dijkstra's and Prim's for shortest paths and minimum spanning trees.

## Features

### Time Complexities

- **Insertions:** O(1) amortized complexity
- **Extract Minimum:** O(log n) amortized complexity
- **Decrease Key:** O(1) amortized complexity
- **Merge two heaps:** O(1) time complexity

### Supported Operations

- **Insert:** Add a new element to the heap.
- **Extract Min:** Remove the element with the smallest value.
- **Decrease Key:** Modify the value of an element, reducing it.
- **Merge:** Merge two heaps efficiently.
- **Peek Min:** View the minimum element without removing it.

### Supported Types

The library now supports multiple data types:
- **Integer types:** `i8`, `i16`, `i32`, `i64`, `i128`, `isize`
- **Unsigned types:** `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
- **Float types:** `f32`, `f64`
- **Character type:** `char`

Type aliases are provided for convenience:
- `FibonacciHeapI32`, `FibonacciHeapF64`, `FibonacciHeapChar`, etc.

For backward compatibility, `FibonacciHeap` is a type alias for `GenericFibonacciHeap<i32>`.

### Improved Architecture

- **Generic implementation:** Single implementation for all supported types
- **NodeRef trait:** Abstraction layer over node references
- **Enhanced validation:** Thread-safe node validation
- **Better error handling:** Comprehensive `HeapError` enum with detailed error types

## Example Usage

### Basic Usage with Generic Type

```rust
use fibonacci_heap::GenericFibonacciHeap;

fn main() {
    let mut heap: GenericFibonacciHeap<i32> = GenericFibonacciHeap::new();

    // Insert elements
    let node1 = heap.insert(10).unwrap();
    let node2 = heap.insert(20).unwrap();

    // Extract the minimum element
    let min = heap.extract_min();
    println!("Extracted min: {:?}", min);  // Output: Some(10)

    // Decrease key
    heap.decrease_key(&node1, 5).unwrap();
    let min_after_decrease = heap.extract_min();
    println!("Extracted min after decrease key: {:?}", min_after_decrease);  // Output: Some(5)
}