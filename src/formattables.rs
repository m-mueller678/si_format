use crate::*;

macro_rules! formattable {
    ($t:ty) => {
        impl Formattable for $t {
            fn format_with(self, format: SiFormat) -> impl Display + Debug {
                SiFormatted { num: self, format }
            }
        }
    };
    ($t:ty,$via:ty) => {
        impl Formattable for $t {
            fn format_with(self, format: SiFormat) -> impl Display + Debug {
                Formattable::format_with(self as $via, format)
            }
        }
    };
}

formattable!(u64);
formattable!(usize, u64);
formattable!(u32, u64);
formattable!(u16, u64);
formattable!(u8, u64);

formattable!(i64);
formattable!(isize, i64);
formattable!(i32, i64);
formattable!(i16, i64);
formattable!(i8, i64);
