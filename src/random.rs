pub trait RandomGenerator {
    fn next(&mut self) -> u64;
    // Returns a uniformly distributed value in the range [0..n-1]
    // REQUIRES: n > 0
    fn uniform(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    // Randomly returns true ~"1/n" of the time, and false otherwise.
    // REQUIRES: n > 0
    fn one_in(&mut self, n: u64) -> bool {
        (self.next() % n) == 0
    }
    // Skewed: pick "base" uniformly from range [0,max_log] and then
    // return "base" random bits.  The effect is to pick a number in the
    // range [0,2^max_log-1] with exponential bias towards smaller numbers.
    fn skewed(&mut self, max_log: u64) -> u64 {
        let tmp = 1 << self.uniform(max_log + 1);
        self.uniform(tmp)
    }
}

pub struct Random {
    seed_: u64,
}

impl Random {
    pub fn new(s: u64) -> Random {
        let mut seed_ = s & 0x7fff_ffff_u64;
        if seed_ == 0 || seed_ == 2_147_483_647 {
            seed_ = 1;
        }
        Random { seed_ }
    }
}

impl RandomGenerator for Random {
    fn next(&mut self) -> u64 {
        static M: u64 = 2_147_483_647; // 2^31-1
        static A: u64 = 16807; // bits 14, 8, 7, 5, 2, 1, 0
        let product = self.seed_.wrapping_mul(A);
        self.seed_ = (product >> 31) + (product & M);

        if self.seed_ > M {
            self.seed_ -= M;
        }
        self.seed_
    }
}
