use std::cmp::Ordering;

pub trait BaseComparator {
    fn compare(&self, a: &[u8], b: &[u8]) -> Ordering;

    /// Less than
    fn lt(&self, a: &[u8], b: &[u8]) -> bool {
        self.compare(a, b) == Ordering::Less
    }

    /// Less than or equal
    fn le(&self, a: &[u8], b: &[u8]) -> bool {
        self.compare(a, b) != Ordering::Greater
    }

    /// Greater than
    fn gt(&self, a: &[u8], b: &[u8]) -> bool {
        self.compare(a, b) == Ordering::Greater
    }

    /// Greater than or equal
    fn ge(&self, a: &[u8], b: &[u8]) -> bool {
        self.compare(a, b) != Ordering::Less
    }

    /// Equal
    fn eq(&self, a: &[u8], b: &[u8]) -> bool {
        self.compare(a, b) == Ordering::Equal
    }

    /// Not equal
    fn ne(&self, a: &[u8], b: &[u8]) -> bool {
        self.compare(a, b) != Ordering::Equal
    }
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

    #[test]
    fn test_simplify() {
        let cmp = DefaultComparator::default();
        assert!(cmp.lt(&[1], &[2]));
        assert!(cmp.le(&[2], &[2]));
        assert!(cmp.ge(&[2], &[2]));
        assert!(cmp.eq(&[2], &[2]));
        assert!(cmp.gt(&[2], &[1]));
        assert!(cmp.ne(&[2], &[1]));
    }
}
