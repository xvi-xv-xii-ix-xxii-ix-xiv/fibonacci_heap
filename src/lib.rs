// src/lib.rs
//! A high-performance Fibonacci Heap implementation in Rust with generic type support.
//!
//! Fibonacci Heap is a collection of trees that satisfies the minimum heap property.
//! It provides efficient operations for insertion, merging, and decreasing keys,
//! making it ideal for algorithms like Dijkstra's and Prim's.
//!
//! # Features
//! - O(1) amortized time for insert and merge operations
//! - O(1) amortized time for decrease key operations
//! - O(log n) amortized time for extract minimum operations
//! - Support for multiple data types (i32, f64, char, etc.)
//! - Comprehensive error handling
//! - Node validity tracking via `valid` flag
//!
//! # Example
//! ```
//! use fibonacci_heap::{GenericFibonacciHeap, FibonacciHeapI32};
//!
//! // Using generic heap
//! let mut heap: GenericFibonacciHeap<i32> = GenericFibonacciHeap::new();
//! let node1 = heap.insert(10).unwrap();
//! let node2 = heap.insert(5).unwrap();
//! assert_eq!(heap.extract_min(), Some(5));
//!
//! heap.decrease_key(&node1, 3).unwrap();
//! assert_eq!(heap.extract_min(), Some(3));
//!
//! // Using type alias for i32
//! let mut heap_i32 = FibonacciHeapI32::new();
//! heap_i32.insert(42).unwrap();
//! ```

use std::cell::RefCell;
use std::cmp::Ordering as CmpOrdering;
use std::fmt::Debug;
use std::rc::{Rc, Weak};

/// Error types for Fibonacci Heap operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeapError {
    InvalidKey,
    NodeNotFound,
    NodeInvalid,
    HeapEmpty,
    KeyComparisonError,
}

/// Result type for heap operations
pub type HeapResult<T> = Result<T, HeapError>;

/// Trait for types that can be used as keys in Fibonacci Heap
pub trait HeapKey: PartialOrd + Clone + Debug + 'static {}
impl<T> HeapKey for T where T: PartialOrd + Clone + Debug + 'static {}

/// A node in the Fibonacci Heap.
///
/// The `key` field is private to enforce heap-property invariants;
/// use [`Node::key`] to read it and [`GenericFibonacciHeap::decrease_key`] to mutate it.
#[derive(Debug)]
pub struct Node<T: HeapKey> {
    key: T,
    degree: usize,
    marked: bool,
    /// Set to `false` when the node is extracted or the heap is cleared,
    /// so that stale [`Rc`] handles returned from [`GenericFibonacciHeap::insert`]
    /// are correctly rejected by [`GenericFibonacciHeap::validate_node`].
    valid: bool,
    parent: Option<Weak<RefCell<Node<T>>>>,
    children: Vec<Rc<RefCell<Node<T>>>>,
}

impl<T: HeapKey> Node<T> {
    fn new(key: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            key,
            degree: 0,
            marked: false,
            valid: true,
            parent: None,
            children: Vec::new(),
        }))
    }

    /// Returns a reference to this node's key.
    pub fn key(&self) -> &T {
        &self.key
    }
}

/// Trait for node reference abstraction.
pub trait NodeRef<T: HeapKey> {
    /// Returns `true` if the node is still present in `heap`.
    fn validate(&self, heap: &GenericFibonacciHeap<T>) -> bool;
    /// Clones and returns the node's current key.
    fn get_key(&self) -> T;
    /// Returns a stable unique identifier for this node (its allocation address).
    fn get_id(&self) -> usize;
}

impl<T: HeapKey> NodeRef<T> for Rc<RefCell<Node<T>>> {
    fn validate(&self, heap: &GenericFibonacciHeap<T>) -> bool {
        heap.validate_node(self)
    }

    fn get_key(&self) -> T {
        self.borrow().key.clone()
    }

    /// Returns the raw pointer of the underlying `RefCell<Node<T>>` cast to `usize`.
    /// This uniquely identifies the node for the lifetime of the `Rc`.
    fn get_id(&self) -> usize {
        self.as_ptr() as usize
    }
}

/// A Fibonacci Heap data structure with generic type support
#[derive(Debug)]
pub struct GenericFibonacciHeap<T: HeapKey> {
    min: Option<Rc<RefCell<Node<T>>>>,
    root_list: Vec<Rc<RefCell<Node<T>>>>,
    node_count: usize,
}

/// Default implementation for GenericFibonacciHeap
impl<T: HeapKey> Default for GenericFibonacciHeap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: HeapKey> GenericFibonacciHeap<T> {
    /// Creates a new empty Fibonacci Heap
    pub fn new() -> Self {
        GenericFibonacciHeap {
            min: None,
            root_list: Vec::new(),
            node_count: 0,
        }
    }

    /// Returns `true` if `node` is currently present in the heap.
    ///
    /// Validity is tracked via a `valid` flag set to `false` on extraction
    /// or on [`clear`][Self::clear], so this is an O(1) check.
    pub fn validate_node(&self, node: &Rc<RefCell<Node<T>>>) -> bool {
        node.borrow().valid
    }

