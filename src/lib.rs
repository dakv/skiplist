#![feature(box_syntax)]
#![feature(box_into_raw_non_null)]

mod skipnode;
pub mod skiplist;

pub use skiplist::SkipList;
pub use skiplist::RandomGenerator;

pub const K_MAX_HEIGHT: usize = 12;
