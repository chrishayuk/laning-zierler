//! Numbers in "the 24, 6 number system" (E-364, section 3).
//!
//! "Numbers appear in the machine essentially in the form a · 2^b where a is
//! a signed 24 place binary number with 1/2 ≤ a < 1 or a = 0 and b is a
//! signed 6 place binary number." So a value has 24 bits of mantissa and a
//! binary exponent from -63 to 63, and the largest magnitude is
//! (1 - 2^-24) · 2^63 = 2^63 - 2^39, the figure the manual gives for an alarm.
//!
//! A value is kept here as an `f64` that is always exactly one of those
//! numbers. Every operation is done in `f64` (exact or nearly so for 24-bit
//! operands) and then rounded back to 24 bits.
//!
//! Zero has a sign. Whirlwind was a one's-complement machine, and the manual
//! says so where it matters: "0 may appear as +0 or -0" (p. 8), which is why
//! `CP` may or may not jump on a zero. [`ZeroModel`] says how a zero gets its
//! sign; see the README for why the default is what it is.

use std::cmp::Ordering;
use std::fmt;

/// The largest magnitude: 2^63 - 2^39.
pub const MAX: f64 = 9_223_371_487_098_961_920.0;

/// The binary exponent range of the 6-place exponent.
const MAX_EXP: i32 = 63;
const MIN_EXP: i32 = -63;

/// How a zero that comes out of arithmetic gets its sign.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ZeroModel {
    /// One's complement: adding a number to its negative gives -0 (all ones),
    /// and so does adding +0 to -0. `n = n - 1` with `n` = 1 gives -0. Only
    /// `n = 0,` itself, or a sum of two +0s, gives +0. This is the default.
    #[default]
    OnesComplement,
    /// Every zero is +0, as a reader of today might assume.
    Positive,
}

/// What went wrong in an operation: the "arithmetic alarm" of section 16.
#[derive(Clone, Debug, PartialEq)]
pub enum Alarm {
    /// A result beyond 2^63 - 2^39.
    Overflow,
    /// Division by zero.
    ZeroDivisor,
    /// A function given an argument outside its range (the square root of a
    /// negative number, the logarithm of zero, sin⁻¹ of 2).
    Domain(&'static str),
}

impl fmt::Display for Alarm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Alarm::Overflow => write!(f, "a result exceeds 2^63 - 2^39"),
            Alarm::ZeroDivisor => write!(f, "division by zero"),
            Alarm::Domain(what) => write!(f, "{what}"),
        }
    }
}

/// A number in the 24, 6 system.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Num(f64);

impl Num {
    pub const ZERO: Num = Num(0.0);
    pub const ONE: Num = Num(1.0);

    /// Round an `f64` to 24 bits. A result too large is an alarm; one too
    /// small to be represented becomes a zero of the same sign.
    pub fn round(x: f64) -> Result<Num, Alarm> {
        if x.is_nan() {
            return Err(Alarm::Domain("the result is not a number"));
        }
        if x == 0.0 {
            return Ok(Num(x));
        }
        if x.is_infinite() {
            return Err(Alarm::Overflow);
        }
        let (m, e) = frexp(x.abs());
        // m is in [0.5, 1): keep 24 bits of it, rounding half away from zero.
        let mut m24 = (m * 16_777_216.0).round() / 16_777_216.0;
        let mut e = e;
        if m24 >= 1.0 {
            m24 = 0.5;
            e += 1;
        }
        if e > MAX_EXP {
            return Err(Alarm::Overflow);
        }
        if e < MIN_EXP {
            return Ok(Num(if x < 0.0 { -0.0 } else { 0.0 }));
        }
        let v = m24 * pow2(e);
        Ok(Num(if x < 0.0 { -v } else { v }))
    }

    /// A value that is already representable (used for constants made here).
    pub fn from_f64(x: f64) -> Num {
        Num::round(x).expect("constant out of range")
    }

    pub fn value(self) -> f64 {
        self.0
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0.0
    }

