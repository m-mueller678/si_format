use crate::*;

macro_rules! formattable {
    ($t:ty) => {
        impl Formattable for $t {
            fn format_with(&self, format: SiFormat) -> impl Display + Debug + 'static {
                SiFormatted { num: *self, format }
            }
        }
    };
}

formattable!(usize);
formattable!(isize);
