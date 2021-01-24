use crate::skipnode::Node;
use crate::K_MAX_HEIGHT;
use std::fmt;
use std::iter;
use std::marker::PhantomData;
use std::mem;
use std::ptr::null_mut;
use std::ptr::NonNull;

pub trait RandomGenerator {
    fn next(&mut self) -> u64;
    // Returns a uniformly distributed value in the range [0..n-1]
    // REQUIRES: n > 0
    fn uniform(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    // Randomly returns true ~"1/n" of the time, and false otherwise.
    // REQUIRES: n > 0
    fn one_in(&mut self, n: u64) -> bool {
        (self.next() % n) == 0
    }
    // Skewed: pick "base" uniformly from range [0,max_log] and then
    // return "base" random bits.  The effect is to pick a number in the
    // range [0,2^max_log-1] with exponential bias towards smaller numbers.
    fn skewed(&mut self, max_log: u64) -> u64 {
        let tmp = 1 << self.uniform(max_log + 1);
        self.uniform(tmp)
    }
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

impl<T: PartialOrd + PartialEq + Clone> SkipList<T> {
    pub fn new(rnd: Box<dyn RandomGenerator>) -> Self {
        SkipList {
            head: Box::new(Node::head()),
            rnd,
            max_height: 1, // max height in all of the nodes except head node
            len: 0,
        }
    }

    /// Returns the number of elements in the skiplist.
    /// # Examples
    /// ```
    /// use dakv_skiplist::SkipList;
    ///
    /// let mut sl = SkipList::default();
    /// assert_eq!(sl.len(), 0);
    ///
    /// sl.insert(&1);
    /// assert_eq!(sl.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the skiplist is empty.
    /// # Examples
    /// ```
    /// use dakv_skiplist::SkipList;
    ///
    /// let mut sl = SkipList::default();
    /// assert!(sl.is_empty());
    ///
    /// sl.insert(&1);
    /// assert_eq!(sl.is_empty(), false);
    /// ```
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

    /// Clear every single node and reset the head node.
    /// # Examples
    /// ```
    /// use dakv_skiplist::SkipList;
    /// let mut sl = SkipList::default();
    /// sl.insert(&1);
    /// sl.clear();
    /// ```
    #[inline]
    pub fn clear(&mut self) -> Box<Node<T>> {
        let new_head = Box::new(Node::head());
        self.len = 0;
        mem::replace(&mut self.head, new_head)
    }

    /// 1/4 probability
    fn random_height(&mut self) -> usize {
        static K_BRANCHING: u64 = 4;
        let mut height = 1;
        while height < K_MAX_HEIGHT && (self.rnd.next() % K_BRANCHING == 0) {
            height += 1;
        }
        assert!(height > 0);
        assert!(height <= K_MAX_HEIGHT);
        height
    }

    /// Look for the node greater than or equal to key
    /// # Safety
    /// todo doc
    pub unsafe fn find(&self, key: &T, prev: &mut Vec<*mut Node<T>>) -> *mut Node<T> {
        // const pointer
        let mut const_ptr: *const Node<T> = self.head.as_ref();
        let mut height = self.get_max_height() - 1;
        loop {
            let next_ptr: *mut Node<T> = match (*const_ptr).get_next(height) {
                None => null_mut(),
                Some(v) => v,
            };
            // if key > next_ptr => now = next
            if key_is_after_node(key, next_ptr) {
                const_ptr = next_ptr as *const Node<T>;
            } else {
                if !prev.is_empty() {
                    prev[height] = const_ptr as *mut Node<T>;
                }
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
                for node in prev.iter_mut().take(height).skip(self.get_max_height()) {
                    *node = self.head.as_mut();
                }
                // todo concurrent support
                self.set_max_height(height);
            }
            let x = Box::new(Node::new(key.clone()));
            let mut x = NonNull::from(Box::leak(x));
            //            let mut x = Box::into_raw_non_null(x);
            for (i, &mut node) in prev.iter_mut().enumerate().take(height) {
                x.as_mut().set_next(i, (*node).get_mut_next(i));
                (*node).set_next(i, Some(x));
            }

            self.len += 1;
        }
    }

    pub fn contains(&mut self, key: &T) -> bool {
        unsafe {
            let mut prev = iter::repeat(null_mut()).take(K_MAX_HEIGHT).collect();
            let x = self.find(key, &mut prev);
            !x.is_null() && key == (*x).data.as_ref().unwrap()
        }
    }

    pub fn find_less_than(&self, key: &T) -> *const Node<T> {
        let mut x: *const Node<T> = unsafe { mem::transmute_copy(&self.head) };
        let mut level = self.max_height - 1;
        loop {
            let next = unsafe { (*x).get_next(level) };
            if next.is_none() || unsafe { (*next.unwrap()).data.as_ref().unwrap() >= key } {
                if level == 0 {
                    return x;
                } else {
                    level -= 1;
                }
            } else {
                x = next.unwrap();
            }
        }
    }

    pub fn find_last(&self) -> *const Node<T> {
        let mut x = &*self.head as *const Node<T>;
        let mut level = self.get_max_height() - 1;

        loop {
            let next = unsafe { (*x).get_next(level) };
            if let Some(v) = next {
                x = v;
            } else if level == 0 {
                return x;
            } else {
                level -= 1;
            }
        }
    }

    pub fn get_head(&self) -> &Node<T> {
        &self.head
    }
}

impl<T: PartialOrd + Clone + fmt::Display> fmt::Display for SkipList<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            write!(f, "[")?;
            let mut head: NonNull<Node<T>> = mem::transmute_copy(&self.head);
            while let Some(next) = head.as_ref().forward[0] {
                if let Some(value) = &next.as_ref().data {
                    write!(f, "{} -> ", value)?
                }
                head = next;
            }
        }
        write!(f, "]")
    }
}

