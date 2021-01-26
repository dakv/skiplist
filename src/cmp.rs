pub trait BaseComparator<T> {
    // error[E0658]: associated type defaults are unstable
    // To make into an object
    fn compare(&self, a: &T, b: &T) -> i8;
}
// fixme i8 -> Order

#[derive(Default)]
pub struct DefaultComparator {}

impl<T> BaseComparator<T> for DefaultComparator
where
    T: PartialOrd,
{
    fn compare(&self, a: &T, b: &T) -> i8 {
        return if a.eq(b) {
            0
        } else if a.gt(b) {
            1
        } else {
            -1
        };
    }
}

#[cfg(test)]
mod tests {
    use super::BaseComparator;
    use crate::cmp::DefaultComparator;

    #[test]
    fn test_basic() {
        let cmp = DefaultComparator::default();
        assert_eq!(cmp.compare(&1u64, &2), -1);
        assert_eq!(cmp.compare(&2u32, &2), 0);
        assert_eq!(cmp.compare(&2u8, &1), 1);
    }
}
