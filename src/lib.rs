mod skiplist;
mod skipnode;

pub use skiplist::Random;
pub use skiplist::RandomGenerator;
pub use skiplist::SkipList;
pub use skipnode::Node;

pub const K_MAX_HEIGHT: usize = 12;
