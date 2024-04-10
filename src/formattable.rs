use crate::float_impl::FormatFloat;
use crate::*;

/// A Type that can be formatted with a `SiFormat`.
pub trait Formattable {
    /// To reduce binary size, most types are formatted by casting them to another type and formatting that.
    /// This is that type.
    /// Currently, all values are formatted as floats, though this may change in the future.
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
            type Backing = <$via as Formattable>::Backing;
            fn si_format(self) -> SiFormatted<Self::Backing> {
                Formattable::si_format(self as $via)
            }
        }
    };
     ($($a:ty $(as $b:ty)?,)*)=>{
        $(
            formattable!( $a $(as $b)? );
        )*
    };
}

formattable!(u8 as u32, u16 as u32, i8 as i32, i16 as i32, u64, i64, u128, i128,);

// this is what std uses to decide how to format
#[cfg(any(target_pointer_width = "64", target_arch = "wasm32"))]
formattable!(u32 as u64, i32 as i64,);

#[cfg(not(any(target_pointer_width = "64", target_arch = "wasm32")))]
formattable!(u32, i32,);

#[cfg(feature = "float64")]
formattable!(f32 as f64);

formattable!(FormatFloat);