    /// Inserts a new key into the heap and returns a reference to the created node
    pub fn insert(&mut self, key: T) -> HeapResult<Rc<RefCell<Node<T>>>> {
        let node = Node::new(key);

        self.root_list.push(Rc::clone(&node));
        self.node_count += 1;

        // Update minimum if needed
        match &self.min {
            Some(min) if node.borrow().key < min.borrow().key => {
                self.min = Some(Rc::clone(&node));
            }
            None => self.min = Some(Rc::clone(&node)),
            _ => (),
        }

        Ok(node)
    }

    /// Merges another Fibonacci Heap into this one.
    ///
    /// The two heaps may have been created independently; their node ID spaces
    /// do not conflict because validity is tracked per-node via a `valid` flag
    /// rather than a shared ID counter.
    pub fn merge(&mut self, other: GenericFibonacciHeap<T>) {
        // Merge root lists
        self.root_list.extend(other.root_list);
        self.node_count += other.node_count;

        // Update minimum if needed
        if let Some(other_min) = other.min {
            match &self.min {
                Some(self_min) if other_min.borrow().key < self_min.borrow().key => {
                    self.min = Some(other_min);
                }
                None => self.min = Some(other_min),
                _ => (),
            }
        }
    }

    /// Extracts the minimum value from the heap
    pub fn extract_min(&mut self) -> Option<T> {
        let min_node = self.min.take()?;

        // Invalidate the node so stale Rc handles are correctly rejected
        min_node.borrow_mut().valid = false;
        let min_key = min_node.borrow().key.clone();

        // Add children to root list, clearing their parent pointers
        let children = std::mem::take(&mut min_node.borrow_mut().children);
        for child in children {
            child.borrow_mut().parent = None;
            self.root_list.push(child);
        }

        // Remove min node from root list.
        // swap_remove is O(1) vs retain's O(n) element-shifting.
        if let Some(pos) = self.root_list.iter().position(|n| Rc::ptr_eq(n, &min_node)) {
            self.root_list.swap_remove(pos);
        }
        self.node_count -= 1;

        if self.root_list.is_empty() {
            self.min = None;
        } else {
            self.consolidate();
        }

        Some(min_key)
    }

    /// Consolidates the trees in the heap to maintain the Fibonacci Heap properties
    fn consolidate(&mut self) {
        // Upper bound on max degree: D(n) ≤ ⌊log_φ(n)⌋ + 1, where log_φ(n) ≈ 1.44·log₂(n).
        // We use ⌊log₂(n)⌋ + ⌊log₂(n)⌋/2 + 2  ≈ 1.5·log₂(n) + 2, a safe over-estimate
        // that avoids mid-loop resize in virtually all cases.
        let max_degree = if self.node_count <= 1 {
            2
        } else {
            let log2n = self.node_count.ilog2() as usize;
            log2n + log2n / 2 + 2
        };
        let mut degree_table: Vec<Option<Rc<RefCell<Node<T>>>>> = vec![None; max_degree];

        let roots = std::mem::take(&mut self.root_list);
        for root in roots {
            let mut current = root;
            let mut degree = current.borrow().degree;

            // Ensure the table is large enough for the current degree
            if degree >= degree_table.len() {
                degree_table.resize(degree + 1, None);
            }

            while let Some(existing) = degree_table[degree].take() {
                if current.borrow().key < existing.borrow().key {
                    // existing becomes child of current
                    Self::link_trees(&existing, &current);
                } else {
                    // current becomes child of existing
                    Self::link_trees(&current, &existing);
                    current = existing;
                }
                degree = current.borrow().degree;

                if degree >= degree_table.len() {
                    degree_table.resize(degree + 1, None);
                }
            }

            degree_table[degree] = Some(Rc::clone(&current));
        }

        self.root_list = degree_table.into_iter().flatten().collect();

        // Recompute min from the consolidated root list. Tracking new_min incrementally
        // is incorrect: a node recorded as new_min can be subsequently linked as
        // a child of another tree, leaving self.min pointing to a non-root node.
        self.min = self
            .root_list
            .iter()
            .reduce(|acc, node| {
                match node.borrow().key.partial_cmp(&acc.borrow().key) {
                    Some(CmpOrdering::Less) => node,
                    _ => acc,
                }
            })
            .cloned();
    }

    /// Links two trees by making `child` a subtree of `parent`.
    ///
    /// This is a static method used exclusively during consolidation, where the
    /// root list has already been taken via `mem::take`. It does **not** touch
    /// `self.root_list`, avoiding a pointless `retain` over an empty `Vec`.
    fn link_trees(child: &Rc<RefCell<Node<T>>>, parent: &Rc<RefCell<Node<T>>>) {
        child.borrow_mut().parent = Some(Rc::downgrade(parent));
        child.borrow_mut().marked = false;
        parent.borrow_mut().children.push(Rc::clone(child));
        parent.borrow_mut().degree += 1;
    }

