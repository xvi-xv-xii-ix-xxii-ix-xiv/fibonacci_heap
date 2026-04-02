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

The library supports multiple data types:

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

### Basic Integer Heap

```rust
use fibonacci_heap::{GenericFibonacciHeap, HeapError};

fn main() -> Result<(), HeapError> {
    // Create a new heap for i32
    let mut heap = GenericFibonacciHeap::<i32>::new();

    // Insert some values; each insertion returns a node reference for later updates
    let node_a = heap.insert(42)?;
    let node_b = heap.insert(17)?;
    let node_c = heap.insert(8)?; // minimum so far

    // Peek at the minimum without removing it
    assert_eq!(heap.peek_min(), Some(8));

    // Decrease the key of node_b from 17 to 3
    heap.decrease_key(&node_b, 3)?;
    assert_eq!(heap.peek_min(), Some(3));

    // Extract minima in order
    assert_eq!(heap.extract_min(), Some(3)); // node_b
    assert_eq!(heap.extract_min(), Some(8)); // node_c
    assert_eq!(heap.extract_min(), Some(42)); // node_a
    assert_eq!(heap.extract_min(), None); // heap empty

    Ok(())
}
```

### Using Type Aliases for Convenience

```rust
use fibonacci_heap::{FibonacciHeapI32, FibonacciHeapF64, HeapError};

fn main() -> Result<(), HeapError> {
    // Type alias for i32 heap
    let mut int_heap = FibonacciHeapI32::new();
    int_heap.insert(100)?;
    int_heap.insert(50)?;
    assert_eq!(int_heap.extract_min(), Some(50));

    // Type alias for f64 heap
    let mut float_heap = FibonacciHeapF64::new();
    let node = float_heap.insert(3.14)?;
    float_heap.decrease_key(&node, 2.71)?;
    assert_eq!(float_heap.extract_min(), Some(2.71));

    Ok(())
}
```

### Merging Two Heaps

```rust
use fibonacci_heap::FibonacciHeapI32;
use fibonacci_heap::HeapError;

fn main() -> Result<(), HeapError> {
    let mut heap1 = FibonacciHeapI32::new();
    heap1.insert(10)?;
    heap1.insert(30)?;

    let mut heap2 = FibonacciHeapI32::new();
    heap2.insert(5)?;
    heap2.insert(20)?;

    // Merge heap2 into heap1 (heap2 becomes empty)
    heap1.merge(heap2);

    assert_eq!(heap1.extract_min(), Some(5));
    assert_eq!(heap1.extract_min(), Some(10));
    assert_eq!(heap1.extract_min(), Some(20));
    assert_eq!(heap1.extract_min(), Some(30));

    Ok(())
}
```

### Error Handling

The `decrease_key` operation can fail for several reasons:

```rust
use fibonacci_heap::{FibonacciHeapI32, FibonacciHeapF64, HeapError};

fn main() -> Result<(), HeapError> {
    let mut heap = FibonacciHeapI32::new();
    let node = heap.insert(10)?;

    // Attempt to increase the key (not allowed) – returns InvalidKey
    assert_eq!(heap.decrease_key(&node, 15), Err(HeapError::InvalidKey));

    // Using an invalid node reference (e.g., from another heap) returns NodeNotFound
    let other_heap = FibonacciHeapI32::new();
    let foreign_node = other_heap.insert(99)?;
    assert_eq!(heap.decrease_key(&foreign_node, 5), Err(HeapError::NodeNotFound));

    // For floating-point types, using NaN may cause a KeyComparisonError
    let mut float_heap = FibonacciHeapF64::new();
    let float_node = float_heap.insert(10.0)?;
    assert_eq!(float_heap.decrease_key(&float_node, f64::NAN), Err(HeapError::KeyComparisonError));

    Ok(())
}
```

### Working with Other Types (char, f64)

```rust
use fibonacci_heap::{FibonacciHeapChar, FibonacciHeapF64, HeapError};

fn main() -> Result<(), HeapError> {
    // Character heap
    let mut char_heap = FibonacciHeapChar::new();
    char_heap.insert('z')?;
    char_heap.insert('a')?;
    assert_eq!(char_heap.extract_min(), Some('a'));

    // Floating-point heap (f64)
    let mut float_heap = FibonacciHeapF64::new();
    float_heap.insert(2.5)?;
    float_heap.insert(1.2)?;
    assert_eq!(float_heap.extract_min(), Some(1.2));

    Ok(())
}
```

All operations return `HeapResult<T>` which is a `Result<T, HeapError>`, allowing you to handle errors gracefully.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

