use crate::skipnode::Node;
use crate::{Arena, BaseComparator, RandomGenerator, SkipList, K_MAX_HEIGHT};
use std::iter;
use std::ptr::{null, null_mut};

pub struct SkipListIter<R, C, A>
where
    R: RandomGenerator,
    C: BaseComparator,
    A: Arena,
{
    list: SkipList<R, C, A>,
    node: *const Node,
}

impl<R, C, A> SkipListIter<R, C, A>
where
    R: RandomGenerator,
    C: BaseComparator,
    A: Arena,
{
    pub fn new(list: SkipList<R, C, A>) -> Self {
        Self { list, node: null() }
    }

    pub fn valid(&self) -> bool {
        !self.node.is_null()
    }

    pub fn seek_to_first(&mut self) {
        let n = self.list.get_head();
        self.node = n.get_next(0);
    }

    pub fn seek_to_last(&mut self) {
        self.node = self.list.find_last();
        if self.node == self.list.get_head() {
            self.node = null();
        }
    }

    /// For mem table to seek entry.
    pub fn seek(&mut self, target: &[u8]) {
        let mut prev = iter::repeat(null_mut()).take(K_MAX_HEIGHT).collect();
        self.node = self.list.find(target, &mut prev);
    }

    pub fn next(&mut self) {
        assert!(self.valid());
        self.node = unsafe { (*self.node).get_next(0) };
    }

    pub fn prev(&mut self) {
        assert!(self.valid());
        let key = unsafe { (*self.node).data.as_ref() };
        self.node = self.list.find_less_than(key);

        if self.node == self.list.get_head() {
            self.node = null();
        }
    }

    pub fn key(&self) -> &[u8] {
        assert!(self.valid());
        unsafe { (*self.node).data.as_ref() as _ }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmp::DefaultComparator;
    use crate::{ArenaImpl, Random};

    #[test]
    fn test_basic() {
        let mut sl = SkipList::new(
            Random::new(0xdead_beef),
            DefaultComparator::default(),
            ArenaImpl::new(),
        );
        for i in 0..100u8 {
            sl.insert(vec![i]);
        }

        let mut iter = SkipListIter::new(sl);
        assert!(!iter.valid());
        iter.seek_to_first();
        assert!(iter.valid());
        assert_eq!(iter.key(), &[0]);
        iter.seek_to_last();
        assert_eq!(iter.key(), &[99]);

        iter.seek(&[88]);
        assert_eq!(iter.key(), &[88]);

        iter.next();
        assert_eq!(iter.key(), &[89]);

        iter.seek(&[99]);
        assert_eq!(iter.key(), &[99]);
        iter.prev();
        assert_eq!(iter.key(), &[98]);
    }
}
