use crate::cmp::DefaultComparator;
use crate::skipnode::Node;
use crate::{BaseComparator, Random, RandomGenerator, K_MAX_HEIGHT};
use bumpalo_herd::Herd;
use bytes::Bytes;
use std::cmp;
use std::fmt;
use std::iter;
use std::marker::PhantomData;
use std::mem;
use std::ptr::{null_mut, NonNull};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Skip list is a data structure that allows O(log n) search complexity as well as
/// O(log n) insertion complexity within an ordered sequence of n elements.
/// Thus it can get the best of array while maintaining a linked list-like structure
/// that allows insertion- which is not possible in an array. Fast search is made
/// possible by maintaining a linked hierarchy of subsequences, with each successive
/// subsequence skipping over fewer elements than the previous one. Searching starts
/// in the sparsest subsequence until two consecutive elements have been found,
/// one smaller and one larger than or equal to the element searched for.
pub struct SkipListInner {
    head: NonNull<Node>,
    rnd: Box<dyn RandomGenerator + Send + Sync>,
    cmp: Arc<dyn BaseComparator + Send + Sync>,
    max_height: AtomicUsize,
    len: AtomicUsize,
    herd: Herd,
}

unsafe impl Send for SkipListInner {}
unsafe impl Sync for SkipListInner {}

impl SkipList {
    pub fn new(
        rnd: Box<dyn RandomGenerator + Send + Sync>,
        cmp: Arc<dyn BaseComparator + Send + Sync>,
    ) -> Self {
        let herd = Herd::new();
        SkipList {
            inner: Arc::new(SkipListInner {
                head: NonNull::from(Node::head(&herd)),
                max_height: AtomicUsize::new(1), // max height in all of the nodes except head node
                len: AtomicUsize::new(0),
                herd,
                rnd,
                cmp,
            }),
        }
    }

    pub fn new_by_cmp(cmp: Arc<dyn BaseComparator + Send + Sync>) -> Self {
        Self::new(Box::new(Random::new(0xdead_beef)), cmp)
    }