    /// Decreases the key of a node
    pub fn decrease_key(&mut self, node: &Rc<RefCell<Node<T>>>, new_key: T) -> HeapResult<()> {
        // Validate node reference
        if !self.validate_node(node) {
            return Err(HeapError::NodeNotFound);
        }

        let old_key = node.borrow().key.clone();

        // Handle NaN for floats - NaN is never less than any value
        match old_key.partial_cmp(&new_key) {
            Some(CmpOrdering::Less) | Some(CmpOrdering::Equal) => {
                return Err(HeapError::InvalidKey);
            }
            Some(CmpOrdering::Greater) => {}
            None => {
                return Err(HeapError::KeyComparisonError);
            }
        }

        node.borrow_mut().key = new_key.clone();

        // Extract the parent Rc *before* entering the body.
        //
        // SAFETY: if we instead wrote `if let Some(pw) = &node.borrow().parent && ...`,
        // the temporary `Ref<Node<T>>` from `node.borrow()` would be kept alive through
        // the entire `if` body by Rust's temporary-lifetime extension rules.  Inside that
        // body, `cut()` calls `node.borrow_mut()`, which would panic at runtime because
        // an immutable borrow is still live.  Resolving the parent into an owned `Rc`
        // here releases the `Ref` before any mutable access occurs.
        let parent_opt = node.borrow().parent.as_ref().and_then(|pw| pw.upgrade());
        if let Some(parent) = parent_opt {
            if new_key < parent.borrow().key {
                self.cut(node, &parent);
                self.cascading_cut(&parent);
            }
        }

        // Update minimum if needed
        match &self.min {
            Some(min) if new_key < min.borrow().key => {
                self.min = Some(Rc::clone(node));
            }
            None => self.min = Some(Rc::clone(node)),
            _ => (),
        }

        Ok(())
    }

    /// Cuts a node from its parent and moves it to the root list
    fn cut(&mut self, node: &Rc<RefCell<Node<T>>>, parent: &Rc<RefCell<Node<T>>>) {
        parent
            .borrow_mut()
            .children
            .retain(|child| !Rc::ptr_eq(child, node));
        parent.borrow_mut().degree -= 1;

        node.borrow_mut().parent = None;
        node.borrow_mut().marked = false;
        self.root_list.push(Rc::clone(node));
    }

    /// Performs cascading cuts on a node's ancestors if needed
    fn cascading_cut(&mut self, node: &Rc<RefCell<Node<T>>>) {
        // Extract the parent Rc before entering the body for the same reason as in
        // decrease_key: holding `node.borrow()` across a call to `cut()` (which calls
        // `node.borrow_mut()`) would panic at runtime.
        let parent_opt = node.borrow().parent.as_ref().and_then(|pw| pw.upgrade());
        if let Some(parent) = parent_opt {
            if !node.borrow().marked {
                node.borrow_mut().marked = true;
            } else {
                self.cut(node, &parent);
                self.cascading_cut(&parent);
            }
        }
    }

    /// Returns the minimum value without removing it
    pub fn peek_min(&self) -> Option<T> {
        self.min.as_ref().map(|min| min.borrow().key.clone())
    }

    /// Checks if the heap is empty
    pub fn is_empty(&self) -> bool {
        self.node_count == 0
    }

    /// Returns the number of nodes in the heap
    pub fn len(&self) -> usize {
        self.node_count
    }

    /// Clears the heap, removing all values.
    ///
    /// All node handles previously returned by [`insert`][Self::insert] are
    /// invalidated: [`validate_node`][Self::validate_node] will return `false`
    /// for them after this call.
    pub fn clear(&mut self) {
        // Walk every tree and mark each node as invalid so that caller-held
        // Rc<Node> handles are correctly rejected by validate_node.
        for root in &self.root_list {
            Self::invalidate_tree(root);
        }
        self.min = None;
        self.root_list.clear();
        self.node_count = 0;
    }

    /// Recursively marks all nodes in a tree as invalid.
    fn invalidate_tree(node: &Rc<RefCell<Node<T>>>) {
        // Collect children first to avoid holding a borrow while recursing.
        let children: Vec<_> = node.borrow().children.iter().cloned().collect();
        node.borrow_mut().valid = false;
        for child in children {
            Self::invalidate_tree(&child);
        }
    }

    /// Deletes an arbitrary node from the heap.
    ///
    /// Implemented as: cut node to root → force it to be `self.min` → `extract_min`.
    /// This avoids the need for a −∞ sentinel while preserving all heap invariants.
    ///
    /// # Errors
    /// Returns [`HeapError::NodeNotFound`] if `node` is not currently in the heap.
    ///
    /// # Complexity
    /// O(log n) amortized — same as `extract_min`.
    pub fn delete(&mut self, node: &Rc<RefCell<Node<T>>>) -> HeapResult<()> {
        if !self.validate_node(node) {
            return Err(HeapError::NodeNotFound);
        }

        // If the node has a parent, cut it to the root list first and
        // perform cascading cuts on its ancestors.
        let parent_opt = node.borrow().parent.as_ref().and_then(|pw| pw.upgrade());
        if let Some(parent) = parent_opt {
            self.cut(node, &parent);
            self.cascading_cut(&parent);
        }

        // Force this node to be the minimum so that extract_min removes it.
        // After extract_min, consolidate re-derives the real minimum from scratch.
        self.min = Some(Rc::clone(node));
        self.extract_min();

        Ok(())
    }

