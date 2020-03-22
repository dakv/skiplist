use std::fmt;
use std::iter;
use std::mem;
use std::ptr::NonNull;
use std::ptr::null_mut;
use crate::skipnode::Node;
use crate::K_MAX_HEIGHT;

pub trait RandomGenerator {
    fn next(&mut self) -> usize;
}

/// Skip list is a data structure that allows O(log n) search complexity as well as
/// O(log n) insertion complexity within an ordered sequence of n elements.
/// Thus it can get the best of array while maintaining a linked list-like structure
/// that allows insertion- which is not possible in an array. Fast search is made
/// possible by maintaining a linked hierarchy of subsequences, with each successive
/// subsequence skipping over fewer elements than the previous one. Searching starts
/// in the sparsest subsequence until two consecutive elements have been found,
/// one smaller and one larger than or equal to the element searched for.
pub struct SkipList<T> {
    head: Box<Node<T>>,
    rnd: Box<dyn RandomGenerator>,
    max_height: usize,
    len: usize,
}

impl<T: PartialOrd + Clone + fmt::Debug> SkipList<T> {
    pub fn new(rnd: Box<dyn RandomGenerator>) -> Self {
        SkipList {
            head: box Node::head(),
            rnd,
            max_height: 1, // max height in all of the nodes except head node
            len: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn get_max_height(&self) -> usize {
        self.max_height
    }

    #[inline]
    pub fn set_max_height(&mut self, h: usize) {
        self.max_height = h;
    }

    /// 1/4 probability
    fn random_height(&mut self) -> usize {
        static K_BRANCHING: usize = 4;
        let mut height = 1;
        while height < K_MAX_HEIGHT && (self.rnd.next() % K_BRANCHING == 0) {
            height += 1;
        }
        assert!(height > 0);
        assert!(height <= K_MAX_HEIGHT);
        height
    }

    /// Look for the node greater than or equal to key
    pub unsafe fn find(&mut self, key: &T, prev: &mut Vec<*mut Node<T>>) -> *mut Node<T> {
        // const pointer
        let mut const_ptr: *const Node<T> = self.head.as_ref();
        let mut height = self.get_max_height() - 1;
        loop {
            // todo get_next()
            let next_ptr: *mut Node<T> = match (*const_ptr).get_next(height) {
                None => { null_mut() }
                Some(mut v) => {
                    v
                }
            };
            // if key > next_ptr => now = next
            if key_is_after_node(key, next_ptr) {
                const_ptr = next_ptr as *const Node<T>;
            } else {
                prev[height] = const_ptr as *mut Node<T>;
                if height == 0 {
                    return next_ptr;
                } else {
                    height -= 1;
                }
            }
        }
    }

    /// 1. Find the node greater than or equal to the key and return the mutable reference
    /// 2. Randomly generate level
    /// 3. Create new node
    /// 4. Insert and set forwards
    pub fn insert(&mut self, key: &T) {
        unsafe {
            let mut prev = iter::repeat(null_mut()).take(K_MAX_HEIGHT).collect();
            self.find(key, &mut prev);
            // random height
            let height = self.random_height();
            // record all previous node that are higher than the current
            if height > self.get_max_height() {
                for i in self.get_max_height()..height {
                    prev[i] = self.head.as_mut();
                }
                // todo concurrent support
                self.set_max_height(height);
            }
            let x = Box::new(Node::new(key.clone()));
            let mut x = Box::into_raw_non_null(x);
            for i in 0..height {
                x.as_mut().set_next(i, (*prev[i]).get_mut_next(i));
                (*prev[i]).set_next(i, Some(x));
            }
        }
    }
}

impl<T: PartialOrd + Clone + fmt::Display> fmt::Display for SkipList<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            write!(f, "head");
            let mut head: NonNull<Node<T>> = mem::transmute_copy(&self.head);
            loop {
                if let Some(next) = head.as_ref().forward[0] {
                    if let Some(value) = &next.as_ref().data {
                        write!(f, " -> {}", value)?
                    }
                    head = next;
                } else {
                    break;
                }
            }
        }
        write!(f, "\n")
    }
}

struct Random {
    seed_: usize
}

impl Random {
    pub fn new(s: usize) -> Random {
        let mut seed_ = s & 0x7fffffffusize;
        if seed_ == 0 || seed_ == 2147483647 {
            seed_ = 1;
        }
        Random { seed_ }
    }
}

impl RandomGenerator for Random {
    fn next(&mut self) -> usize {
        static M: usize = 2147483647; // 2^31-1
        static A: usize = 16807; // bits 14, 8, 7, 5, 2, 1, 0
        let product = self.seed_ * A;
        self.seed_ = (product >> 31) + (product & M);

        if self.seed_ > M {
            self.seed_ -= M;
        }
        self.seed_
    }
}

impl<T: PartialOrd + Clone + fmt::Debug> Default for SkipList<T> {
    #[inline]
    fn default() -> Self {
        Self::new(box Random::new(0xdeadbeef))
    }
}

fn key_is_after_node<T: PartialOrd + fmt::Debug>(key: &T, node: *mut Node<T>) -> bool {
    if node.is_null() {
        false
    } else {
        unsafe {
            match &(*node).data {
                None => panic!("Data can not be None"),
                Some(v) => {
                    let c = v < key;
                    c
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut sl: SkipList<usize> = SkipList::default();
        for i in 0..12 {
            sl.insert(&i);
        }
        assert_eq!("head -> 0 -> 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8 -> 9 -> 10 -> 11\n", format!("{}", sl));
    }

    #[test]
    fn test_basic_desc() {
        let mut sl: SkipList<usize> = SkipList::default();
        for i in (0..12).rev() {
            sl.insert(&i);
        }
        assert_eq!("head -> 0 -> 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8 -> 9 -> 10 -> 11\n", format!("{}", sl));
    }

    #[test]
    fn test_basic_desc2() {
        let mut sl: SkipList<usize> = SkipList::default();
        for i in vec![3, 4, 6, 7, 1, 2, 5] {
            sl.insert(&i);
        }
        assert_eq!("head -> 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7\n", format!("{}", sl));
    }
}