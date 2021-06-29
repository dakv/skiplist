use std::cmp::Ordering;

pub trait BaseComparator {
    fn compare(&self, a: &[u8], b: &[u8]) -> Ordering;
}

#[derive(Default)]
pub struct DefaultComparator {}

impl BaseComparator for DefaultComparator {
    fn compare(&self, a: &[u8], b: &[u8]) -> Ordering {
        if a.eq(b) {
            Ordering::Equal
        } else if a.gt(b) {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BaseComparator;
    use crate::cmp::DefaultComparator;
    use std::cmp::Ordering;

    #[test]
    fn test_basic() {
        let cmp = DefaultComparator::default();
        assert_eq!(cmp.compare(&[1], &[2]), Ordering::Less);
        assert_eq!(cmp.compare(&[2], &[2]), Ordering::Equal);
        assert_eq!(cmp.compare(&[2], &[1]), Ordering::Greater);
    }
}
