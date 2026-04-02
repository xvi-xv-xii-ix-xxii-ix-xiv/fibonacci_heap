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
//! - Thread-safe node validation
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
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

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

/// A node in the Fibonacci Heap
#[derive(Debug)]
pub struct Node<T: HeapKey> {
    pub key: T,
    degree: usize,
    marked: bool,
    parent: Option<Weak<RefCell<Node<T>>>>,
    children: Vec<Rc<RefCell<Node<T>>>>,
    id: usize,
}

impl<T: HeapKey> Node<T> {
    /// Creates a new node with the given key and unique ID
    fn new(key: T, id: usize) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            key,
            degree: 0,
            marked: false,
            parent: None,
            children: Vec::new(),
            id,
        }))
    }
}

/// Trait for node reference abstraction
pub trait NodeRef<T: HeapKey> {
    fn validate(&self, heap: &GenericFibonacciHeap<T>) -> bool;
    fn get_key(&self) -> T;
    fn get_id(&self) -> usize;
}

impl<T: HeapKey> NodeRef<T> for Rc<RefCell<Node<T>>> {
    fn validate(&self, heap: &GenericFibonacciHeap<T>) -> bool {
        heap.validate_node(self)
    }

    fn get_key(&self) -> T {
        self.borrow().key.clone()
    }

    fn get_id(&self) -> usize {
        self.borrow().id
    }
}

