// makes configuration easier
#![allow(unused_macros)]

use testcrate::*;

/// Make sure that the the edge case tester and randomized tester don't break, and list examples of
/// fuzz values for documentation purposes.
#[test]
fn fuzz_values() {
    const VALS: [u16; 47] = [
        0b0, // edge cases
        0b1111111111111111,
        0b1111111111111110,
        0b1111111111111100,
        0b1111111110000000,
        0b1111111100000000,
        0b1110000000000000,
        0b1100000000000000,
        0b1000000000000000,
        0b111111111111111,
        0b111111111111110,
        0b111111111111100,
        0b111111110000000,
        0b111111100000000,
        0b110000000000000,
        0b100000000000000,
        0b11111111111111,
        0b11111111111110,
        0b11111111111100,
        0b11111110000000,
        0b11111100000000,
        0b10000000000000,
        0b111111111,
        0b111111110,
        0b111111100,
        0b110000000,
        0b100000000,
        0b11111111,
        0b11111110,
        0b11111100,
        0b10000000,
        0b111,
        0b110,
        0b100,
        0b11,
        0b10,
        0b1,
        0b1010110100000, // beginning of random fuzzing
        0b1100011001011010,
        0b1001100101001111,
        0b1101010100011010,
        0b100010001,
        0b1000000000000000,
        0b1100000000000101,
        0b1100111101010101,
        0b1100010111111111,
        0b1111110101111111,
    ];
    let mut i = 0;
    fuzz(10, |x: u16| {
        assert_eq!(x, VALS[i]);
        i += 1;
    });
}

#[test]
fn leading_zeros() {
    use compiler_builtins::int::leading_zeros::{leading_zeros_default, leading_zeros_riscv};
    {
        use compiler_builtins::int::leading_zeros::__clzsi2;
        fuzz(N, |x: u32| {
            if x == 0 {
                return; // undefined value for an intrinsic
            }
            let lz = x.leading_zeros() as usize;
            let lz0 = __clzsi2(x);
            let lz1 = leading_zeros_default(x);
            let lz2 = leading_zeros_riscv(x);
            if lz0 != lz {
                panic!("__clzsi2({}): std: {}, builtins: {}", x, lz, lz0);
            }
            if lz1 != lz {
                panic!(
                    "leading_zeros_default({}): std: {}, builtins: {}",
                    x, lz, lz1
                );
            }
            if lz2 != lz {
                panic!("leading_zeros_riscv({}): std: {}, builtins: {}", x, lz, lz2);
            }
        });
    }

    {
        use compiler_builtins::int::leading_zeros::__clzdi2;
        fuzz(N, |x: u64| {
            if x == 0 {
                return; // undefined value for an intrinsic
            }
            let lz = x.leading_zeros() as usize;
            let lz0 = __clzdi2(x);
            let lz1 = leading_zeros_default(x);
            let lz2 = leading_zeros_riscv(x);
            if lz0 != lz {
                panic!("__clzdi2({}): std: {}, builtins: {}", x, lz, lz0);
            }
            if lz1 != lz {
                panic!(
                    "leading_zeros_default({}): std: {}, builtins: {}",
                    x, lz, lz1
                );
            }
            if lz2 != lz {
                panic!("leading_zeros_riscv({}): std: {}, builtins: {}", x, lz, lz2);
            }
        });
    }

    {
        use compiler_builtins::int::leading_zeros::__clzti2;
        fuzz(N, |x: u128| {
            if x == 0 {
                return; // undefined value for an intrinsic
            }
            let lz = x.leading_zeros() as usize;
            let lz0 = __clzti2(x);
            if lz0 != lz {
                panic!("__clzti2({}): std: {}, builtins: {}", x, lz, lz0);
            }
        });
    }
}

#[test]
#[cfg(not(target_arch = "avr"))]
fn bswap() {
    use compiler_builtins::int::bswap::{__bswapdi2, __bswapsi2};
    fuzz(N, |x: u32| {
        assert_eq!(x.swap_bytes(), __bswapsi2(x));
    });
    fuzz(N, |x: u64| {
        assert_eq!(x.swap_bytes(), __bswapdi2(x));
    });

    assert_eq!(__bswapsi2(0x12345678u32), 0x78563412u32);
    assert_eq!(__bswapsi2(0x00000001u32), 0x01000000u32);
    assert_eq!(__bswapdi2(0x123456789ABCDEF0u64), 0xF0DEBC9A78563412u64);
    assert_eq!(__bswapdi2(0x0200000001000000u64), 0x0000000100000002u64);

    #[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
    {
        use compiler_builtins::int::bswap::__bswapti2;
        fuzz(N, |x: u128| {
            assert_eq!(x.swap_bytes(), __bswapti2(x));
        });

        assert_eq!(
            __bswapti2(0x123456789ABCDEF013579BDF02468ACEu128),
            0xCE8A4602DF9B5713F0DEBC9A78563412u128
        );
        assert_eq!(
            __bswapti2(0x04000000030000000200000001000000u128),
            0x00000001000000020000000300000004u128
        );
    }
}

// This is approximate because of issues related to
// https://github.com/rust-lang/rust/issues/73920.
// TODO how do we resolve this indeterminacy?
macro_rules! pow {
    ($($f:ty, $tolerance:expr, $fn:ident);*;) => {
        $(
            #[test]
            fn $fn() {
                use compiler_builtins::float::pow::$fn;
                use compiler_builtins::float::Float;
                fuzz_float_2(N, |x: $f, y: $f| {
                    if !(Float::is_subnormal(x) || Float::is_subnormal(y) || x.is_nan()) {
                        let n = y.to_bits() & !<$f as Float>::SIGNIFICAND_MASK;
                        let n = (n as <$f as Float>::SignedInt) >> <$f as Float>::SIGNIFICAND_BITS;
                        let n = n as i32;
                        let tmp0: $f = x.powi(n);
                        let tmp1: $f = $fn(x, n);
                        let (a, b) = if tmp0 < tmp1 {
                            (tmp0, tmp1)
                        } else {
                            (tmp1, tmp0)
                        };
                        let good = {
                            if a == b {
                                // handles infinity equality
                                true
                            } else if a < $tolerance {
                                b < $tolerance
                            } else {
                                let quo = b / a;
                                (quo < (1. + $tolerance)) && (quo > (1. - $tolerance))
                            }
                        };
                        if !good {
                            panic!(
                                "{}({}, {}): std: {}, builtins: {}",
                                stringify!($fn), x, n, tmp0, tmp1
                            );
                        }
                    }
                });
            }
        )*
    };
}

#[cfg(not(all(target_arch = "x86", not(target_feature = "sse"))))]
mod float_pow {
    use super::*;

    pow! {
        f32, 1e-4, __powisf2;
        f64, 1e-12, __powidf2;
    }
}
