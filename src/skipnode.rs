use std::ptr::NonNull;
use std::fmt;
use std::fmt::{Formatter, Error};
use crate::K_MAX_HEIGHT;


/// Key and value should never be None, except the head node.
/// Forward can not be None, except head node
#[derive(Copy, Clone, Debug)]
pub struct Node<T> {
    pub data: Option<T>,
    pub forward: [Option<NonNull<Node<T>>>; K_MAX_HEIGHT],
}

impl<T> Node<T> {
    pub fn new(data: T) -> Self {
        Node {
            data: Some(data),
            forward: [None; K_MAX_HEIGHT],
        }
    }

    pub fn head() -> Self {
        Node {
            data: None,
            forward: [None; K_MAX_HEIGHT],
        }
    }

    #[inline]
    pub fn set_next(&mut self, n: usize, node: Option<NonNull<Node<T>>>) {
        self.forward[n] = node;
    }

    #[inline]
    pub fn get_next(&self, n: usize) -> Option<*mut Node<T>> {
        self.forward[n].map(|v| {
            v.as_ptr()
        })
    }

    #[inline]
    pub fn get_mut_next(&self, n: usize) -> Option<NonNull<Node<T>>> {
        self.forward[n]
    }
}

impl<T> fmt::Display for Node<T>
    where T: fmt::Display
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if let Some(ref v) = self.data {
            write!(f, "{}", v)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Node;
    use std::ptr::NonNull;

    #[test]
    fn test_new_node() {
        let node = Node::<u8>::head();
        assert_eq!(format!("{}", node), "");

        let node = Node::new(1);
        assert_eq!(format!("{}", node), "1");

        let node = Node::new("da");
        assert_eq!(format!("{}", node), "da");
    }

    #[test]
    fn test_next() {
        let mut node = Node::new(1);
        let mut next = Node::new(2);
        let mut tail = Node::new(3);
        node.set_next(2, NonNull::new(&mut next));
        let ret = node.get_next(1);
        assert!(ret.is_none());
        let ret = node.get_next(2);
        assert!(ret.is_some());
        next.set_next(3, NonNull::new(&mut tail));
        unsafe {
            if let Some(v) = next.get_next(3) {
                let d = *v;
                assert_eq!(d.data.unwrap(), 3);
            }
        }
    }
}
