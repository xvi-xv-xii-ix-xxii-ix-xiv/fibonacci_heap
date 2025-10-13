//! A high-performance Fibonacci Heap implementation in Rust.
//!
//! Fibonacci Heap is a collection of trees that satisfies the minimum heap property.
//! It provides efficient operations for insertion, merging, and decreasing keys,
//! making it ideal for algorithms like Dijkstra's and Prim's.
//!
//! # Features
//! - O(1) amortized time for insert and merge operations
//! - O(1) amortized time for decrease key operations
//! - O(log n) amortized time for extract minimum operations
//! - Comprehensive error handling
//! - Optimized implementations for [iu]{8,16,32,64,128,size}, f32, f64, and char:
//!   - FibbonachiHeap (defaults to i32)
//!   - FibbonachiHeapi8
//!   - FibbonachiHeapchar
//!   - etc.: FibbonachiHeap{{type}}
//!
//! # Note: This fork isn't thread-safe and neither is the upstream, in spite of superfluous use of
//! atomic usize.
//!
//!
//! # Create a new empty Fibonacci Heap
//!
//! ```
//! use fibonacci_heap::FibonacciHeap;
//! let heap = FibonacciHeap::new();
//! assert!(heap.is_empty());
//! ```
//!
//! # Insert a new key into the heap and returns a reference to the created node
//!
//! ```
//! use fibonacci_heap::FibonacciHeap;
//! let mut heap = FibonacciHeap::new();
//! let node = heap.insert(42).unwrap();
//! ```
//!
//!
//! # Merge another Fibonacci Heap into this one
//!
//! ```
//! use fibonacci_heap::FibonacciHeap;
//!
//! let mut heap1 = FibonacciHeap::new();
//! heap1.insert(10).unwrap();
//!
//! let mut heap2 = FibonacciHeap::new();
//! heap2.insert(5).unwrap();
//!
//! heap1.merge(heap2);
//! assert_eq!(heap1.extract_min(), Some(5));
//! ```
//!
//!
//! # Extract the minimum value from the heap
//!
//! ```
//! use fibonacci_heap::FibonacciHeap;
//!
//! let mut heap = FibonacciHeap::new();
//! heap.insert(10).unwrap();
//! heap.insert(5).unwrap();
//!
//! assert_eq!(heap.extract_min(), Some(5));
//! ```
//!
//!
//! # Return minimum value without removing it
//!
//! ```
//! use fibonacci_heap::FibonacciHeap;
//!
//! let mut heap = FibonacciHeap::new();
//! heap.insert(10).unwrap();
//! heap.insert(5).unwrap();
//!
//! assert_eq!(heap.peek_min(), Some(5));
//! ```
//!
//! # Check if the heap is empty
//!
//! ```
//! use fibonacci_heap::FibonacciHeap;
//!
//! let heap = FibonacciHeap::new();
//! assert!(heap.is_empty());
//! ```
//!
//!
//! # Return number of nodes in the heap
//!
//! ```
//! use fibonacci_heap::FibonacciHeap;
//!
//! let mut heap = FibonacciHeap::new();
//! heap.insert(10).unwrap();
//! heap.insert(20).unwrap();
//!
//! assert_eq!(heap.len(), 2);
//! ```
//!
//!
//! # Clear the heap, removing all values
//!
//! ```
//! use fibonacci_heap::FibonacciHeap;
//!
//! let mut heap = FibonacciHeap::new();
//! heap.insert(10).unwrap();
//! heap.clear();
//!
//! assert!(heap.is_empty());
//! ```
//!
//!
//! # Decrease key of a node
//!
//! ```
//! use fibonacci_heap::FibonacciHeap;
//!
//! let mut heap = FibonacciHeap::new();
//! let node = heap.insert(20).unwrap();
//! heap.insert(10).unwrap();
//!
//! assert_eq!(heap.extract_min(), Some(10));
//! heap.decrease_key(&node, 5).unwrap();
//! assert_eq!(heap.extract_min(), Some(5));
//! ```
//!
//!
//! # Create a Fibonacci Heap of type char
//!
//! ```
//! use fibonacci_heap::FibonacciHeapchar;
//! let mut heap = FibonacciHeapchar::new();
//! heap.insert('a');
//! heap.insert('c');
//! heap.insert('d');
//! assert_eq!(heap.len(), 3);
//! ```
//!
use paste::paste;
use std::{
    cell::RefCell,
    cmp::Ordering,
    collections::HashMap,
    mem::take,
    rc::{Rc, Weak},
};

/// Error types for Fibonacci Heap operations
#[derive(Debug, PartialEq)]
pub enum HeapError {
    InvalidKey,
    NodeNotFound,
    HeapEmpty,
}

pub type Result<T> = std::result::Result<T, HeapError>;

pub trait NodeRef {
    type RefCell;
    type Weak;
    type Rc;
    type OptionalRc;
    type OptionalWeak;
}