    /// The sign bit: true for a negative number and for -0. This is what `CP`
    /// and `CR` test.
    pub fn is_negative(self) -> bool {
        self.0.is_sign_negative()
    }


    pub fn add(self, other: Num, zero: ZeroModel) -> Result<Num, Alarm> {
        let sum = self.0 + other.0;
        if sum == 0.0 {
            return Ok(Num(signed_zero_of_sum(self, other, zero)));
        }
        // A sum too small to hold underflows to a zero, which the zero model
        // signs like any other.
        Ok(settle(Num::round(sum)?, zero))
    }

    /// -x. Under [`ZeroModel::OnesComplement`] the sign of a zero flips like
    /// any other, so -(+0) is -0; under [`ZeroModel::Positive`] every zero is
    /// +0, so -(+0) is +0. (The `-` operator always flips the sign.)
    pub fn negate(self, zero: ZeroModel) -> Num {
        settle(-self, zero)
    }

    /// The same value, with a zero signed by the zero model: under
    /// [`ZeroModel::Positive`] a -0 becomes +0.
    pub fn settled(self, zero: ZeroModel) -> Num {
        settle(self, zero)
    }

    pub fn sub(self, other: Num, zero: ZeroModel) -> Result<Num, Alarm> {
        self.add(-other, zero)
    }

    pub fn mul(self, other: Num, zero: ZeroModel) -> Result<Num, Alarm> {
        let p = Num::round(self.0 * other.0)?;
        Ok(settle(p, zero))
    }

    pub fn div(self, other: Num, zero: ZeroModel) -> Result<Num, Alarm> {
        if other.0 == 0.0 {
            return Err(Alarm::ZeroDivisor);
        }
        let q = Num::round(self.0 / other.0)?;
        Ok(settle(q, zero))
    }

    /// An integral power by repeated multiplication, as the exponents of
    /// section 2D are: "positive (unsigned) and negative integral exponents".
    pub fn powi(self, n: i32, zero: ZeroModel) -> Result<Num, Alarm> {
        if n == 0 {
            return Ok(Num::ONE);
        }
        let mut acc = self;
        for _ in 1..n.unsigned_abs() {
            acc = acc.mul(self, zero)?;
        }
        if n < 0 {
            Num::ONE.div(acc, zero)
        } else {
            Ok(acc)
        }
    }

    /// Compare by value, -0 equal to +0.
    pub fn cmp_value(self, other: Num) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(Ordering::Equal)
    }
}

/// Under [`ZeroModel::Positive`] every zero is +0.
fn settle(n: Num, zero: ZeroModel) -> Num {
    if zero == ZeroModel::Positive && n.0 == 0.0 {
        Num(0.0)
    } else {
        n
    }
}

/// The sign of a zero sum. In one's complement, x + (-x) is all ones, -0;
/// +0 + +0 is +0; anything with a -0 in it stays -0 unless the other operand
/// is a nonzero number (then the sum is not zero at all).
fn signed_zero_of_sum(a: Num, b: Num, zero: ZeroModel) -> f64 {
    match zero {
        ZeroModel::Positive => 0.0,
        ZeroModel::OnesComplement => {
            let both_plus_zero = a.0 == 0.0 && b.0 == 0.0 && !a.is_negative() && !b.is_negative();
            if both_plus_zero {
                0.0
            } else {
                -0.0
            }
        }
    }
}

/// x = m · 2^e with m in [0.5, 1), for finite positive x.
fn frexp(x: f64) -> (f64, i32) {
    let bits = x.to_bits();
    let exp_bits = ((bits >> 52) & 0x7ff) as i32;
    if exp_bits == 0 {
        // subnormal: scale up first
        let (m, e) = frexp(x * pow2(64));
        return (m, e - 64);
    }
    let e = exp_bits - 1022;
    let m = f64::from_bits((bits & !(0x7ff << 52)) | (1022u64 << 52));
    (m, e)
}

fn pow2(e: i32) -> f64 {
    2f64.powi(e)
}

