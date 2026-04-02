[![Crates.io](https://img.shields.io/crates/v/fibonacci_heap.svg)](https://crates.io/crates/fibonacci_heap)
![Rust](https://img.shields.io/badge/Rust-1.85+-orange)
![License](https://img.shields.io/badge/License-MIT-blue)
![Edition](https://img.shields.io/badge/Edition-2024-purple)

# Fibonacci Heap in Rust

A high-performance **Fibonacci Heap** implementation in Rust with **generic type support**. The **Fibonacci Heap** is a heap data structure consisting of a collection of trees that satisfies the minimum-heap property. Compared to binary heaps, it offers superior amortized time complexities for **decrease-key** and **merge**, making it particularly useful in graph algorithms like Dijkstra's and Prim's.

## Features

### Time Complexities

| Operation    | Amortized complexity |
| ------------ | -------------------- |
| Insert       | O(1)                 |
| Peek Min     | O(1)                 |
| Merge        | O(1)                 |
| Decrease Key | O(1)                 |
| Extract Min  | O(log n)             |

### Supported Operations

- **Insert** — add a new element; returns an `Rc<RefCell<Node<T>>>` handle for later updates.
- **Extract Min** — remove and return the smallest element.
- **Decrease Key** — reduce the key of a node referenced by a previously obtained handle.
- **Merge** — merge two heaps in O(1) without ID conflicts.
- **Peek Min** — read the minimum without removing it.
- **Validate Node** — O(1) check whether a handle still refers to a node in the heap.
- **Clear** — remove all elements; all previously issued handles are invalidated.

### Supported Types

Any type that implements `PartialOrd + Clone + Debug + 'static` can be used as a key.
Predefined type aliases are provided for convenience:

| Alias                                       | Key type                        |
| ------------------------------------------- | ------------------------------- |
| `FibonacciHeap`                             | `i32` (backward-compat default) |
| `FibonacciHeapI8` … `FibonacciHeapI128`     | signed integers                 |
| `FibonacciHeapU8` … `FibonacciHeapU128`     | unsigned integers               |
| `FibonacciHeapISize` / `FibonacciHeapUSize` | `isize` / `usize`               |
| `FibonacciHeapF32` / `FibonacciHeapF64`     | floats                          |
| `FibonacciHeapChar`                         | `char`                          |

### Architecture

- **Generic implementation** — a single `GenericFibonacciHeap<T>` covers all key types.
- **`NodeRef` trait** — abstraction over node handles; provides `get_key()`, `get_id()`, and `validate()`.
- **Private `key` field** — the key is read-only from outside the heap; mutation is only possible through `decrease_key`, preserving heap invariants.
- **Per-node validity flag** — each node carries a `valid: bool` that is set to `false` on extraction or `clear()`, allowing O(1) stale-handle detection without a global registry.
- **Correct `max_degree` bound** — consolidation uses `⌊log₂ n⌋ + ⌊log₂ n⌋/2 + 2 ≈ 1.5·log₂ n`, matching the theoretical Fibonacci-heap degree bound and avoiding mid-loop reallocations.

## Example Usage

### Basic Integer Heap

```rust
use fibonacci_heap::{GenericFibonacciHeap, HeapError};

fn main() -> Result<(), HeapError> {
    let mut heap = GenericFibonacciHeap::<i32>::new();

    // insert() returns a node handle that can be used for decrease_key later
    let node_a = heap.insert(42)?;
    let node_b = heap.insert(17)?;
    let _node_c = heap.insert(8)?; // current minimum

    assert_eq!(heap.peek_min(), Some(8));

    // Decrease node_b from 17 → 3; it becomes the new minimum
    heap.decrease_key(&node_b, 3)?;
    assert_eq!(heap.peek_min(), Some(3));

    // Extract minima in ascending order
    assert_eq!(heap.extract_min(), Some(3));
    assert_eq!(heap.extract_min(), Some(8));
    assert_eq!(heap.extract_min(), Some(42));
    assert_eq!(heap.extract_min(), None); // heap is empty

    Ok(())
}
```

### Reading a Node's Key

The `key` field is private to protect heap invariants. Use the `NodeRef` trait or
the `key()` method on the borrowed node:

```rust
use fibonacci_heap::{GenericFibonacciHeap, NodeRef};

let mut heap = GenericFibonacciHeap::<i32>::new();
let node = heap.insert(42).unwrap();

// Via the NodeRef trait (clones the value)
let k: i32 = node.get_key();
assert_eq!(k, 42);

// Via a direct borrow (zero-copy reference)
let k_ref: &i32 = &*node.borrow(); // borrows Node<i32>, then calls key()
// or equivalently:
assert_eq!(*node.borrow().key(), 42);
```

### Using Type Aliases

```rust
use fibonacci_heap::{FibonacciHeapI32, FibonacciHeapF64, HeapError};

fn main() -> Result<(), HeapError> {
    let mut int_heap = FibonacciHeapI32::new();
    int_heap.insert(100)?;
    int_heap.insert(50)?;
    assert_eq!(int_heap.extract_min(), Some(50));

    let mut float_heap = FibonacciHeapF64::new();
    let node = float_heap.insert(3.14)?;
    float_heap.decrease_key(&node, 2.71)?;
    assert_eq!(float_heap.extract_min(), Some(2.71));

    Ok(())
}
```

### Merging Two Heaps

Nodes from both heaps retain correct validity after a merge — there is no
ID-space collision between independently created heaps.

```rust
use fibonacci_heap::FibonacciHeapI32;

let mut heap1 = FibonacciHeapI32::new();
heap1.insert(10).unwrap();
heap1.insert(30).unwrap();

let mut heap2 = FibonacciHeapI32::new();
heap2.insert(5).unwrap();
heap2.insert(20).unwrap();

// Merge heap2 into heap1; heap2 is consumed
heap1.merge(heap2);

assert_eq!(heap1.extract_min(), Some(5));
assert_eq!(heap1.extract_min(), Some(10));
assert_eq!(heap1.extract_min(), Some(20));
assert_eq!(heap1.extract_min(), Some(30));
```

### Error Handling

```rust
use fibonacci_heap::{FibonacciHeapI32, FibonacciHeapF64, HeapError};

fn main() -> Result<(), HeapError> {
    let mut heap = FibonacciHeapI32::new();
    let node = heap.insert(10)?;

    // Attempting to increase a key returns InvalidKey
    assert_eq!(heap.decrease_key(&node, 15), Err(HeapError::InvalidKey));

    // A handle from a different heap returns NodeNotFound
    let mut other = FibonacciHeapI32::new();
    let foreign = other.insert(99)?;
    assert_eq!(heap.decrease_key(&foreign, 5), Err(HeapError::NodeNotFound));

    // An already-extracted node also returns NodeNotFound
    heap.extract_min(); // removes key=10
    assert_eq!(heap.decrease_key(&node, 1), Err(HeapError::NodeNotFound));

    // NaN as a new key returns KeyComparisonError
    let mut fheap = FibonacciHeapF64::new();
    let fnode = fheap.insert(10.0)?;
    assert_eq!(
        fheap.decrease_key(&fnode, f64::NAN),
        Err(HeapError::KeyComparisonError)
    );

    Ok(())
}
```

### Node Validity and `clear()`

```rust
use fibonacci_heap::GenericFibonacciHeap;

let mut heap = GenericFibonacciHeap::<i32>::new();
let handle = heap.insert(7).unwrap();

assert!(heap.validate_node(&handle)); // present in heap

heap.clear();

// After clear(), all previously issued handles are invalidated
assert!(!heap.validate_node(&handle));
assert!(heap.is_empty());
```

### Working with Other Types

```rust
use fibonacci_heap::{FibonacciHeapChar, FibonacciHeapF64, HeapError};

fn main() -> Result<(), HeapError> {
    let mut char_heap = FibonacciHeapChar::new();
    char_heap.insert('z')?;
    char_heap.insert('a')?;
    assert_eq!(char_heap.extract_min(), Some('a'));

    let mut float_heap = FibonacciHeapF64::new();
    float_heap.insert(2.5)?;
    float_heap.insert(1.2)?;
    assert_eq!(float_heap.extract_min(), Some(1.2));

    Ok(())
}
```

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.