    /// Returns the number of elements in the skiplist.
    /// # Examples
    /// ```
    /// use dakv_skiplist::SkipList;
    ///
    /// let mut sl = SkipList::default();
    /// assert_eq!(sl.len(), 0);
    ///
    /// sl.insert(vec![1u8]);
    /// assert_eq!(sl.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len.load(Ordering::SeqCst)
    }

    /// Returns `true` if the skiplist is empty.
    /// # Examples
    /// ```
    /// use dakv_skiplist::SkipList;
    ///
    /// let mut sl = SkipList::default();
    /// assert!(sl.is_empty());
    ///
    /// sl.insert(vec![1u8]);
    /// assert_eq!(sl.is_empty(), false);
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn memory_size(&self) -> usize {
        (self.inner.len.load(Ordering::SeqCst) + 1) * mem::size_of::<Node>()
    }

    #[inline]
    pub fn get_max_height(&self) -> usize {
        self.inner.max_height.load(Ordering::SeqCst)
    }

    #[inline]
    pub fn set_max_height(&mut self, h: usize) {
        self.inner.max_height.store(h, Ordering::SeqCst);
    }

    /// Clear every single node and reset the head node.
    /// # Examples
    /// ```
    /// use dakv_skiplist::SkipList;
    /// let mut sl = SkipList::default();
    /// sl.insert(vec![1u8]);
    /// sl.clear();
    /// assert_eq!(sl.is_empty(), true);
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        // let new_head = Node::head(&self.inner.herd);
        self.inner.len.store(0, Ordering::SeqCst);
        // unsafe { mem::replace(&mut self.inner.head.as_ptr(), new_head) }
    }

    /// 1/4 probability
    fn random_height(&mut self) -> usize {
        static K_BRANCHING: u64 = 4;
        let mut height = 1;
        while height < K_MAX_HEIGHT && (self.inner.rnd.next() % K_BRANCHING == 0) {
            height += 1;
        }
        assert!(height > 0);
        assert!(height <= K_MAX_HEIGHT);
        height
    }

    /// Look for the node greater than or equal to key
    /// # Safety
    /// todo doc
    pub fn find(&self, key: &[u8], prev: &mut Vec<*mut Node>) -> *mut Node {
        // const pointer
        let mut const_ptr: *const Node = unsafe { self.inner.head.as_ref() };
        let mut height = self.get_max_height() - 1;
        loop {
            let next_ptr = unsafe { (*const_ptr).get_next(height) };
            // if key > next_ptr => now = next
            if self.key_is_after_node(key, next_ptr) {
                const_ptr = next_ptr as *const Node;
            } else {
                if !prev.is_empty() {
                    prev[height] = const_ptr as *mut Node;
                }
                if height == 0 {
                    return next_ptr;
                } else {
                    height -= 1;
                }
            }
        }
    }

    fn key_is_after_node(&self, key: &[u8], node: *mut Node) -> bool {
        if node.is_null() {
            false
        } else {
            self.lt(unsafe { (*node).data.as_ref() }, key)
        }
    }

    /// 1. Find the node greater than or equal to the key and return the mutable reference
    /// 2. Randomly generate level
    /// 3. Create new node
    /// 4. Insert and set forwards
    pub fn insert(&mut self, key: impl Into<Bytes>) {
        let key: Bytes = key.into();

        let mut prev = iter::repeat(null_mut()).take(K_MAX_HEIGHT).collect();
        self.find(key.as_ref(), &mut prev);
        // random height
        let height = self.random_height();
        // record all previous node that are higher than the current
        if height > self.get_max_height() {
            for node in prev.iter_mut().take(height).skip(self.get_max_height()) {
                *node = self.inner.head.as_ptr();
            }
            self.set_max_height(height);
        }
        // Accelerate memory allocation
        let n = Node::new(key, height, &self.inner.herd);
        for (i, &mut node) in prev.iter_mut().enumerate().take(height) {
            unsafe {
                let tmp = (*node).get_next(i);
                n.set_next(i, tmp);
                (*node).set_next(i, n);
            }
        }

        self.inner.len.fetch_add(1, Ordering::SeqCst);
    }

    pub fn contains(&mut self, key: &[u8]) -> bool {
        let mut prev = iter::repeat(null_mut()).take(K_MAX_HEIGHT).collect();
        let x = self.find(key, &mut prev);
        !x.is_null() && self.eq(key, unsafe { (*x).data.as_ref() })
    }

    fn eq(&self, a: &[u8], b: &[u8]) -> bool {
        self.inner.cmp.compare(a, b) == cmp::Ordering::Equal
    }

    fn lt(&self, a: &[u8], b: &[u8]) -> bool {
        self.inner.cmp.compare(a, b) == cmp::Ordering::Less
    }

    fn gte(&self, a: &[u8], b: &[u8]) -> bool {
        let r = self.inner.cmp.compare(a, b);
        r == cmp::Ordering::Greater || r == cmp::Ordering::Equal
    }

    pub fn get_head(&self) -> &Node {
        unsafe { self.inner.head.as_ref() }
    }

    #[allow(clippy::unnecessary_unwrap)]
    pub fn find_less_than(&self, key: &[u8]) -> *const Node {
        let mut x: *const Node = unsafe { mem::transmute_copy(&self.inner.head) };
        let mut level = self.get_max_height() - 1;
        unsafe {
            loop {
                let next = (*x).get_next(level);
                if next.is_null() || self.gte((*next).data.as_ref(), key) {
                    if level == 0 {
                        return x;
                    } else {
                        level -= 1;
                    }
                } else {
                    x = next;
                }
            }
        }
    }

    pub fn find_last(&self) -> *const Node {
        let mut x = self.inner.head.as_ptr() as *const Node;
        let mut level = self.get_max_height() - 1;

        loop {
            let next = unsafe { (*x).get_next(level) };
            if !next.is_null() {
                x = next;
            } else if level == 0 {
                return x;
            } else {
                level -= 1;
            }
        }
    }
}

#[derive(Clone)]
pub struct SkipList {
    inner: Arc<SkipListInner>,
}

impl From<&SkipList> for SkipList {
    fn from(sl: &SkipList) -> Self {
        SkipList {
            inner: sl.inner.clone(),
        }
    }
}

impl fmt::Display for SkipList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        unsafe {
            let mut head: *const Node = mem::transmute_copy(&self.inner.head);
            loop {
                let next = (*head).get_next(0);
                if next.is_null() {
                    break;
                } else {
                    write!(f, "{:?} ", (*next).data.as_ref())?;
                    head = next as *const Node;
                }
            }
        }
        write!(f, "]")
    }
}

