use crate::*;

macro_rules! formattable {
    ($t:ty) => {
        impl Formattable for $t {
            type BackingImpl = $t;
            fn si_format(self) -> SiFormatted<Self::BackingImpl> {
                SiFormatted {
                    num: self,
                    config: Config::new(),
                }
            }
        }
    };
    ($t:ty,$via:ty) => {
        impl Formattable for $t {
            type Formatted = SiFormatted<$via>;
            fn si_format(self) -> Self:: {
                if let Ok(x) = <$via as TryFrom<$t>>::try_from(self){
                    Formattable::si_format(x)
                }else{
                    let mut ret = Formattable::si_format(self as $via);
                    ret.config.shift = isize::MAX;
                    ret
                }
            }
        }
    };
}

formattable!(u128);
formattable!(u64);
formattable!(usize, u64);
formattable!(u32, u64);
formattable!(u16, u64);
formattable!(u8, u64);

formattable!(i128);
formattable!(i64);
formattable!(isize, i64);
formattable!(i32, i64);
formattable!(i16, i64);
formattable!(i8, i64);