    /// Returns a cloned `Rc` handle to the minimum node without removing it.
    ///
    /// Unlike [`peek_min`][Self::peek_min], the returned handle can be passed
    /// to [`decrease_key`][Self::decrease_key] or [`delete`][Self::delete].
    pub fn peek_min_node(&self) -> Option<Rc<RefCell<Node<T>>>> {
        self.min.as_ref().map(Rc::clone)
    }

    /// Consumes the heap and returns all keys in ascending (sorted) order.
    ///
    /// Equivalent to repeatedly calling `extract_min` until empty.
    /// Complexity: O(n log n).
    pub fn into_sorted_vec(mut self) -> Vec<T> {
        let mut result = Vec::with_capacity(self.node_count);
        while let Some(val) = self.extract_min() {
            result.push(val);
        }
        result
    }
}

// ── Iteration ────────────────────────────────────────────────────────────────

/// Consuming iterator that yields keys in ascending order.
///
/// Produced by [`GenericFibonacciHeap::into_iter`].
pub struct IntoIter<T: HeapKey> {
    heap: GenericFibonacciHeap<T>,
}

impl<T: HeapKey> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.heap.extract_min()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.heap.len();
        (n, Some(n))
    }
}

impl<T: HeapKey> ExactSizeIterator for IntoIter<T> {}

impl<T: HeapKey> IntoIterator for GenericFibonacciHeap<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { heap: self }
    }
}

impl<T: HeapKey> FromIterator<T> for GenericFibonacciHeap<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut heap = Self::new();
        for item in iter {
            // insert only returns Err for NaN-like values; generic T is always valid
            let _ = heap.insert(item);
        }
        heap
    }
}

impl<T: HeapKey> Extend<T> for GenericFibonacciHeap<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            let _ = self.insert(item);
        }
    }
}

// ── Type aliases ─────────────────────────────────────────────────────────────

/// Type aliases for backward compatibility and convenience
pub type FibonacciHeap = GenericFibonacciHeap<i32>;
pub type FibonacciHeapI8 = GenericFibonacciHeap<i8>;
pub type FibonacciHeapI16 = GenericFibonacciHeap<i16>;
pub type FibonacciHeapI32 = GenericFibonacciHeap<i32>;
pub type FibonacciHeapI64 = GenericFibonacciHeap<i64>;
pub type FibonacciHeapI128 = GenericFibonacciHeap<i128>;
pub type FibonacciHeapISize = GenericFibonacciHeap<isize>;
pub type FibonacciHeapU8 = GenericFibonacciHeap<u8>;
pub type FibonacciHeapU16 = GenericFibonacciHeap<u16>;
pub type FibonacciHeapU32 = GenericFibonacciHeap<u32>;
pub type FibonacciHeapU64 = GenericFibonacciHeap<u64>;
pub type FibonacciHeapU128 = GenericFibonacciHeap<u128>;
pub type FibonacciHeapUSize = GenericFibonacciHeap<usize>;
pub type FibonacciHeapF32 = GenericFibonacciHeap<f32>;
pub type FibonacciHeapF64 = GenericFibonacciHeap<f64>;
pub type FibonacciHeapChar = GenericFibonacciHeap<char>;

#[cfg(test)]
mod tests {
    use rand::RngExt;
    use super::*;

    // ── insert / extract_min / peek_min ──────────────────────────────────────

    #[test]
    fn test_basic_operations_i32() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        assert!(heap.is_empty());

        heap.insert(10).unwrap();
        heap.insert(5).unwrap();
        assert_eq!(heap.len(), 2);

