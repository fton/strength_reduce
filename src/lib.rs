//! `strength_reduce` implements integer division and modulo via "arithmetic strength reduction"
//!
//! Modern processors can do multiplication and shifts much faster than division, and "arithmetic strength
//! reduction" is an algorithm to transform divisions into multiplications and shifts.
//! Compilers already perform this optimization for divisors that are known at compile time; this library 
//! enables this optimization for divisors that are only known at runtime.
//!
//! Benchmarking shows a 5-10x speedup or integer division and modulo operations.
//!
//! # Example:
//! ```
//! # extern crate core;
//! use strength_reduce::StrengthReducedU64;
//! use core::num::NonZeroU64;
//! 
//! let mut my_array: Vec<u64> = (0..500).collect();
//! let divisor = 3;
//! let modulo = 14;
//!
//! // slow naive division and modulo
//! for element in &mut my_array {
//!     *element = (*element / divisor) % modulo;
//! }
//!
//! // fast strength-reduced division and modulo
//! let reduced_divisor = StrengthReducedU64::new(NonZeroU64::new(divisor).unwrap());
//! let reduced_modulo = StrengthReducedU64::new(NonZeroU64::new(modulo).unwrap());
//! for element in &mut my_array {
//!     *element = (*element / reduced_divisor) % reduced_modulo;
//! }
//! ```
//!
//! This library is intended for hot loops like the example above, where a division is repeated many times
//! in a loop with the divisor remaining unchanged. 
//! There is a setup cost associated with creating stength-reduced division instances, so using strength-
//! reduced division for 1-2 divisions is not worth the setup cost.
//! The break-even point differs by use-case, but is typically low: Benchmarking has shown that takes 3 to
//! 4 repeated divisions with the same StengthReduced## instance to be worth it.
//! 
//! `strength_reduce` is `#![no_std]`
//!
//! The optimizations that this library provides are inherently dependent on architecture, compiler, and 
//! platform, so test before you use. 
#![no_std]

#[cfg(test)]
extern crate num_bigint;
#[cfg(test)]
extern crate rand;

use core:: {
    num:: { NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize },
    ops:: { Div, Rem, Range },
};

mod long_division;
mod long_multiplication;

/// Implements unsigned division and modulo via mutiplication and shifts.
///
/// Creating a an instance of this struct is more expensive than a single division, but if the division is
/// repeated, this version will be several times faster than naive division.
#[derive(Clone, Copy, Debug)]
pub struct StrengthReducedU8 {
    multiplier: u16,
    divisor: NonZeroU8,
}
impl StrengthReducedU8 {
    /// Creates a new divisor instance.
    ///
    /// If possible, avoid calling new() from an inner loop: The intended usage is to create an instance 
    /// of this struct outside the loop, and use it for divison and remainders inside the loop.
    ///
    /// # Panics:
    /// 
    /// When using in const context, you can use NonZeroUxxxx::new_unchecked() because zero `divisor` 
    /// always stop the compiler by zero divide.
    #[inline]
    pub const fn new(divisor: NonZeroU8) -> Self {
        Self {
            divisor,
            multiplier: if divisor.get().is_power_of_two() { // divisor is not zero.
                0
            } else {
                let divided = core::u16::MAX / (divisor.get() as u16); // panic! if divisor is zero.

                divided + 1
            },
        }
    }

    /// Simultaneous truncated integer division and modulus.
    /// Returns `(quotient, remainder)`.
    #[inline]
    pub const fn div_rem(numerator: u8, denom: Self) -> (u8, u8) {
        (denom.divide(numerator), denom.remainder(numerator))
    }

    /// Retrieve the value used to create this struct
    #[inline]
    pub const fn get(&self) -> u8 {
        self.divisor.get()
    }

    #[inline]
    pub const fn divide(&self, num: u8) -> u8 {
        if self.multiplier == 0 {
            (num as u16 >> self.get().trailing_zeros()) as u8
        } else {
            let numerator = num as u16;
            let multiplied_hi = numerator * (self.multiplier >> 8);
            let multiplied_lo = (numerator * self.multiplier as u8 as u16) >> 8;

            ((multiplied_hi + multiplied_lo) >> 8) as u8
        }
    }

    #[inline]
    pub const fn remainder(&self, num: u8) -> u8 {
        if self.multiplier == 0 {
            num & (self.get() - 1)
        } else {
            let product = self.multiplier.wrapping_mul(num as u16) as u32;
            let divisor = self.get() as u32;

            let shifted = (product * divisor) >> 16;
            shifted as u8
        }
    }
}