impl Default for SkipList {
    #[inline]
    fn default() -> Self {
        SkipList::new(
            Box::new(Random::new(0xdead_beef)),
            Arc::new(DefaultComparator::default()),
        )
    }
}

impl<T> Extend<T> for SkipList
where
    T: Into<u8>,
{
    #[inline]
    fn extend<I: iter::IntoIterator<Item = T>>(&mut self, iterable: I) {
        let iterator = iterable.into_iter();
        for element in iterator {
            self.insert(Bytes::from(vec![element.into()]));
        }
    }
}

impl<T> iter::FromIterator<T> for SkipList
where
    T: Into<u8>,
{
    #[inline]
    fn from_iter<I>(iter: I) -> SkipList
    where
        I: iter::IntoIterator<Item = T>,
    {
        let mut sl = SkipList::default();
        sl.extend(iter);
        sl
    }
}

pub struct Iter<'a> {
    head: *const Node,
    size: usize,
    _lifetime: PhantomData<&'a Node>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            // If the lowest forward node is None, return None.
            let next = (*self.head).get_next(0);
            if !next.is_null() {
                self.head = next;
                if self.size > 0 {
                    self.size -= 1;
                }
                return Some(&&*self.head);
            }
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.size, Some(self.size))
    }
}

impl<'a> iter::IntoIterator for &'a SkipList {
    type Item = &'a Node;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        Iter {
            head: unsafe { mem::transmute_copy(&self.inner.head) },
            size: self.len(),
            _lifetime: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_basic() {
        let mut sl = SkipList::default();
        for i in 0..100u8 {
            sl.insert(Bytes::from(vec![i]));
        }
        assert_eq!(sl.len(), 100);
        for i in 0..100 {
            assert!(sl.contains(&[i]));
        }
        for i in 100..120 {
            assert_eq!(sl.contains(&[i]), false);
        }
    }

    #[test]
    fn test_clear() {
        let mut sl = SkipList::default();
        for i in 0..12 {
            sl.insert(Bytes::from(vec![i]));
        }
        sl.clear();
        assert!(sl.is_empty());
        // assert_eq!(format!("{}", sl), "[]");
    }

    #[test]
    fn test_extend() {
        let mut sl = SkipList::default();
        sl.extend(0..10);
        assert_eq!(sl.len(), 10);
        for i in 0..10 {
            assert!(sl.contains(&[i]));
        }
    }

    #[test]
    fn test_from_iter() {
        let mut sl: SkipList = (0..10).collect();
        for i in 0..10 {
            assert!(sl.contains(&[i]));
        }
    }

    #[test]
    fn test_into_iter() {
        let mut sl = SkipList::default();
        sl.extend(0..10);
        for (count, i) in (&sl).into_iter().enumerate() {
            assert_eq!(i.data.as_ref()[0], count as u8);
        }

        let mut sl = SkipList::default();
        sl.extend(vec![3, 4, 6, 7, 1, 2, 5]);
        for i in [3, 4, 6, 7, 1, 2, 5] {
            assert!(sl.contains(&[i]));
        }
    }

    #[test]
    fn test_basic_desc() {
        let mut sl = SkipList::default();
        for i in (0..12).rev() {
            sl.insert(Bytes::from(vec![i]));
        }
        assert_eq!(
            "[[0] [1] [2] [3] [4] [5] [6] [7] [8] [9] [10] [11] ]",
            format!("{}", sl)
        );

        let mut sl = SkipList::default();
        for i in [3, 4, 6, 7, 1, 2, 5] {
            sl.insert(vec![i]);
        }
        assert_eq!("[[1] [2] [3] [4] [5] [6] [7] ]", format!("{}", sl));
        assert_eq!(sl.memory_size(), 1088);
    }

    #[test]
    #[ignore]
    fn test_concurrency() {
        // todo concurrent test
        let sl = SkipList::default();
        for i in 0..12 {
            let mut csl = sl.clone();
            thread::Builder::new()
                .name(format!("thread:{}", i))
                .spawn(move || {
                    csl.insert(Bytes::from(vec![i]));
                })
                .unwrap();
        }
        assert_eq!(
            "[[0] [1] [2] [3] [4] [5] [6] [7] [8] [9] [10] [11] ]",
            format!("{}", sl)
        );
    }
}