macro_rules! impl_for_types {
    ($($t:ty),+, $(,)?) => {
        $(
            paste! {
                /// A node in the Fibonacci Heap
                #[derive(Debug)]
                pub struct [<Node $t>] {
                    pub key: $t,
                    id: usize, // Unique identifier for node validation
                    degree: usize,
                    marked: bool,
                    parent: <Self as NodeRef>::OptionalWeak,
                    children: Vec<<Self as NodeRef>::Rc>,
                }

                impl PartialEq for [<Node $t>] {
                    fn eq(&self, other: &Self) -> bool {
                        self.key == other.key
                    }
                }

                impl PartialOrd for [<Node $t>] {
                    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                        self.key.partial_cmp(&other.key)
                    }
                }

                impl [<Node $t>] {
                    /// Creates a new node with the given key and unique ID
                    fn new(key: $t, id: usize) -> <Self as NodeRef>::Rc {
                        Rc::new(
                            RefCell::new(
                                Self {
                                    key,
                                    id,
                                    degree: 0,
                                    marked: false,
                                    parent: None,
                                    children: vec![],
                                }
                            )
                        )
                    }
                }

                impl NodeRef for [<Node $t>] {
                    type RefCell = RefCell<Self>;
                    type Weak = Weak<Self::RefCell>;
                    type Rc = Rc<Self::RefCell>;
                    type OptionalRc = Option<Self::Rc>;
                    type OptionalWeak = Option<Self::Weak>;
                }

                /// A Fibonacci Heap data structure
                #[derive(Debug, Default)]
                pub struct [<FibonacciHeap $t>] {
                    min: <[<Node $t>] as NodeRef>::OptionalRc,
                    root_list: Vec<<[<Node $t>] as NodeRef>::Rc>,
                    node_count: usize,
                    next_id: usize,
                    active_nodes: HashMap<usize, <[<Node $t>] as NodeRef>::Weak>,
                }

                impl [<FibonacciHeap $t>] {
                    /// Create a new empty Fibonacci Heap
                    #[inline]
                    pub fn new() -> Self {
                        Default::default()
                    }

                    /// Insert a new key into the heap and returns a reference to the created node
                    pub fn insert(&mut self, key: $t) -> Result<<[<Node $t>] as NodeRef>::Rc> {
                        self.next_id += 1;
                        let id = self.next_id;
                        let node = [<Node $t>]::new(key, id);

                        // Store weak reference for validation
                        self.active_nodes.insert(id, Rc::downgrade(&node));

                        self.root_list.push(Rc::clone(&node));
                        self.node_count += 1;

                        // Update minimum if needed
                        if self.peek_min().map_or(true, |min| min > key)  {
                            self.min = Some(Rc::clone(&node));
                        }

                        Ok(node)
                    }

                    /// Merge another Fibonacci Heap into this one
                    pub fn merge(&mut self, other: Self) {
                        // Merge root lists
                        self.root_list.extend(other.root_list);
                        self.node_count += other.node_count;

                        // Merge active nodes
                        self.active_nodes.extend(other.active_nodes);

                        // Update minimum if needed
                        if other.min < self.min {
                            self.min = other.min;
                        }
                    }

                    /// Decrease key of a node
                    pub fn extract_min(&mut self) -> Option<$t> {
                        let min_node = self.min.take()?;

                        // Remove from active nodes
                        self.active_nodes.remove(&min_node.borrow().id);

                        // Add children to root list
                        let children = take(&mut min_node.borrow_mut().children);
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

                        Some(min_node.borrow().key)
                    }

                    /// Consolidates trees in the heap to maintain the Fibonacci Heap properties
                    #[inline]
                    fn consolidate(&mut self) {
                        // Calculate maximum possible degree based on node count
                        let max_degree = (self.node_count as f64).log2() as usize + 2;
                        let mut degree_table: Vec<<[<Node $t>] as NodeRef>::OptionalRc> = vec![None; max_degree];
                        let mut new_min = None;

                        // Process all root nodes
                        for mut current in take(&mut self.root_list) {
                            let mut degree = current.borrow().degree;

                            // Combine trees with same degree
                            while let Some(existing) = degree_table[degree].take() {
                                if current < existing {
                                    self.link(existing, &current);
                                } else {
                                    self.link(current, &existing);
                                    current = existing;
                                }
                                degree = current.borrow().degree;

                                // Extend degree table if needed
                                if degree >= degree_table.len() {
                                    degree_table.resize(degree + 1, None);
                                }
                            }

                            degree_table[degree] = Some(current.clone());

                            // Track new minimum
                            if new_min
                                .as_ref()
                                .is_none_or(|min: &<[<Node $t>] as NodeRef>::Rc| &current < min)
                            {
                                new_min = Some(current);
                            }
                        }

                        // Rebuild root list from degree table
                        self.root_list = degree_table.into_iter().flatten().collect();
                        self.min = new_min;
                    }

                    /// Links two trees by making one a child of the other
                    #[inline]
                    fn link(&mut self, child: <[<Node $t>] as NodeRef>::Rc, parent: &<[<Node $t>] as NodeRef>::Rc) {
                        // Remove child from root list
                        self.root_list.retain(|node| !Rc::ptr_eq(node, &child));

                        // Update child's parent
                        child.borrow_mut().parent = Some(Rc::downgrade(parent));
                        child.borrow_mut().marked = false;

                        // Add child to parent's children
                        parent.borrow_mut().children.push(child);
                        parent.borrow_mut().degree += 1;
                    }

                    /// Decrease key of a node
                    pub fn decrease_key(
                        &mut self,
                        node: &<[<Node $t>] as NodeRef>::Rc,
                        new_key: $t,
                    ) -> Result<()> {
                        if !self.active_nodes.contains_key(&node.borrow().id) { // Validate node reference
                            return Err(HeapError::NodeNotFound);
                        }

                        if new_key > node.borrow().key { // Validate key
                            return Err(HeapError::InvalidKey);
                        }

                        // Update key
                        node.borrow_mut().key = new_key;

                        // Check if heap property is violated
                        if let Some(parent) = &node.borrow().parent.as_ref().map(Weak::upgrade).flatten()
                            && node < parent
                        {
                            self.cut(node, parent);
                            self.cascading_cut(parent);
                        }

                        // Update minimum if needed
                        if self.peek_min().map_or(true, |min| min > new_key) {
                            self.min = Some(Rc::clone(node));
                        }
                        Ok(())
                    }

                    /// Cuts a node from its parent and moves it to the root list
                    #[inline]
                    fn cut(&mut self, node: &<[<Node $t>] as NodeRef>::Rc, parent: &<[<Node $t>] as NodeRef>::Rc) {
                        // Remove node from parent's children
                        parent.borrow_mut()
                            .children
                            .retain(|child| !Rc::ptr_eq(child, node));
                        parent.borrow_mut().degree -= 1;

                        // Add node to root list
                        node.borrow_mut().parent = None;
                        node.borrow_mut().marked = false;
                        self.root_list.push(Rc::clone(node));
                    }

                    /// Performs cascading cuts on a node's ancestors if needed
                    #[inline]
                    fn cascading_cut(&mut self, node: &<[<Node $t>] as NodeRef>::Rc) {
                        loop {
                            if let Some(parent) = node.borrow().parent.as_ref().map(Weak::upgrade).flatten() {
                                if node.borrow().marked {
                                    self.cut(node, &parent);
                                    continue;
                                } else {
                                    node.borrow_mut().marked = true;
                                }
                            }
                            return;
                        }
                    }

                    /// Return minimum value without removing it
                    #[inline]
                    pub fn peek_min(&self) -> Option<$t> {
                        self.min.as_ref().map(|min| min.borrow().key)
                    }

                    /// Check if the heap is empty
                    #[inline]
                    pub const fn is_empty(&self) -> bool {
                        self.node_count == 0
                    }

                    /// Return number of nodes in the heap
                    #[inline]
                    pub const fn len(&self) -> usize {
                        self.node_count
                    }

                    /// Clear heap, removing all values
                    #[inline]
                    pub fn clear(&mut self) {
                        *self = Self::default();
                    }
                }
            }
        )+
    }
}
impl_for_types!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64, char,
);