impl Div<StrengthReducedU8> for u8 {
    type Output = u8;

    #[inline]
    fn div(self, rhs: StrengthReducedU8) -> Self::Output {
        rhs.divide(self)
    }
}

impl Rem<StrengthReducedU8> for u8 {
    type Output = u8;

    #[inline]
    fn rem(self, rhs: StrengthReducedU8) -> Self::Output {
        rhs.remainder(self)
    }
}

// small types prefer to do work in the intermediate type
macro_rules! strength_reduced_u16 {
    ($struct_name:ident, $primitive_type:ty, $non_zero_type:ty) => (
        /// Implements unsigned division and modulo via mutiplication and shifts.
        ///
        /// Creating a an instance of this struct is more expensive than a single division, but if the 
        /// division is repeated, this version will be several times faster than naive division.
        #[derive(Clone, Copy, Debug)]
        pub struct $struct_name {
            multiplier: u32,
            divisor: $non_zero_type,
        }
        impl $struct_name {
            /// Creates a new divisor instance.
            ///
            /// If possible, avoid calling new() from an inner loop: The intended usage is to create an 
            /// instance of this struct outside the loop, and use it for divison and remainders inside 
            /// the loop.
            ///
            /// # Panics:
            /// 
            /// When using in const context, you can use NonZeroUxxxx::new_unchecked() because zero 
            /// `divisor` always stop the compiler by zero divide.
            #[inline]
            pub const fn new(divisor: $non_zero_type) -> Self {
                Self {
                     divisor,
                     multiplier: if divisor.get().is_power_of_two() { // divisor is not zero.
                        0
                    } else {
                        // panic! here if divisor is zero.
                        let divided = core::u32::MAX / (divisor.get() as u32);

                        divided + 1
                    }
                }
            }

            /// Simultaneous truncated integer division and modulus.
            /// Returns `(quotient, remainder)`.
            #[inline]
            pub const fn div_rem(numerator: $primitive_type, denom: Self)
                -> ($primitive_type, $primitive_type)
            {
                let quotient = denom.divide(numerator);
                let remainder = numerator - quotient * denom.get();
                (quotient, remainder)
            }

            /// Retrieve the value used to create this struct
            #[inline]
            pub const fn get(&self) -> $primitive_type {
                self.divisor.get()
            }

            #[inline]
            pub const fn divide(&self, num: $primitive_type) -> $primitive_type {
                if self.multiplier == 0 {
                    num >> self.get().trailing_zeros()
                } else {
                    let numerator = num as u32;
                    let multiplied_hi = numerator * (self.multiplier >> 16);
                    let multiplied_lo = (numerator * self.multiplier as u16 as u32) >> 16;

                    ((multiplied_hi + multiplied_lo) >> 16) as $primitive_type
                }
            }

            #[inline]
            pub const fn remainder(&self, num: $primitive_type) -> $primitive_type {
                if self.multiplier == 0 {
                    num & (self.get() - 1)
                } else {
                    let quotient = self.divide(num);
                    num - quotient * self.get()
                }
            }
        }

        impl Div<$struct_name> for $primitive_type {
            type Output = $primitive_type;

            #[inline]
            fn div(self, rhs: $struct_name) -> Self::Output {
                rhs.divide(self)
            }
        }

        impl Rem<$struct_name> for $primitive_type {
            type Output = $primitive_type;

            #[inline]
            fn rem(self, rhs: $struct_name) -> Self::Output {
                rhs.remainder(self)
            }
        }
    )
}