/// The printed form of section 4: a sign, a point, eight digits, and the power
/// of ten. `5` is `+.50000000+01`; `.05` is `+.50000000-01`; `-π` is
/// `-.31415927+01`. A zero keeps its sign: `-.00000000+00`.
///
/// The digits are truncated, not rounded: a choice made to fit the manual's
/// illustrations, not something they establish. The manual prints `.05` as
/// `+.50000000-01`, and the 24-bit number nearest .05 (this reconstruction's
/// reading of it) is .0500000007...: a conversion that rounded would type
/// `+.50000001-01`. Truncation gives both of the manual's printed values
/// (`.05` and `π`).
pub fn format(n: Num) -> String {
    let sign = if n.is_negative() { '-' } else { '+' };
    let x = n.0.abs();
    if x == 0.0 {
        return format!("{sign}.00000000+00");
    }
    let (digits, e) = decimal(x);
    let esign = if e < 0 { '-' } else { '+' };
    format!("{sign}.{digits:08}{esign}{:02}", e.unsigned_abs())
}

/// Eight truncated decimal digits d and the exponent E with x = .d · 10^E,
/// computed exactly: x is m · 2^k with m an integer of at most 24 bits.
fn decimal(x: f64) -> (u64, i32) {
    let (m, k) = frexp(x);
    let mant = (m * 16_777_216.0) as u128; // x = mant · 2^(k-24)
    let k = k - 24;
    // x as a fraction num/den with both integers.
    let (mut num, mut den): (u128, u128) = if k >= 0 { (mant << k, 1) } else { (mant, 1u128 << (-k)) };
    // Scale by powers of ten until .1 <= num/den < 1.
    let mut e = 0i32;
    while num >= den {
        // divide by 10: multiply den, keeping exactness
        den *= 10;
        e += 1;
    }
    while num * 10 < den {
        num *= 10;
        e -= 1;
    }
    // num/den is in [.1, 1): its first eight digits.
    let digits = (num * 100_000_000) / den;
    (digits as u64, e)
}

impl std::ops::Neg for Num {
    type Output = Num;
    /// The sign flipped, zero included: -(+0) is -0.
    fn neg(self) -> Num {
        Num(-self.0)
    }
}

impl fmt::Display for Num {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format(*self))
    }
}

/// The twenty-three function subroutines of section 11, F¹ to F²³.
pub const FUNCTIONS: [(&str, &str); 23] = [
    ("sqrt", "square root"),
    ("sin", "sine"),
    ("cos", "cosine"),
    ("tan", "tangent"),
    ("asin", "sine⁻¹"),
    ("acos", "cosine⁻¹"),
    ("atan", "tangent⁻¹"),
    ("sec", "secant"),
    ("csc", "cosecant"),
    ("cot", "cotangent"),
    ("abs", "absolute value"),
    ("sgn", "signum"),
    ("exp", "exponential (eˣ)"),
    ("expm", "negative exponential (e⁻ˣ)"),
    ("sinh", "sinh"),
    ("cosh", "cosh"),
    ("tanh", "tanh"),
    ("sech", "sech"),
    ("csch", "csch"),
    ("coth", "coth"),
    ("log10", "log₁₀"),
    ("log2", "log₂"),
    ("ln", "logₑ"),
];

