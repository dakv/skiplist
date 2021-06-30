use crate::skipnode::Node;
use crate::{SkipList, K_MAX_HEIGHT};
use std::iter;
use std::ptr::{null, null_mut};

pub struct SkipListIter {
    list: SkipList,
    node: *const Node,
}

impl SkipListIter {
    pub fn new(list: &SkipList) -> Self {
        Self {
            list: SkipList::from(list),
            node: null(),
        }
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

    pub fn seek(&mut self, s: &[u8]) {
        let mut prev = iter::repeat(null_mut()).take(K_MAX_HEIGHT).collect();
        self.node = self.list.find(s, &mut prev);
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