// small types prefer to do work in the intermediate type
macro_rules! strength_reduced_u32 {
    ($struct_name:ident, $primitive_type:ty, $non_zero_type:ty) => (
        /// Implements unsigned division and modulo via mutiplication and shifts.
        ///
        /// Creating a an instance of this struct is more expensive than a single division, but if the 
        /// division is repeated, this version will be several times faster than naive division.
        #[derive(Clone, Copy, Debug)]
        pub struct $struct_name {
            multiplier: u64,
            divisor: $non_zero_type,
        }
        impl $struct_name {
            /// Creates a new divisor instance.
            ///
            /// If possible, avoid calling new() from an inner loop: The intended usage is to create an 
            /// instance of this struct outside the loop, and use it for divison and remainders inside 
            /// the loop.
            ///
            /// # Panics:
            /// 
            /// When using in const context, you can use NonZeroUxxxx::new_unchecked() because zero 
            /// `divisor` always stop the compiler by zero divide.
            #[inline]
            pub const fn new(divisor: $non_zero_type) -> Self {
                Self {
                    divisor,
                    multiplier: if divisor.get().is_power_of_two() { // divisor is not zero.
                        0
                    } else {
                        let divided = core::u64::MAX / (divisor.get() as u64);

                        divided + 1
                    }
                }
            }

            /// Simultaneous truncated integer division and modulus.
            /// Returns `(quotient, remainder)`.
            #[inline]
            pub const fn div_rem(numerator: $primitive_type, denom: Self) 
                -> ($primitive_type, $primitive_type)
            {
                if denom.multiplier == 0 {
                    (numerator >> denom.get().trailing_zeros(), numerator & (denom.get() - 1))
                }
                else {
                    let numerator64 = numerator as u64;
                    let multiplied_hi = numerator64 * (denom.multiplier >> 32);
                    let multiplied_lo = numerator64 * (denom.multiplier as u32 as u64) >> 32;

                    let quotient = ((multiplied_hi + multiplied_lo) >> 32) as $primitive_type;
                    let remainder = numerator - quotient * denom.get();
                    (quotient, remainder)
                }
            }

            /// Retrieve the value used to create this struct
            #[inline]
            pub const fn get(&self) -> $primitive_type {
                self.divisor.get()
            }

            #[inline]
            pub const fn divide(&self, num: $primitive_type) -> $primitive_type {
                if self.multiplier == 0 {
                    num >> self.get().trailing_zeros()
                } else {
                    let numerator = num as u64;
                    let multiplied_hi = numerator * (self.multiplier >> 32);
                    let multiplied_lo = numerator * (self.multiplier as u32 as u64) >> 32;

                    ((multiplied_hi + multiplied_lo) >> 32) as $primitive_type
                }
            }
            
            #[inline]
            pub const fn remainder(&self, num: $primitive_type) -> $primitive_type {
                if self.multiplier == 0 {
                    num & (self.get() - 1)
                } else {
                    let product = self.multiplier.wrapping_mul(num as u64) as u128;
                    let divisor = self.get() as u128;

                    let shifted = (product * divisor) >> 64;
                    shifted as $primitive_type
                }
            }
        }

        impl Div<$struct_name> for $primitive_type {
            type Output = $primitive_type;

            #[inline]
            fn div(self, rhs: $struct_name) -> Self::Output {
                rhs.divide(self)
            }
        }

        impl Rem<$struct_name> for $primitive_type {
            type Output = $primitive_type;

            #[inline]
            fn rem(self, rhs: $struct_name) -> Self::Output {
                rhs.remainder(self)
            }
        }
    )
}

macro_rules! strength_reduced_u64 {
    ($struct_name:ident, $primitive_type:ty, $non_zero_type:ty) => (
        /// Implements unsigned division and modulo via mutiplication and shifts.
        ///
        /// Creating a an instance of this struct is more expensive than a single division, but if the 
        /// division is repeated, this version will be several times faster than naive division.
        #[derive(Clone, Copy, Debug)]
        pub struct $struct_name {
            multiplier: u128,
            divisor: $non_zero_type,
        }
        impl $struct_name {
            /// Creates a new divisor instance.
            ///
            /// If possible, avoid calling new() from an inner loop: The intended usage is to create an 
            /// instance of this struct outside the loop, and use it for divison and remainders inside 
            /// the loop.
            ///
            /// # Panics:
            /// 
            /// When using in const context, you can use NonZeroUxxxx::new_unchecked() because zero 
            /// `divisor` always stop the compiler by zero divide.
            #[inline]
            pub const fn new(divisor: $non_zero_type) -> Self {
                Self {
                    divisor,
                    multiplier: if divisor.get().is_power_of_two() { // diviser is not zero.
                        0
                    } else {
                        let quotient = long_division::divide_128_max_by_64(divisor.get() as u64);

                        quotient + 1
                    }
                }
            }

            /// Simultaneous truncated integer division and modulus.
            /// Returns `(quotient, remainder)`.
            #[inline]
            pub const fn div_rem(numerator: $primitive_type, denom: Self) 
                -> ($primitive_type, $primitive_type)
            {
                if denom.multiplier == 0 {
                    (numerator >> denom.get().trailing_zeros(), numerator & (denom.get() - 1))
                }
                else {
                    let numerator128 = numerator as u128;
                    let multiplied_hi = numerator128 * (denom.multiplier >> 64);
                    let multiplied_lo = numerator128 * (denom.multiplier as u64 as u128) >> 64;

                    let quotient = ((multiplied_hi + multiplied_lo) >> 64) as $primitive_type;
                    let remainder = numerator - quotient * denom.get();
                    (quotient, remainder)
                }
            }

            /// Retrieve the value used to create this struct
            #[inline]
            pub const fn get(&self) -> $primitive_type {
                self.divisor.get()
            }

            #[inline]
            pub const fn divide(&self, num: $primitive_type) -> $primitive_type {
                if self.multiplier == 0 {
                    num >> self.get().trailing_zeros()
                } else {
                    let numerator = num as u128;
                    let multiplied_hi = numerator * (self.multiplier >> 64);
                    let multiplied_lo = numerator * (self.multiplier as u64 as u128) >> 64;

                    ((multiplied_hi + multiplied_lo) >> 64) as $primitive_type
                }
            }

            #[inline]
            pub const fn remainder(&self, num: $primitive_type) -> $primitive_type {
                if self.multiplier == 0 {
                    num & (self.get() - 1)
                } else {
                    let quotient = self.divide(num);
                    num - quotient * self.get()
                }
            }
        }

        impl Div<$struct_name> for $primitive_type {
            type Output = $primitive_type;

            #[inline]
            fn div(self, rhs: $struct_name) -> Self::Output {
                rhs.divide(self)
            }
        }

        impl Rem<$struct_name> for $primitive_type {
            type Output = $primitive_type;

            #[inline]
            fn rem(self, rhs: $struct_name) -> Self::Output {
                rhs.remainder(self)
            }
        }
    )
}