/// A Fibonacci Heap data structure with generic type support
#[derive(Debug)]
pub struct GenericFibonacciHeap<T: HeapKey> {
    min: Option<Rc<RefCell<Node<T>>>>,
    root_list: Vec<Rc<RefCell<Node<T>>>>,
    node_count: usize,
    next_id: AtomicUsize,
    active_nodes: HashMap<usize, Weak<RefCell<Node<T>>>>,
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
            next_id: AtomicUsize::new(0),
            active_nodes: HashMap::new(),
        }
    }

    /// Validates if a node exists in the heap
    pub fn validate_node(&self, node: &Rc<RefCell<Node<T>>>) -> bool {
        let node_id = node.borrow().id;
        if let Some(weak_ref) = self.active_nodes.get(&node_id)
            && let Some(strong_ref) = weak_ref.upgrade()
        {
            return Rc::ptr_eq(&strong_ref, node);
        }
        false
    }

    /// Inserts a new key into the heap and returns a reference to the created node
    pub fn insert(&mut self, key: T) -> HeapResult<Rc<RefCell<Node<T>>>> {
        let id = self.next_id.fetch_add(1, AtomicOrdering::SeqCst);
        let node = Node::new(key, id);

        // Store weak reference for validation
        self.active_nodes.insert(id, Rc::downgrade(&node));

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

    /// Merges another Fibonacci Heap into this one
    pub fn merge(&mut self, other: GenericFibonacciHeap<T>) {
        // Merge root lists
        self.root_list.extend(other.root_list);
        self.node_count += other.node_count;

        // Merge active nodes
        self.active_nodes.extend(other.active_nodes);

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
        let min_key = min_node.borrow().key.clone();
        let min_id = min_node.borrow().id;

        // Remove from active nodes
        self.active_nodes.remove(&min_id);

        // Add children to root list
        let children = std::mem::take(&mut min_node.borrow_mut().children);
        for child in children {
            child.borrow_mut().parent = None;
            self.root_list.push(child);
        }

        // Remove min node from root list
        self.root_list.retain(|node| !Rc::ptr_eq(node, &min_node));
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
        let max_degree = (self.node_count as f64).log2() as usize + 2;
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
                    self.link(&existing, &current);
                } else {
                    self.link(&current, &existing);
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
        // Recompute min from the final root list. Tracking new_min incrementally
        // is incorrect: a node recorded as new_min can be subsequently linked as
        // a child of another tree, leaving self.min pointing to a non-root node.
        self.min = self.root_list.iter()
            .reduce(|acc, node| {
                match node.borrow().key.partial_cmp(&acc.borrow().key) {
                    Some(CmpOrdering::Less) => node,
                    _ => acc,
                }
            })
            .cloned();
    }

    /// Links two trees by making one a child of the other
    fn link(&mut self, child: &Rc<RefCell<Node<T>>>, parent: &Rc<RefCell<Node<T>>>) {
        // Remove child from root list
        self.root_list.retain(|node| !Rc::ptr_eq(node, child));

        // Update child's parent
        child.borrow_mut().parent = Some(Rc::downgrade(parent));
        child.borrow_mut().marked = false;

        // Add child to parent's children
        parent.borrow_mut().children.push(Rc::clone(child));
        parent.borrow_mut().degree += 1;
    }

    /// Decreases the key of a node
    pub fn decrease_key(&mut self, node: &Rc<RefCell<Node<T>>>, new_key: T) -> HeapResult<()> {
        // Validate node reference
        if !self.validate_node(node) {
            return Err(HeapError::NodeNotFound);
        }

        // Validate key comparison
        let old_key = node.borrow().key.clone();

        // Handle NaN for floats - NaN is never less than any value
        match old_key.partial_cmp(&new_key) {
            Some(CmpOrdering::Less) | Some(CmpOrdering::Equal) => {
                // New key is greater or equal, which is not allowed for decrease_key
                return Err(HeapError::InvalidKey);
            }
            Some(CmpOrdering::Greater) => {
                // Key is decreasing, which is allowed
            }
            None => {
                // Comparison failed (NaN involved)
                return Err(HeapError::KeyComparisonError);
            }
        }

        // Update key
        node.borrow_mut().key = new_key.clone();

        // Check if heap property is violated
        if let Some(parent_weak) = &node.borrow().parent
            && let Some(parent) = parent_weak.upgrade()
            && new_key < parent.borrow().key
        {
            self.cut(node, &parent);
            self.cascading_cut(&parent);
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
        // Remove node from parent's children
        parent
            .borrow_mut()
            .children
            .retain(|child| !Rc::ptr_eq(child, node));
        parent.borrow_mut().degree -= 1;

        // Add node to root list
        node.borrow_mut().parent = None;
        node.borrow_mut().marked = false;
        self.root_list.push(Rc::clone(node));
    }

    /// Performs cascading cuts on a node's ancestors if needed
    fn cascading_cut(&mut self, node: &Rc<RefCell<Node<T>>>) {
        if let Some(parent_weak) = &node.borrow().parent
            && let Some(parent) = parent_weak.upgrade()
        {
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
        self.root_list.is_empty()
    }

    /// Returns the number of nodes in the heap
    pub fn len(&self) -> usize {
        self.node_count
    }

    /// Clears the heap, removing all values
    pub fn clear(&mut self) {
        self.min = None;
        self.root_list.clear();
        self.node_count = 0;
        self.active_nodes.clear();
        self.next_id.store(0, AtomicOrdering::SeqCst);
    }
}

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
    fn test_decrease_key() {
        let mut heap = GenericFibonacciHeap::new();
        let node = heap.insert(20).unwrap();
        heap.insert(10).unwrap();

        assert_eq!(heap.extract_min(), Some(10));
        heap.decrease_key(&node, 5).unwrap();
        assert_eq!(heap.extract_min(), Some(5));
    }

    #[test]
    fn test_decrease_key_validation() {
        let mut heap = GenericFibonacciHeap::new();
        let node = heap.insert(10).unwrap();

        // Invalid key
        assert_eq!(heap.decrease_key(&node, 15), Err(HeapError::InvalidKey));

        // Valid key
        assert!(heap.decrease_key(&node, 5).is_ok());
    }

    #[test]
    fn test_decrease_key_with_nan() {
        let mut heap = GenericFibonacciHeap::<f64>::new();
        let node = heap.insert(10.0).unwrap();

        // NaN should cause comparison error
        assert_eq!(
            heap.decrease_key(&node, f64::NAN),
            Err(HeapError::KeyComparisonError)
        );
    }

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
        // Test that the original FibonacciHeap type alias works
        let mut heap: FibonacciHeap = FibonacciHeap::new();
        heap.insert(100).unwrap();
        heap.insert(50).unwrap();

        assert_eq!(heap.extract_min(), Some(50));
        assert_eq!(heap.extract_min(), Some(100));
    }

    #[test]
    fn test_triple_zero_four_pops() {
        let mut heap = GenericFibonacciHeap::<i32>::new();
        heap.insert(0).unwrap();
        heap.insert(0).unwrap();
        heap.insert(0).unwrap();
        assert_eq!(heap.extract_min(), Some(0)); // 1st
        assert_eq!(heap.extract_min(), Some(0)); // 2nd
        assert_eq!(heap.extract_min(), Some(0)); // 3rd
        assert_eq!(heap.extract_min(), None);    // 4th — heap is empty

        // Same but exhaust varying counts of equal-key nodes
        for n in 1..=16 {
            let mut h = GenericFibonacciHeap::<i32>::new();
            for _ in 0..n { h.insert(0).unwrap(); }
            for i in 0..n { assert_eq!(h.extract_min(), Some(0), "pop {} of {}", i+1, n); }
            assert_eq!(h.extract_min(), None, "extra pop after {} zeros", n);
            assert_eq!(h.len(), 0, "len after draining {} zeros", n);
        }
    }

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
}
