# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2026-04-02

### Added

- `delete(node)` — remove an arbitrary node by handle in O(log n) amortized.
  Implemented without a −∞ sentinel: the node is cut to the root list and then
  extracted via the existing `extract_min` path, preserving all heap invariants.
- `peek_min_node()` — returns a cloned `Rc` handle to the minimum node so it
  can be passed directly to `decrease_key` or `delete` without a separate lookup.
- `into_sorted_vec()` — consumes the heap and returns all keys in ascending order.
- `IntoIterator` — consuming iterator that yields keys in sorted order;
  implements `ExactSizeIterator` with an accurate `size_hint`.
- `FromIterator<T>` — build a heap from any iterator via `.collect()`.
- `Extend<T>` — bulk-insert from an iterator via `heap.extend(values)`.

### Tests

- Test suite expanded from 12 to 53 unit tests covering: `delete` (7 cases
  including deep nodes, cascading cuts, and stress), `peek_min_node` (3),
  `into_sorted_vec` (3), `IntoIterator` (4), `FromIterator` (3), `Extend` (3),
  `Default` trait, negative keys, duplicate keys, heap-sort correctness,
  and a mixed-operations stress test (insert + decrease\_key + delete).

## [1.0.0] - 2025-01-01

### Added

- Initial release: `GenericFibonacciHeap<T>` with `insert`, `extract_min`,
  `decrease_key`, `merge`, `peek_min`, `is_empty`, `len`, `clear`,
  `validate_node`.
- Type aliases for all primitive numeric types and `char`.
- `NodeRef` trait for node handle abstraction.
- `Default` implementation.

[1.1.0]: https://github.com/xvi-xv-xii-ix-xxii-ix-xiv/fibonacci_heap/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/xvi-xv-xii-ix-xxii-ix-xiv/fibonacci_heap/releases/tag/v1.0.0