        assert_eq!(heap.extract_min(), Some(5));
        assert_eq!(heap.extract_min(), Some(10));
        assert!(heap.is_empty());
    }

    #[test]
    fn test_basic_operations_f64() {
        let mut heap = GenericFibonacciHeap::<f64>::new();
        assert!(heap.is_empty());

        heap.insert(10.5).unwrap();
        heap.insert(5.2).unwrap();
        assert_eq!(heap.len(), 2);

        assert_eq!(heap.extract_min(), Some(5.2));
        assert_eq!(heap.extract_min(), Some(10.5));
        assert!(heap.is_empty());
    }

    #[test]
    fn test_basic_operations_char() {
        let mut heap = GenericFibonacciHeap::<char>::new();
        assert!(heap.is_empty());

        heap.insert('z').unwrap();
        heap.insert('a').unwrap();
        assert_eq!(heap.len(), 2);

        assert_eq!(heap.extract_min(), Some('a'));
        assert_eq!(heap.extract_min(), Some('z'));
        assert!(heap.is_empty());
    }

    #[test]
    fn test_single_element() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        heap.insert(42).unwrap();
        assert_eq!(heap.len(), 1);
        assert_eq!(heap.peek_min(), Some(42));
        assert_eq!(heap.extract_min(), Some(42));
        assert!(heap.is_empty());
        assert_eq!(heap.extract_min(), None);
    }

    #[test]
    fn test_peek_min_does_not_remove() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        heap.insert(3).unwrap();
        heap.insert(1).unwrap();
        heap.insert(2).unwrap();

        assert_eq!(heap.peek_min(), Some(1));
        assert_eq!(heap.len(), 3); // unchanged
        assert_eq!(heap.peek_min(), Some(1)); // stable
    }

    #[test]
    fn test_peek_min_empty() {
        let heap = GenericFibonacciHeap::<i32>::new();
        assert_eq!(heap.peek_min(), None);
    }

    #[test]
    fn test_extract_min_sorted_order() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        let values = [50, 10, 30, 20, 40];
        for v in values {
            heap.insert(v).unwrap();
        }
        let mut sorted = values.to_vec();
        sorted.sort_unstable();
        for expected in sorted {
            assert_eq!(heap.extract_min(), Some(expected));
        }
    }

    // ── peek_min_node ────────────────────────────────────────────────────────

    #[test]
    fn test_peek_min_node_empty() {
        let heap = GenericFibonacciHeap::<i32>::new();
        assert!(heap.peek_min_node().is_none());
    }

    #[test]
    fn test_peek_min_node_returns_min() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        heap.insert(99).unwrap();
        let min_node_handle = heap.insert(1).unwrap();
        heap.insert(50).unwrap();

        let peeked = heap.peek_min_node().unwrap();
        // The peeked handle must point to the same allocation as the min we inserted.
        assert!(Rc::ptr_eq(&peeked, &min_node_handle));
        assert_eq!(peeked.borrow().key(), &1);
        assert_eq!(heap.len(), 3); // no element was removed
    }

    #[test]
    fn test_peek_min_node_handle_usable_for_decrease_key() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        heap.insert(10).unwrap();
        heap.insert(5).unwrap();

        let handle = heap.peek_min_node().unwrap();
        // decrease_key with same value should return InvalidKey (not ≤ check)
        assert_eq!(heap.decrease_key(&handle, 5), Err(HeapError::InvalidKey));
        // valid decrease
        heap.decrease_key(&handle, 1).unwrap();
        assert_eq!(heap.peek_min(), Some(1));
    }

    // ── merge ────────────────────────────────────────────────────────────────

    #[test]
    fn test_merge() {
        let mut heap1 = GenericFibonacciHeap::new();
        heap1.insert(10).unwrap();
        heap1.insert(20).unwrap();

        let mut heap2 = GenericFibonacciHeap::new();
        heap2.insert(5).unwrap();
        heap2.insert(15).unwrap();

        heap1.merge(heap2);
        assert_eq!(heap1.len(), 4);
        assert_eq!(heap1.extract_min(), Some(5));
    }

    #[test]
    fn test_merge_with_empty_heap() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        heap.insert(7).unwrap();

        heap.merge(GenericFibonacciHeap::new());
        assert_eq!(heap.len(), 1);
        assert_eq!(heap.extract_min(), Some(7));
    }

    #[test]
    fn test_merge_into_empty_heap() {
        let mut heap1 = GenericFibonacciHeap::<i32>::new();
        let mut heap2 = GenericFibonacciHeap::new();
        heap2.insert(3).unwrap();
        heap2.insert(1).unwrap();

        heap1.merge(heap2);
        assert_eq!(heap1.len(), 2);
        assert_eq!(heap1.extract_min(), Some(1));
    }

    #[test]
    fn test_merge_no_id_collision() {
        let mut heap_a = GenericFibonacciHeap::<i32>::new();
        let a0 = heap_a.insert(100).unwrap();
        let a1 = heap_a.insert(200).unwrap();

        let mut heap_b = GenericFibonacciHeap::<i32>::new();
        let b0 = heap_b.insert(50).unwrap();
        let b1 = heap_b.insert(150).unwrap();

        heap_a.merge(heap_b);

        assert!(heap_a.validate_node(&a0));
        assert!(heap_a.validate_node(&a1));
        assert!(heap_a.validate_node(&b0));
        assert!(heap_a.validate_node(&b1));

        assert_eq!(heap_a.extract_min(), Some(50));
        assert!(!heap_a.validate_node(&b0));
        assert!(heap_a.validate_node(&a0));
        assert!(heap_a.validate_node(&a1));
        assert!(heap_a.validate_node(&b1));
    }

    // ── decrease_key ─────────────────────────────────────────────────────────

    #[test]
    fn test_decrease_key() {
        let mut heap = GenericFibonacciHeap::new();
        let node = heap.insert(20).unwrap();
        heap.insert(10).unwrap();

        assert_eq!(heap.extract_min(), Some(10));
        heap.decrease_key(&node, 5).unwrap();
        assert_eq!(heap.extract_min(), Some(5));
    }

    /// Verifies that decrease_key correctly cuts a node from its parent.
    #[test]
    fn test_decrease_key_with_parent_cut() {
        let mut heap = GenericFibonacciHeap::<i32>::new();

        for i in 1..=8 {
            heap.insert(i * 10).unwrap();
        }
        assert_eq!(heap.extract_min(), Some(10));

        let mut heap2 = GenericFibonacciHeap::<i32>::new();
        let nodes: Vec<_> = (1..=8).map(|i| heap2.insert(i * 10).unwrap()).collect();
        heap2.extract_min();

        heap2.decrease_key(&nodes[7], 5).unwrap(); // 80 → 5
        assert_eq!(heap2.extract_min(), Some(5));
    }

    #[test]
    fn test_decrease_key_becomes_new_min() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        heap.insert(1).unwrap();
        let node = heap.insert(100).unwrap();

        heap.decrease_key(&node, 0).unwrap();
        assert_eq!(heap.peek_min(), Some(0));
        assert_eq!(heap.extract_min(), Some(0));
        assert_eq!(heap.extract_min(), Some(1));
    }

    /// Trigger multi-level cascading cuts: mark two ancestors, then cut a third.
    #[test]
    fn test_decrease_key_cascading_cut() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        // Build a 16-node heap; consolidate creates trees deep enough for cascading.
        let nodes: Vec<_> = (1..=16).map(|i| heap.insert(i * 10).unwrap()).collect();
        heap.extract_min(); // consolidate

        // First cut: marks an ancestor
        heap.decrease_key(&nodes[14], 5).unwrap(); // 150 → 5
        heap.extract_min(); // remove 5

        // Second cut in the same ancestor: triggers cascading
        heap.decrease_key(&nodes[13], 4).unwrap(); // 140 → 4
        assert_eq!(heap.extract_min(), Some(4));
    }

    #[test]
    fn test_decrease_key_validation() {
        let mut heap = GenericFibonacciHeap::new();
        let node = heap.insert(10).unwrap();

        assert_eq!(heap.decrease_key(&node, 15), Err(HeapError::InvalidKey));
        assert_eq!(heap.decrease_key(&node, 10), Err(HeapError::InvalidKey));
        assert!(heap.decrease_key(&node, 5).is_ok());
    }

    #[test]
    fn test_decrease_key_with_nan() {
        let mut heap = GenericFibonacciHeap::<f64>::new();
        let node = heap.insert(10.0).unwrap();

        assert_eq!(
            heap.decrease_key(&node, f64::NAN),
            Err(HeapError::KeyComparisonError)
        );
    }

    // ── delete ───────────────────────────────────────────────────────────────

    #[test]
    fn test_delete_min_node() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        let min = heap.insert(1).unwrap();
        heap.insert(2).unwrap();
        heap.insert(3).unwrap();

        heap.delete(&min).unwrap();
        assert_eq!(heap.len(), 2);
        assert!(!heap.validate_node(&min));
        assert_eq!(heap.extract_min(), Some(2));
        assert_eq!(heap.extract_min(), Some(3));
        assert_eq!(heap.extract_min(), None);
    }

    #[test]
    fn test_delete_non_min_root_node() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        heap.insert(1).unwrap();
        let node = heap.insert(99).unwrap();
        heap.insert(50).unwrap();

        heap.delete(&node).unwrap();
        assert_eq!(heap.len(), 2);
        assert!(!heap.validate_node(&node));

        let mut out = Vec::new();
        while let Some(v) = heap.extract_min() {
            out.push(v);
        }
        assert_eq!(out, vec![1, 50]);
    }

    /// Delete a node that sits deep inside a consolidated tree (has a parent).
    #[test]
    fn test_delete_deep_node() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        let nodes: Vec<_> = (1..=8).map(|i| heap.insert(i * 10).unwrap()).collect();
        heap.extract_min(); // consolidate: nodes[1..] now have parents

        // nodes[6] = key 70 — should be somewhere inside the tree
        heap.delete(&nodes[6]).unwrap();
        assert!(!heap.validate_node(&nodes[6]));

        // The remaining 6 keys are 20,30,40,50,60,80; extract all and verify sorted.
        let mut out = Vec::new();
        while let Some(v) = heap.extract_min() {
            out.push(v);
        }
        assert_eq!(out, vec![20, 30, 40, 50, 60, 80]);
    }

    #[test]
    fn test_delete_only_element() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        let node = heap.insert(42).unwrap();

        heap.delete(&node).unwrap();
        assert!(heap.is_empty());
        assert_eq!(heap.extract_min(), None);
    }

    #[test]
    fn test_delete_invalid_node_returns_error() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        let node = heap.insert(10).unwrap();
        heap.extract_min(); // node is now invalid

        assert_eq!(heap.delete(&node), Err(HeapError::NodeNotFound));
    }

    #[test]
    fn test_delete_all_nodes_sequentially() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        let nodes: Vec<_> = (1..=5).map(|i| heap.insert(i).unwrap()).collect();

        for node in &nodes {
            heap.delete(node).unwrap();
        }
        assert!(heap.is_empty());
        assert_eq!(heap.extract_min(), None);
    }

    #[test]
    fn test_delete_then_insert_and_extract() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        let nodes: Vec<_> = (1..=4).map(|i| heap.insert(i * 10).unwrap()).collect();
        heap.extract_min(); // consolidate

        heap.delete(&nodes[2]).unwrap(); // delete key=30
        heap.insert(5).unwrap();

        let mut out = Vec::new();
        while let Some(v) = heap.extract_min() {
            out.push(v);
        }
        assert_eq!(out, vec![5, 20, 40]);
    }

    // ── clear ────────────────────────────────────────────────────────────────

    #[test]
    fn test_clear_invalidates_nodes() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        let node = heap.insert(42).unwrap();
        assert!(heap.validate_node(&node));

        heap.clear();

        assert!(!heap.validate_node(&node));
        assert_eq!(heap.len(), 0);
        assert!(heap.is_empty());
    }

    #[test]
    fn test_clear_then_reuse() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        heap.insert(1).unwrap();
        heap.insert(2).unwrap();
        heap.clear();

        heap.insert(99).unwrap();
        assert_eq!(heap.len(), 1);
        assert_eq!(heap.extract_min(), Some(99));
    }

    // ── node validity ────────────────────────────────────────────────────────

    #[test]
    fn test_extracted_node_is_invalid() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        let node = heap.insert(1).unwrap();
        heap.insert(2).unwrap();

        assert!(heap.validate_node(&node));
        heap.extract_min();
        assert!(!heap.validate_node(&node));

        assert_eq!(heap.decrease_key(&node, 0), Err(HeapError::NodeNotFound));
    }

    #[test]
    fn test_node_key_accessor() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        let node = heap.insert(42).unwrap();
        assert_eq!(*node.borrow().key(), 42);
    }

    #[test]
    fn test_node_ref_trait() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        let node = heap.insert(7).unwrap();

        assert_eq!(node.get_key(), 7);
        assert!(node.validate(&heap));

        let id_before = node.get_id();
        // id is stable — pointer must not change between calls
        assert_eq!(node.get_id(), id_before);
    }

    // ── into_sorted_vec ──────────────────────────────────────────────────────

    #[test]
    fn test_into_sorted_vec_basic() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        for v in [3, 1, 4, 1, 5, 9, 2, 6] {
            heap.insert(v).unwrap();
        }
        let sorted = heap.into_sorted_vec();
        let mut expected = vec![3, 1, 4, 1, 5, 9, 2, 6];
        expected.sort_unstable();
        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_into_sorted_vec_empty() {
        let heap = GenericFibonacciHeap::<i32>::new();
        assert_eq!(heap.into_sorted_vec(), Vec::<i32>::new());
    }

    #[test]
    fn test_into_sorted_vec_single() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        heap.insert(42).unwrap();
        assert_eq!(heap.into_sorted_vec(), vec![42]);
    }

    // ── IntoIterator ─────────────────────────────────────────────────────────

    #[test]
    fn test_into_iter_sorted_order() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        for v in [5, 2, 8, 1, 9, 3] {
            heap.insert(v).unwrap();
        }
        let result: Vec<_> = heap.into_iter().collect();
        assert_eq!(result, vec![1, 2, 3, 5, 8, 9]);
    }

    #[test]
    fn test_into_iter_size_hint() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        heap.insert(10).unwrap();
        heap.insert(20).unwrap();
        heap.insert(30).unwrap();

        let mut iter = heap.into_iter();
        assert_eq!(iter.size_hint(), (3, Some(3)));
        iter.next();
        assert_eq!(iter.size_hint(), (2, Some(2)));
    }

    #[test]
    fn test_into_iter_empty_heap() {
        let heap = GenericFibonacciHeap::<i32>::new();
        let result: Vec<_> = heap.into_iter().collect();
        assert!(result.is_empty());
    }

    #[test]
    fn test_exact_size_iterator() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        heap.insert(1).unwrap();
        heap.insert(2).unwrap();

        let iter = heap.into_iter();
        assert_eq!(iter.len(), 2);
    }

    // ── FromIterator ─────────────────────────────────────────────────────────

    #[test]
    fn test_from_iter() {
        let heap: GenericFibonacciHeap<i32> = [4, 2, 7, 1, 5].into_iter().collect();
        assert_eq!(heap.len(), 5);
        assert_eq!(heap.peek_min(), Some(1));
    }

    #[test]
    fn test_from_iter_empty() {
        let heap: GenericFibonacciHeap<i32> = std::iter::empty().collect();
        assert!(heap.is_empty());
    }

    #[test]
    fn test_from_iter_sorted_output() {
        let data = vec![9i32, 3, 6, 1, 8, 2, 7, 4, 5];
        let heap: GenericFibonacciHeap<i32> = data.iter().copied().collect();
        let sorted: Vec<_> = heap.into_iter().collect();
        assert_eq!(sorted, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    // ── Extend ───────────────────────────────────────────────────────────────

    #[test]
    fn test_extend() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        heap.insert(10).unwrap();
        heap.extend([3, 7, 1, 9]);

        assert_eq!(heap.len(), 5);
        assert_eq!(heap.extract_min(), Some(1));
    }

    #[test]
    fn test_extend_empty_iter() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        heap.insert(5).unwrap();
        heap.extend(std::iter::empty::<i32>());
        assert_eq!(heap.len(), 1);
    }

    #[test]
    fn test_extend_then_drain() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        heap.extend(0..10);
        let sorted: Vec<_> = heap.into_iter().collect();
        assert_eq!(sorted, (0..10).collect::<Vec<_>>());
    }

    // ── type aliases ─────────────────────────────────────────────────────────

    #[test]
    fn test_type_aliases() {
        let mut heap_i32: FibonacciHeapI32 = FibonacciHeapI32::new();
        heap_i32.insert(42).unwrap();
        assert_eq!(heap_i32.extract_min(), Some(42));

        let mut heap_f64: FibonacciHeapF64 = FibonacciHeapF64::new();
        heap_f64.insert(3.14).unwrap();
        assert_eq!(heap_f64.extract_min(), Some(3.14));

        let mut heap_char: FibonacciHeapChar = FibonacciHeapChar::new();
        heap_char.insert('x').unwrap();
        assert_eq!(heap_char.extract_min(), Some('x'));
    }

    #[test]
    fn test_backward_compatibility() {
        let mut heap: FibonacciHeap = FibonacciHeap::new();
        heap.insert(100).unwrap();
        heap.insert(50).unwrap();

        assert_eq!(heap.extract_min(), Some(50));
        assert_eq!(heap.extract_min(), Some(100));
    }

    // ── edge cases ───────────────────────────────────────────────────────────

    #[test]
    fn test_triple_zero_four_pops() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        heap.insert(0).unwrap();
        heap.insert(0).unwrap();
        heap.insert(0).unwrap();
        assert_eq!(heap.extract_min(), Some(0));
        assert_eq!(heap.extract_min(), Some(0));
        assert_eq!(heap.extract_min(), Some(0));
        assert_eq!(heap.extract_min(), None);

        for n in 1..=16 {
            let mut h = GenericFibonacciHeap::<i32>::new();
            for _ in 0..n {
                h.insert(0).unwrap();
            }
            for i in 0..n {
                assert_eq!(h.extract_min(), Some(0), "pop {} of {}", i + 1, n);
            }
            assert_eq!(h.extract_min(), None, "extra pop after {} zeros", n);
            assert_eq!(h.len(), 0, "len after draining {} zeros", n);
        }
    }

    #[test]
    fn test_negative_keys() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        heap.insert(-5).unwrap();
        heap.insert(-1).unwrap();
        heap.insert(-10).unwrap();
        assert_eq!(heap.extract_min(), Some(-10));
        assert_eq!(heap.extract_min(), Some(-5));
        assert_eq!(heap.extract_min(), Some(-1));
    }

    #[test]
    fn test_default_trait() {
        let heap: GenericFibonacciHeap<i32> = Default::default();
        assert!(heap.is_empty());
    }

    /// Heap-sort correctness: i32 values including duplicates.
    #[test]
    fn test_heap_sort_with_duplicates() {
        let mut input = vec![3i32, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5];
        let heap: GenericFibonacciHeap<i32> = input.iter().copied().collect();
        let sorted: Vec<_> = heap.into_iter().collect();

        input.sort_unstable();
        assert_eq!(sorted, input);
    }

    // ── stress tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_consolidate_no_panic_stress() {
        let mut rng = rand::rng();

        for _ in 0..200 {
            let mut heap = GenericFibonacciHeap::<i32>::new();
            let n = rng.random_range(10_000..50_000);

            for _ in 0..n {
                heap.insert(rng.random_range(0..1_000_000)).unwrap();
            }

            for _ in 0..(n / 2) {
                heap.extract_min();
            }

            for _ in 0..1000 {
                if rng.random_bool(0.5) {
                    heap.insert(rng.random_range(0..1_000_000)).unwrap();
                } else {
                    heap.extract_min();
                }
            }

            assert!(heap.len() <= n + 1000);
        }
    }

    /// Mix of insert / decrease_key / delete under random workload.
    #[test]
    fn test_mixed_operations_stress() {
        let mut rng = rand::rng();
        let n = 1_000usize;

        let mut heap = GenericFibonacciHeap::<i32>::new();
        let mut nodes: Vec<Rc<RefCell<Node<i32>>>> = Vec::with_capacity(n);
        let mut live_count = 0usize;

        for _ in 0..n {
            let v = rng.random_range(0..10_000);
            nodes.push(heap.insert(v).unwrap());
            live_count += 1;
        }

        for _ in 0..(n / 2) {
            let idx = rng.random_range(0..nodes.len());
            if !heap.validate_node(&nodes[idx]) {
                continue;
            }
            let cur = nodes[idx].borrow().key().clone();
            let delta = rng.random_range(1..=cur.max(1));
            let _ = heap.decrease_key(&nodes[idx], cur - delta);
        }

        for _ in 0..(n / 4) {
            let idx = rng.random_range(0..nodes.len());
            if heap.validate_node(&nodes[idx]) {
                heap.delete(&nodes[idx]).unwrap();
                live_count -= 1;
            }
        }

        assert_eq!(heap.len(), live_count);

        // Extract remaining elements and verify they are in sorted order.
        let extracted: Vec<_> = heap.into_iter().collect();
        for w in extracted.windows(2) {
            assert!(w[0] <= w[1], "not sorted: {} > {}", w[0], w[1]);
        }
        assert_eq!(extracted.len(), live_count);
    }
}