pub struct Random {
    seed_: u64,
}

impl Random {
    pub fn new(s: u64) -> Random {
        let mut seed_ = s & 0x7fff_ffff_u64;
        if seed_ == 0 || seed_ == 2_147_483_647 {
            seed_ = 1;
        }
        Random { seed_ }
    }
}

impl RandomGenerator for Random {
    fn next(&mut self) -> u64 {
        static M: u64 = 2_147_483_647; // 2^31-1
        static A: u64 = 16807; // bits 14, 8, 7, 5, 2, 1, 0
        let product = self.seed_.wrapping_mul(A);
        self.seed_ = (product >> 31) + (product & M);

        if self.seed_ > M {
            self.seed_ -= M;
        }
        self.seed_
    }
}

impl<T: PartialOrd + Clone> Default for SkipList<T> {
    #[inline]
    fn default() -> Self {
        Self::new(Box::new(Random::new(0xdead_beef)))
    }
}

impl<T> Extend<T> for SkipList<T>
where
    T: PartialEq + PartialOrd + Clone,
{
    #[inline]
    fn extend<I: iter::IntoIterator<Item = T>>(&mut self, iterable: I) {
        let iterator = iterable.into_iter();
        for element in iterator {
            self.insert(&element);
        }
    }
}

impl<T> iter::FromIterator<T> for SkipList<T>
where
    T: PartialOrd + PartialEq + Clone,
{
    #[inline]
    fn from_iter<I>(iter: I) -> SkipList<T>
    where
        I: iter::IntoIterator<Item = T>,
    {
        let mut sl = SkipList::default();
        sl.extend(iter);
        sl
    }
}

pub struct Iter<'a, T: 'a> {
    head: *const Node<T>,
    size: usize,
    _lifetime: PhantomData<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        unsafe {
            // If the lowest forward node is None, return None.
            (*self.head).forward[0]?;
            if let Some(next) = (*self.head).forward[0] {
                self.head = next.as_ptr() as *const Node<T>;
                if self.size > 0 {
                    self.size -= 1;
                }
                return (*self.head).data.as_ref();
            }
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.size, Some(self.size))
    }
}

impl<'a, T> iter::IntoIterator for &'a SkipList<T>
where
    T: PartialOrd + PartialEq + Clone,
{
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        Iter {
            head: unsafe { mem::transmute_copy(&self.head) },
            size: self.len(),
            _lifetime: PhantomData,
        }
    }
}

fn key_is_after_node<T: PartialOrd>(key: &T, node: *mut Node<T>) -> bool {
    if node.is_null() {
        false
    } else {
        unsafe {
            match &(*node).data {
                None => panic!("Data can not be None"),
                Some(v) => v < key,
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
        for i in 0..100 {
            sl.insert(&i);
        }
        assert_eq!(sl.len(), 100);
        for i in 0..100 {
            assert!(sl.contains(&i));
        }
        for i in 100..120 {
            assert_eq!(sl.contains(&i), false);
        }
    }

    #[test]
    fn test_clear() {
        let mut sl = SkipList::default();
        for i in 0..12 {
            sl.insert(&i);
        }
        sl.clear();
        assert!(sl.is_empty());
        assert_eq!(format!("{}", sl), "[]");
    }

    #[test]
    fn test_extend() {
        let mut sl: SkipList<usize> = SkipList::default();
        sl.extend(0..10);
        assert_eq!(sl.len(), 10);
        for i in 0..10 {
            assert!(sl.contains(&i));
        }
    }

    #[test]
    fn test_from_iter() {
        let mut sl: SkipList<i32> = (0..10).collect();
        for i in 0..10 {
            assert!(sl.contains(&i));
        }
    }

    #[test]
    fn test_into_iter() {
        let mut sl = SkipList::default();
        sl.extend(0..10);
        for (count, i) in (&sl).into_iter().enumerate() {
            assert_eq!(i, &count);
        }

        let mut sl = SkipList::default();
        let data = vec![3, 4, 6, 7, 1, 2, 5];
        sl.extend(&data);
        for i in &data {
            assert!(sl.contains(&i));
        }
    }

    #[test]
    fn test_basic_desc() {
        let mut sl: SkipList<usize> = SkipList::default();
        for i in (0..12).rev() {
            sl.insert(&i);
        }
        assert_eq!(
            "[0 -> 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8 -> 9 -> 10 -> 11 -> ]",
            format!("{}", sl)
        );

        let mut sl: SkipList<usize> = SkipList::default();
        for i in &[3, 4, 6, 7, 1, 2, 5] {
            sl.insert(&i);
        }
        assert_eq!("[1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> ]", format!("{}", sl));
    }
}
