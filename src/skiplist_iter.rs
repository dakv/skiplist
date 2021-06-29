use crate::skipnode::Node;
use crate::SkipList;
use std::ptr::null;

pub struct SkipListIter {
    list: *const SkipList,
    node: *const Node,
}

impl SkipListIter {
    pub fn new(list: *const SkipList) -> Self {
        Self { list, node: null() }
    }

    pub fn valid(&self) -> bool {
        !self.node.is_null()
    }

    pub fn seek_to_first(&mut self) {
        let n = unsafe { (*self.list).get_head() };
        self.node = n.get_next(0);
    }

    pub fn seek_to_last(&mut self) {
        self.node = unsafe { (*self.list).find_last() };
        if self.node == unsafe { (*self.list).get_head() } {
            self.node = null();
        }
    }

    pub fn seek(&mut self, s: &[u8]) {
        self.node = unsafe { (*self.list).find(s, &mut vec![]) };
    }

    pub fn next(&mut self) {
        assert!(self.valid());
        self.node = unsafe { (*self.node).get_next(0) };
    }

    pub fn prev(&mut self) {
        assert!(self.valid());
        let key = unsafe { (*self.node).data.as_ref() };
        self.node = unsafe { (*self.list).find_less_than(key) };

        if self.node == unsafe { (*self.list).get_head() } {
            self.node = null();
        }
    }

    pub fn key(&self) -> &[u8] {
        assert!(self.valid());
        unsafe { (*self.node).data.as_ref() as _ }
    }
}