/// Alias for FibonacciHeap with i32 keys
pub type FibonacciHeap = FibonacciHeapi32;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut heap = FibonacciHeap::new();
        assert!(heap.is_empty());

        heap.insert(10).unwrap();
        heap.insert(5).unwrap();
        assert_eq!(heap.len(), 2);

        assert_eq!(heap.extract_min(), Some(5));
        assert_eq!(heap.extract_min(), Some(10));
        assert!(heap.is_empty());
    }

    #[test]
    fn test_merge() {
        let mut heap1 = FibonacciHeap::new();
        heap1.insert(10).unwrap();
        heap1.insert(20).unwrap();

        let mut heap2 = FibonacciHeap::new();
        heap2.insert(5).unwrap();
        heap2.insert(15).unwrap();

        heap1.merge(heap2);
        assert_eq!(heap1.len(), 4);
        assert_eq!(heap1.extract_min(), Some(5));
    }

    #[test]
    fn test_decrease_key() {
        let mut heap = FibonacciHeap::new();
        let node = heap.insert(20).unwrap();
        heap.insert(10).unwrap();

        assert_eq!(heap.extract_min(), Some(10));
        heap.decrease_key(&node, 5).unwrap();
        assert_eq!(heap.extract_min(), Some(5));
    }

    #[test]
    fn test_decrease_key_validation() {
        let mut heap = FibonacciHeap::new();
        let node = heap.insert(10).unwrap();

        // Invalid key
        assert_eq!(heap.decrease_key(&node, 15), Err(HeapError::InvalidKey));

        // Valid key
        assert!(heap.decrease_key(&node, 5).is_ok());
    }

    #[test]
    fn test_decrease_key_validation_for_u64() {
        let mut heap = FibonacciHeapu64::new();
        let node = heap.insert(10).unwrap();

        // Invalid key
        assert_eq!(heap.decrease_key(&node, 15), Err(HeapError::InvalidKey));

        // Valid key
        assert!(heap.decrease_key(&node, 5).is_ok());
    }
}
