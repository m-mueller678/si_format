#[cfg(all(feature = "std", feature = "libm"))]
compile_error!("feature \"std\" and feature \"libm\" cannot be enabled at the same time");

#[cfg(not(feature = "libm"))]
pub type MathImpl = FormatFloat;

#[cfg(feature = "float64")]
pub type FormatFloat = f64;
#[cfg(not(feature = "float64"))]
pub type FormatFloat = f32;

#[cfg(feature = "libm")]
pub type MathImpl = libm::Libm<FormatFloat>;

#[allow(clippy::needless_return)]
pub fn powi(b: FormatFloat, e: i32) -> FormatFloat {
    #[cfg(not(feature = "libm"))]
    {
        return b.powi(e);
    }
    #[cfg(feature = "libm")]
    {
        return MathImpl::pow(b, e as FormatFloat);
    }
}

#[allow(clippy::needless_return)]
pub fn abs(x: FormatFloat) -> FormatFloat {
    #[cfg(not(feature = "libm"))]
    {
        return MathImpl::abs(x);
    }
    #[cfg(feature = "libm")]
    {
        return MathImpl::fabs(x);
    }
}
