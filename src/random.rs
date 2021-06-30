use std::sync::atomic::{AtomicU64, Ordering};

pub trait RandomGenerator {
    fn next(&self) -> u64;
    // Returns a uniformly distributed value in the range [0..n-1]
    // REQUIRES: n > 0
    fn uniform(&self, n: u64) -> u64 {
        self.next() % n
    }
    // Randomly returns true ~"1/n" of the time, and false otherwise.
    // REQUIRES: n > 0
    fn one_in(&self, n: u64) -> bool {
        (self.next() % n) == 0
    }
    // Skewed: pick "base" uniformly from range [0,max_log] and then
    // return "base" random bits.  The effect is to pick a number in the
    // range [0,2^max_log-1] with exponential bias towards smaller numbers.
    fn skewed(&self, max_log: u64) -> u64 {
        let tmp = 1 << self.uniform(max_log + 1);
        self.uniform(tmp)
    }
}

pub struct Random {
    seed: AtomicU64,
}

impl Random {
    pub fn new(s: u64) -> Random {
        let mut seed = s & 0x7fff_ffff_u64;
        if seed == 0 || seed == 2_147_483_647 {
            seed = 1;
        }
        Random {
            seed: AtomicU64::new(seed),
        }
    }
}

impl RandomGenerator for Random {
    fn next(&self) -> u64 {
        static M: u64 = 2_147_483_647; // 2^31-1
        static A: u64 = 16807; // bits 14, 8, 7, 5, 2, 1, 0
        let product = self.seed.load(Ordering::SeqCst) * A;
        self.seed
            .store((product >> 31) + (product & M), Ordering::SeqCst);

        if self.seed.load(Ordering::SeqCst) > M {
            self.seed.fetch_sub(M, Ordering::SeqCst);
        }
        self.seed.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Random, RandomGenerator};

    #[test]
    fn test_cmp() {
        let s = Random::new(0xdead_beef);
        assert_eq!(s.next(), 1624403320);
        assert!(s.one_in(386994929));
        assert_eq!(s.uniform(1643288587 + 1), 1643288587);
        assert_eq!(s.next(), 2111581289);
    }
}