/// Implements unsigned division and modulo via mutiplication and shifts.
///
/// Creating a an instance of this struct is more expensive than a single division, but if the division is
/// repeated, this version will be several times faster than naive division.
#[derive(Clone, Copy, Debug)]
pub struct StrengthReducedU128 {
    multiplier_hi: u128,
    multiplier_lo: u128,
    divisor: NonZeroU128,
}
impl StrengthReducedU128 {
    /// Creates a new divisor instance.
    ///
    /// If possible, avoid calling new() from an inner loop: The intended usage is to create an instance 
    /// of this struct outside the loop, and use it for divison and remainders inside the loop.
    ///
    /// # Panics:
    /// 
    /// When using in const context, you can use NonZeroUxxxx::new_unchecked() because zero `divisor` 
    /// always stop the compiler by zero divide.
    #[inline]
    pub const fn new(divisor: NonZeroU128) -> Self {
        if divisor.get().is_power_of_two() { 
            Self{ multiplier_hi: 0, multiplier_lo: 0, divisor }
        } else {
            let (quotient_hi, quotient_lo) = long_division::divide_256_max_by_128(divisor.get());
            let multiplier_lo = quotient_lo.wrapping_add(1);
            let multiplier_hi = if multiplier_lo == 0 { quotient_hi + 1 } else { quotient_hi };
            Self{ multiplier_hi, multiplier_lo, divisor }
        }
    }

    /// Simultaneous truncated integer division and modulus.
    /// Returns `(quotient, remainder)`.
    #[inline]
    pub const fn div_rem(numerator: u128, denom: Self) -> (u128, u128) {
        let quotient = denom.divide(numerator);
        let remainder = numerator - quotient * denom.get();
        (quotient, remainder)
    }

    /// Retrieve the value used to create this struct
    #[inline]
    pub const fn get(&self) -> u128 {
        self.divisor.get()
    }

    #[inline]
    pub const fn divide(&self, num: u128) -> u128 {
        if self.multiplier_hi == 0 {
            num >> self.get().trailing_zeros()
        } else {
            long_multiplication::multiply_256_by_128_upperbits(self.multiplier_hi, self.multiplier_lo, num)
        }
    }

    #[inline]
    pub const fn remainder(&self, num: u128) -> u128 {
        if self.multiplier_hi == 0 {
            num & (self.get() - 1)
        } else {
             let quotient = 
                long_multiplication::multiply_256_by_128_upperbits(self.multiplier_hi, self.multiplier_lo, num);
             num - quotient * self.get()
        }
    }
}

impl Div<StrengthReducedU128> for u128 {
    type Output = u128;

    #[inline]
    fn div(self, rhs: StrengthReducedU128) -> Self::Output {
        rhs.divide(self)
    }
}

impl Rem<StrengthReducedU128> for u128 {
    type Output = u128;

