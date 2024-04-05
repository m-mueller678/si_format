use crate::*;

/// A Type that can be formatted with a `SiFormat`.
pub trait Formattable {
    /// To reduce binary size, most types are formatted by casting them to another type and formatting that.
    /// This is that type.
    /// Currently, all values are formatted as `f64`, though this may change in the future.
    /// The concrete backing type used for any type is an implementation detail and you should not rely on it.
    #[allow(missing_docs)]
    #[allow(private_bounds)]
    type Backing: FormatImpl;
    /// Wraps self for formatting.
    /// The returned object can be further configured before display.
    fn si_format(self) -> SiFormatted<Self::Backing>;
}

macro_rules! formattable {
    ($t:ty) => {
        impl Formattable for $t {
            type Backing = $t;
            fn si_format(self) -> SiFormatted<Self::Backing> {
                SiFormatted {
                    num: self,
                    config: Config::new(),
                }
            }
        }
    };
    ($t:ty as $via:ty) => {
        impl Formattable for $t {
            type Backing = $via;
            fn si_format(self) -> SiFormatted<Self::Backing> {
                Formattable::si_format(self as $via)
            }
        }
    };
     ($($a:ty as $b:ty,)*)=>{
        $(
            formattable!( $a as $b );
        )*
    };
}

formattable!(
    usize as f64,
    u128 as f64,
    u64 as f64,
    u32 as f64,
    u16 as f64,
    u8 as f64,
    isize as f64,
    i128 as f64,
    i64 as f64,
    i32 as f64,
    i16 as f64,
    i8 as f64,
    f32 as f64,
);

formattable!(f64);
