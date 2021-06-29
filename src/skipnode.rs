use crate::K_MAX_HEIGHT;
use bumpalo_herd::Herd;
use bytes::Bytes;
use std::fmt;
use std::fmt::{Error, Formatter};
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicPtr, Ordering};

pub struct Node {
    pub data: Bytes,
    // if fetch some node height is greater than height, return error
    // because the pointer is null.
    pub height: usize,
    pub forward: [AtomicPtr<Self>; K_MAX_HEIGHT],
}

impl Node {
    #[allow(clippy::mut_from_ref)]
    pub fn new(data: Bytes, height: usize, herd: &Herd) -> &mut Self {
        // todo hack the forward to optimize memory usage.
        let forward = {
            // Create an array of uninitialized values.
            let mut array: [MaybeUninit<AtomicPtr<Self>>; K_MAX_HEIGHT] =
                unsafe { MaybeUninit::uninit().assume_init() };
            for element in array.iter_mut() {
                *element = MaybeUninit::new(AtomicPtr::default());
            }
            unsafe { std::mem::transmute::<_, [AtomicPtr<Self>; K_MAX_HEIGHT]>(array) }
        };
        let bump = herd.get();
        bump.alloc(Node {
            data,
            height,
            forward,
        })
    }

    #[allow(clippy::mut_from_ref)]
    pub fn head(herd: &Herd) -> &mut Self {
        Self::new(Bytes::new(), K_MAX_HEIGHT, herd)
    }

    #[inline]
    pub fn set_next(&self, n: usize, node: *mut Node) {
        self.forward[n].store(node, Ordering::SeqCst);
    }

    #[inline]
    pub fn get_next(&self, n: usize) -> *mut Node {
        self.forward[n].load(Ordering::SeqCst)
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}", self.data.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::Node;
    use bumpalo_herd::Herd;

    #[test]
    fn test_new_node() {
        let herd = Herd::new();

        let node = Node::head(&herd);
        assert_eq!(format!("{}", node), "[]");

        let node = Node::new("da".into(), 0, &herd);
        assert_eq!(format!("{}", node), "[100, 97]");
    }

    #[test]
    fn test_next() {
        let herd = Herd::new();

        let node = Node::new(vec![1].into(), 3, &herd);
        let next = Node::new(vec![2].into(), 4, &herd);
        let tail = Node::new(vec![3].into(), 0, &herd);
        node.set_next(2, next);
        let ret = node.get_next(1);
        assert!(ret.is_null());
        let ret = node.get_next(2);
        assert!(!ret.is_null());
        unsafe {
            assert_eq!((*ret).data.as_ref(), &[2]);
        }

        next.set_next(3, tail);
        let v = next.get_next(3);
        unsafe {
            assert_eq!((*v).data.as_ref(), &[3]);
        }
    }
}