    #[inline]
    fn rem(self, rhs: StrengthReducedU128) -> Self::Output {
        rhs.remainder(self)
    }
}

// We just hardcoded u8 and u128 since they will never be a usize. for the rest, we have macros, so we can reuse
// the same code for usize
strength_reduced_u16!(StrengthReducedU16, u16, NonZeroU16);
strength_reduced_u32!(StrengthReducedU32, u32, NonZeroU32);
strength_reduced_u64!(StrengthReducedU64, u64, NonZeroU64);

// Our definition for usize will depend on how big usize is
#[cfg(target_pointer_width = "16")]
strength_reduced_u16!(StrengthReducedUsize, usize, NonZeroUsize);
#[cfg(target_pointer_width = "32")]
strength_reduced_u32!(StrengthReducedUsize, usize, NonZeroUsize);
#[cfg(target_pointer_width = "64")]
strength_reduced_u64!(StrengthReducedUsize, usize, NonZeroUsize);


pub(crate) const fn len(r: &Range<usize>) -> usize {
	r.end - r.start
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    macro_rules! reduction_test {
        ($test_name:ident, $struct_name:ident, $primitive_type:ident, $non_zero_type:ident) => (
            #[test]
            fn $test_name() {
                let max = core::$primitive_type::MAX;
                let divisors = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,max-1,max];
                let numerators = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20];

                for &divisor in &divisors {
                    let reduced_divisor = $struct_name::new($non_zero_type::new(divisor).unwrap());
                    for &numerator in &numerators {
                        let expected_div = numerator / divisor;
                        let expected_rem = numerator % divisor;

                        let reduced_div = numerator / reduced_divisor;

                        assert_eq!(
                            expected_div, reduced_div, 
                            "Divide failed with numerator: {}, divisor: {}", numerator, divisor
                        );
                        let reduced_rem = numerator % reduced_divisor;

                        let (reduced_combined_div, reduced_combined_rem) = 
                            $struct_name::div_rem(numerator, reduced_divisor);

                        
                        assert_eq!(
                            expected_rem, reduced_rem, 
                            "Modulo failed with numerator: {}, divisor: {}", numerator, divisor
                        );
                        assert_eq!(
                            expected_div, reduced_combined_div, 
                            "div_rem divide failed with numerator: {}, divisor: {}", numerator, divisor
                        );
                        assert_eq!(
                            expected_rem, reduced_combined_rem, 
                            "div_rem modulo failed with numerator: {}, divisor: {}", numerator, divisor
                        );
                    }
                }
            }
        )
    }

    reduction_test!(test_strength_reduced_u8, StrengthReducedU8, u8, NonZeroU8);
    reduction_test!(test_strength_reduced_u16, StrengthReducedU16, u16, NonZeroU16);
    reduction_test!(test_strength_reduced_u32, StrengthReducedU32, u32, NonZeroU32);
    reduction_test!(test_strength_reduced_u64, StrengthReducedU64, u64, NonZeroU64);
    reduction_test!(test_strength_reduced_usize, StrengthReducedUsize, usize, NonZeroUsize);
    reduction_test!(test_strength_reduced_u128, StrengthReducedU128, u128, NonZeroU128);

    #[test]
    fn for_debug() {
        let numerator = 0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFE;
        let divisor = 0xC187_F639_BEF7_D9FE_6EB9_F118_6E65_06E1;
        let expected_div = numerator / divisor;
        let expected_rem = numerator % divisor;
        let reduced_divisor = StrengthReducedU128::new(NonZeroU128::new(divisor).unwrap());
        let reduced_div = numerator / reduced_divisor;

        assert_eq!(expected_div, reduced_div, 
            "Divide failed with numerator: {:#X}, divisor: {:#X}", numerator, divisor);

        let reduced_rem = numerator % reduced_divisor;
        assert_eq!(expected_rem, reduced_rem, 
            "Modulo failed with numerator: {:#X}, divisor: {:#X}", numerator, divisor);

        let (reduced_combined_div, reduced_combined_rem) = 
            StrengthReducedU128::div_rem(numerator, reduced_divisor);
        assert_eq!(expected_div, reduced_combined_div, 
            "div_rem divide failed with numerator: {:#X}, divisor: {:#X}", numerator, divisor);
        assert_eq!(expected_rem, reduced_combined_rem, 
            "div_rem modulo failed with numerator: {:#X}, divisor: {:#X}", numerator, divisor);
    }
}