/// Apply F^k. The routines of 1954 are not known; these are computed in
/// double precision from the 24-bit argument and rounded to 24 bits.
pub fn function(k: u32, x: Num, zero: ZeroModel) -> Result<Num, Alarm> {
    let v = x.0;
    let recip = |d: f64| -> Result<f64, Alarm> {
        if d == 0.0 {
            Err(Alarm::ZeroDivisor)
        } else {
            Ok(1.0 / d)
        }
    };
    let r = match k {
        1 => {
            if v < 0.0 {
                return Err(Alarm::Domain("F¹: the square root of a negative number"));
            }
            v.sqrt()
        }
        2 => v.sin(),
        3 => v.cos(),
        4 => v.tan(),
        5 => {
            if v.abs() > 1.0 {
                return Err(Alarm::Domain("F⁵: sine⁻¹ of a number beyond 1"));
            }
            v.asin()
        }
        6 => {
            if v.abs() > 1.0 {
                return Err(Alarm::Domain("F⁶: cosine⁻¹ of a number beyond 1"));
            }
            v.acos()
        }
        7 => v.atan(),
        8 => recip(v.cos())?,
        9 => recip(v.sin())?,
        10 => recip(v.tan())?,
        11 => v.abs(),
        12 => {
            if v == 0.0 {
                v
            } else {
                v.signum()
            }
        }
        13 => v.exp(),
        14 => (-v).exp(),
        15 => v.sinh(),
        16 => v.cosh(),
        17 => v.tanh(),
        18 => recip(v.cosh())?,
        19 => recip(v.sinh())?,
        20 => recip(v.tanh())?,
        21..=23 => {
            if v <= 0.0 {
                return Err(Alarm::Domain("a logarithm of a number that is not positive"));
            }
            match k {
                21 => v.log10(),
                22 => v.log2(),
                _ => v.ln(),
            }
        }
        _ => return Err(Alarm::Domain("no such function subroutine")),
    };
    Ok(settle(Num::round(r)?, zero))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(x: f64) -> Num {
        Num::from_f64(x)
    }

    #[test]
    fn the_manuals_printed_forms() {
        assert_eq!(format(n(5.0)), "+.50000000+01");
        assert_eq!(format(n(0.05)), "+.50000000-01");
        assert_eq!(format(n(-std::f64::consts::PI)), "-.31415927+01");
    }

    #[test]
    fn the_largest_number() {
        assert_eq!(MAX, 2f64.powi(63) - 2f64.powi(39));
        assert!(Num::round(MAX).is_ok());
        assert_eq!(Num::round(2f64.powi(63)), Err(Alarm::Overflow));
        // 9223371487098961920, truncated to eight digits
        assert_eq!(format(n(MAX)), "+.92233714+19");
    }

    #[test]
    fn twenty_four_bits() {
        let third = n(1.0).div(n(3.0), ZeroModel::default()).unwrap();
        // 1/3 = .10101...₂; 24 bits of it, rounded, is 11184811 / 2^25
        assert_eq!(third.value(), 11_184_811.0 / 33_554_432.0);
        assert_eq!(format(third), "+.33333334+00");
    }

    #[test]
    fn a_difference_of_equals_is_minus_zero() {
        let z = ZeroModel::OnesComplement;
        let d = n(5.0).sub(n(5.0), z).unwrap();
        assert!(d.is_zero() && d.is_negative());
        assert_eq!(format(d), "-.00000000+00");
        let p = n(5.0).sub(n(5.0), ZeroModel::Positive).unwrap();
        assert!(!p.is_negative());
        // +0 + +0 stays +0; the constant 0 is +0
        assert!(!Num::ZERO.add(Num::ZERO, z).unwrap().is_negative());
    }

    #[test]
    fn plus_zero_signs_every_zero() {
        let z = ZeroModel::Positive;
        assert!(!Num::ZERO.negate(z).is_negative());
        assert!(Num::ZERO.negate(ZeroModel::OnesComplement).is_negative());
        // a difference too small to hold underflows to +0, not -0
        let a = n(2f64.powi(-63) * (1.0 + 2f64.powi(-23)));
        let b = n(2f64.powi(-63));
        let d = b.sub(a, z).unwrap();
        assert!(d.is_zero() && !d.is_negative());
        assert!(b.sub(a, ZeroModel::OnesComplement).unwrap().is_negative());
    }

    #[test]
    fn powers() {
        let z = ZeroModel::default();
        assert_eq!(n(5.0).powi(2, z).unwrap(), n(25.0));
        assert_eq!(n(2.0).powi(-2, z).unwrap(), n(0.25));
        assert_eq!(n(7.0).powi(0, z).unwrap(), n(1.0));
    }
}
