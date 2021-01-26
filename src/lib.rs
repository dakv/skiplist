mod cmp;
mod random;
mod skiplist;
mod skipnode;

pub use cmp::KeyComparator;
pub use random::{Random, RandomGenerator};
pub use skiplist::SkipList;
pub use skipnode::Node;

pub const K_MAX_HEIGHT: usize = 12;
