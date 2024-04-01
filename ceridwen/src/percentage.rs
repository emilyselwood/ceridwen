pub trait ConvertToFloat {
    fn to_f64_lossy(self) -> f64;
}

impl ConvertToFloat for f64 {
    fn to_f64_lossy(self) -> f64 {
        self
    }
}

macro_rules! implement_from_to_string {
    ($($primitive:ty,)*) => (
        $(impl ConvertToFloat for $primitive {
            #[inline]
            fn to_f64_lossy(self) -> f64 {
                self as f64
            }
        })*
    );
}

implement_from_to_string! {
    i8, i16, i32, i64, isize,
    u8, u16, u32, u64, usize,
    f32,
}

pub fn percentage<T1, T2>(total: T1, amount: T2) -> f64
where
    T1: ConvertToFloat,
    T2: ConvertToFloat,
{
    amount.to_f64_lossy() / (total.to_f64_lossy() / 100.0)
}

#[cfg(test)]
mod tests {

    use crate::percentage::percentage;

    #[test]
    fn test_percentage() {
        assert_eq!(percentage(10_usize, 1_usize), 10.0);
        assert_eq!(percentage(2000_u64, 100_u64), 5.0);
        assert_eq!(percentage(100_i32, 110_i32), 110.0);
    }
}
