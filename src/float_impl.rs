#[cfg(all(feature = "std", feature = "libm"))]
compile_error!("feature \"std\" and feature \"libm\" cannot be enabled at the same time");

#[cfg(not(feature = "libm"))]
pub type MathImpl = f64;

#[cfg(feature = "libm")]
pub type MathImpl = libm::Libm<f64>;

#[allow(clippy::needless_return)]
pub fn powi(b: f64, e: i32) -> f64 {
    #[cfg(not(feature = "libm"))]
    {
        return b.powi(e);
    }
    #[cfg(feature = "libm")]
    {
        return MathImpl::pow(b, e as f64);
    }
}

#[allow(clippy::needless_return)]
pub fn abs(x: f64) -> f64 {
    #[cfg(not(feature = "libm"))]
    {
        return MathImpl::abs(x);
    }
    #[cfg(feature = "libm")]
    {
        return MathImpl::fabs(x);
    }
}
