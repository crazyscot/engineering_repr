[![Crates.io](https://img.shields.io/crates/v/engineering_repr.svg)](https://crates.io/crates/engineering_repr)
![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/crazyscot/engineering_repr)
[![Build status](https://github.com/crazyscot/engineering_repr/actions/workflows/rust.yml/badge.svg)](https://github.com/crazyscot/engineering_repr/actions/workflows/rust.yml)
[![Documentation](https://img.shields.io/docsrs/engineering-repr)](https://docs.rs/engineering_repr/)
![License](https://img.shields.io/badge/license-MIT-blue)
[![Coverage](https://coveralls.io/repos/github/crazyscot/engineering_repr/badge.svg?branch=main)](https://coveralls.io/github/crazyscot/engineering_repr?branch=main)

Numeric conversions for [engineering notation](https://en.wikipedia.org/wiki/Engineering_notation)
and [RKM code](https://en.wikipedia.org/wiki/RKM_code).

## Overview

In engineering applications it is common to express quantities relative to the next-lower power of 1000, described by an [SI (metric) prefix](https://en.wikipedia.org/wiki/Metric_prefix).

This is normally done by writing the SI multiplier after the quantity. In the "RKM code" variant, the SI multiplier replaces the decimal point.

For example:

| Number  | Engineering | RKM  |
| --:     | --:         | --:  |
| 42      | 42          | 42   |
| 999     | 999         | 999  |
| 1000    | 1k          | 1k   |
| 1500    | 1.5k        | 1k5  |
| 42900   | 42.9k       | 42k9 |
| 2340000 | 2.34M       | 2M34 |

And so on going up the SI prefixes, including the new ones R (10<sup>27</sup>) and Q (10<sup>30</sup>) which were added in 2022.

This crate exists to support convenient conversion of numbers to/from engineering and RKM notation.
The intended use case is for parsing user-entered configuration.

## Detail

This crate is centred around the `EngineeringQuantity<T>` type. This type supports comparisons via `PartialEq`, `Eq`, `PartialOrd` and `Ord`, though if you want to perform actual maths you are probably better off converting to int or `Ratio`.

### Storage

* The generic parameter `T` specifies the storage type to use for the significand.
  This can be any primitive integer except for `i8` or `u8`, which are too small to be useful.
  * For example, `EngineeringQuantity<u64>`.
* The exponent is always stored as an `i8`. This can range from -10 (q) to +10 (Q); going beyond that will likely cause `Overflow` or `Underflow` errors.

### Conversions

You can convert an `EngineeringQuantity` to:
* integer types, truncating any fraction:
  * directly `into` type `T`, or a larger integer type (one which implements `From<T>`);
  * any integer type using the `num_traits::ToPrimitive` trait (`to_i32()` and friends, which apply an overflow check);
* String, optionally via the `DisplayAdapter` type to control the formatting;
* another `EngineeringQuantity` (`convert` if the destination storage type is larger; `try_convert` if it is smaller);
* `f32` and `f64` (with an over/underflow check);
* `num_rational::Ratio` (with an over/underflow check);
* its component parts, as a tuple `(<T>, i8)` (see `to_raw`).

You can create an `EngineeringQuantity` from:
* type `T`, or a smaller integer type (one which implements `Into<T>`);
* String or `&str`, which autodetects both standard and RKM code variants;
* `num_rational::Ratio`, which requires the denominator be a power of 1000;
* its component parts `(<T>, i8)` (see `from_raw`), which will overflow if the converted number cannot fit into `T`.

Supported integer types may be converted directly to string via the `EngineeringRepr` convenience trait.

Or, if you prefer, here are the type relations in diagram form:

```text
                                            ┌────────────────────┐
                                            │      integers      │
                                            └────────────────────┘
                                              ▲                I
                                              ╵                I [impl]
                                              ╵                I
                                              ▼                ▼
          ┌───────────────────────────────────────────┐  ┌─────────────────────┐
          │           EngineeringQuantity<T>          │  │   EngineeringRepr   │
          │                                           │  │ (convenience trait) │
          └───────────────────────────────────────────┘  └─────────────────────┘
            ▲             ▲            ╵        ▲    │        │
            ╵             ╵            ╵        ╵    │        │ to_eng()
            ╵             ╵            ╵        ╵    │        │ to_rkm()
            ▼             ▼            ▼        ╵    ▼        ▼
┌─────────────┐  ┌────────────────┐  ┌───────┐  ╵   ┌───────────────────┐
│ "raw" tuple │  │ num_rational:: │  │  f32  │  ╵   │ DisplayAdapter<T> │
│   (T, i8)   │  │    Ratio<T>    │  │  f64  │  ╵   │                   │
└─────────────┘  └────────────────┘  └───────┘  ╵   └───────────────────┘
                                                ╵       │
                                                ╵       │
                                                ▼       ▼
                                           ┌───────────────────┐
                                           │      String       │
                                           └───────────────────┘
```

### Examples

#### String to number
```rust
use engineering_repr::EngineeringQuantity as EQ;
use std::str::FromStr as _;
use num_rational::Ratio;

// Standard notation
let eq = EQ::<i64>::from_str("1.5k").unwrap();
assert_eq!(i64::try_from(eq).unwrap(), 1500);

// RKM style notation
let eq2 = EQ::<i64>::from_str("1k5").unwrap();
assert_eq!(eq, eq2);

// Conversion to the nearest integer
let eq3 = EQ::<i32>::from_str("3m").unwrap();
assert_eq!(i32::try_from(eq3).unwrap(), 0);
// Convert to Ratio
let r : Ratio<i32> = eq3.try_into().unwrap();
assert_eq!(r, Ratio::new(3, 1000)); // => 3 / 1000
// Convert to float
let f : f64 = eq3.try_into().unwrap();
assert_eq!(f, 0.003); // caution, not all float conversions will work out exactly
```

#### Number to string
```rust
use engineering_repr::EngineeringQuantity as EQ;

// default precision (3 places, "sloppy" omitting trailing zeroes)
let ee1 = EQ::<i32>::from(1200);
assert_eq!(ee1.to_string(), "1.2k");
// strict precision
assert_eq!(ee1.with_strict_precision(3).to_string(), "1.20k");
// explicit precision
let ee2 = EQ::<i32>::from(1234567);
assert_eq!(ee2.with_precision(2).to_string(), "1.2M");

// RKM style
assert_eq!(ee2.rkm_with_precision(2).to_string(), "1M2");

// Zero precision means "automatic, lossless"
assert_eq!(ee2.with_precision(0).to_string(), "1.234567M");
assert_eq!(ee2.rkm_with_precision(0).to_string(), "1M234567");
```

#### Integer directly to string via convenience trait
```rust
use engineering_repr::EngineeringRepr as _;
assert_eq!("123.45k", 123456.to_eng(5));
assert_eq!("123.456k", 123456.to_eng(0)); // automatic precision
assert_eq!("123k4", 123456.to_rkm(4));
```

# Limitations

* Multipliers which are not a power of 1000 (da, h, d, c) are not supported.

# Alternatives

* [human-repr](https://crates.io/crates/human-repr) is great for converting numbers to human-friendly representations.
* [humanize-rs](https://crates.io/crates/humanize-rs) is great for converting some human-friendly representations to numbers, though engineering-repr offers more flexibility.