mod cmp;
mod random;
mod skiplist;
mod skiplist_iter;
mod skipnode;

pub use cmp::BaseComparator;
pub use random::{Random, RandomGenerator};
pub use skiplist::SkipList;
pub use skiplist_iter::SkipListIter;

pub const K_MAX_HEIGHT: usize = 12;
