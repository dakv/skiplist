use crate::{Node, SkipList};
use std::ptr::null;

pub struct SkipListIter<T> {
    list: *const SkipList<T>,
    node: *const Node<T>,
}

impl<T> SkipListIter<T>
where
    T: Clone + std::fmt::Display,
{
    pub fn new(list: *const SkipList<T>) -> Self {
        Self { list, node: null() }
    }

    pub fn valid(&self) -> bool {
        !self.node.is_null()
    }

    pub fn seek_to_first(&mut self) {
        let n = unsafe { (*self.list).get_head() };
        self.node = match n.get_next(0) {
            Some(v) => v,
            None => null(),
        };
    }

    pub fn seek_to_last(&mut self) {
        self.node = unsafe { (*self.list).find_last() };
        if self.node == unsafe { (*self.list).get_head() } {
            self.node = null();
        }
    }

    pub fn seek(&mut self, s: &T) {
        self.node = unsafe { (*self.list).find(s, &mut vec![]) };
    }

    pub fn next(&mut self) {
        assert!(self.valid());
        if let Some(s) = unsafe { (*self.node).get_next(0) } {
            self.node = s;
        } else {
            self.node = null();
        }
    }

    pub fn prev(&mut self) {
        assert!(self.valid());
        let key = unsafe { (*self.node).data.as_ref().unwrap() };
        self.node = unsafe { (*self.list).find_less_than(key) };

        if self.node == unsafe { (*self.list).get_head() } {
            self.node = null();
        }
    }

    pub fn key(&self) -> &T {
        assert!(self.valid());
        unsafe {
            let result = (*self.node).data.as_ref().unwrap();
            result
        }
    }
}
