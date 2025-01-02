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
The intended use case is reading user-entered strings from configuration files.

## Detail

This crate is centred around the [`EngineeringQuantity`]`<T>` type.

* The generic parameter `T` specifies the storage type to use for the significand.
  This can be any primitive integer except for `i8` or `u8`.
  * For example, `EngineeringQuantity<u64>`.
* The exponent is always stored as an `i8`.

You can convert an `EngineeringQuantity` to:
* most primitive integer types (not `i8` or `u8`)
* String, directly by [`std::fmt::Display`], or via the [`DisplayAdapter`] type for control over the output.
* its component parts, as a tuple `(<StorageT>, i8)` (see [`to_raw`](EngineeringQuantity::to_raw))
* another `EngineeringQuantity` of a different storage type

You can create an `EngineeringQuantity` from:
* most primitive integer types (not `i8` or `u8`)
* `&str`, by [`FromStr`](core::str::FromStr)
  * this autodetects both standard and RKM code variants
* its component parts `(<StorageT>, i8)` (see [`from_raw`](EngineeringQuantity::from_raw)).

Primitive integers may be converted directly to string via the `EngineeringRepr` convenience trait.

Or, if you prefer, here are the type relations in diagram form:

```text
    ╭─────────────────────╮    ╭─────────────────────╮    ╭───────────╮
    │      integers       │    │    integer types    │    │ raw tuple │
    │ (i16/u16 or larger) │    │   (where Into<T>)   │    │  (T, i8)  │
    ╰─────────────────────╯    ╰─────────────────────╯    ╰───────────╯
      ╵          ▲                       │                      ▲
      ╵          │ TryFrom               │ From                 │ From
      ╵          │                       ▼                      ▼
      ╵       ┌───────────────────────────────────────────────────────────┐
      ╵       │              EngineeringQuantity<T>                       │
      ╵       └───────────────────────────────────────────────────────────┘
      ╵                                  │                 ▲          │
      ╵       ┌─────────────────────┐    │                 │          │
      ╵ impl  │   EngineeringRepr   │    │ (configurable   │ FromStr  │ Display
      └−−−−−▶ │ (convenience trait) │    │ format)         │          │
              └─────────────────────┘    │                 │          │
                │ to_eng(), to_rkm()     │                 │          │
                ▼                        │                 │          │
              ┌─────────────────────┐    │                 │          │
              │  DisplayAdapter<T>  │ ◀──┘                 │          │
              └─────────────────────┘                      │          │
                │ Display                                  │          │
                ▼                                          │          │
              ╭─────────────────────╮                      │          │
              │       String        │ ─────────────────────┘          │
              ╰─────────────────────╯ ◀───────────────────────────────┘
```

### Examples

#### String to number
```rust
use engineering_repr::EngineeringQuantity as EQ;
use std::str::FromStr as _;

// Standard notation
let eq = EQ::<i64>::from_str("1.5k").unwrap();
assert_eq!(i64::try_from(eq).unwrap(), 1500);

// RKM style notation
let eq2 = EQ::<i64>::from_str("1k5").unwrap();
assert_eq!(eq, eq2);
```

#### Number to string
```rust
use engineering_repr::EngineeringQuantity as EQ;

// default precision (3 places)
let ee1 = EQ::<i32>::from(1200);
assert_eq!(ee1.to_string(), "1.20k");
// explicit precision
let ee2 = EQ::<i32>::from(1234567);
assert_eq!(ee2.with_precision(2).to_string(), "1.2M");

// RKM style
assert_eq!(ee2.rkm_with_precision(2).to_string(), "1M2");

// Zero precision means "automatic, lossless"
assert_eq!(ee2.with_precision(0).to_string(), "1.234567M");
assert_eq!(ee2.rkm_with_precision(0).to_string(), "1M234567");
```

#### Convenience trait
```rust
use engineering_repr::EngineeringRepr as _;
assert_eq!("123.45k", 123456.to_eng(5));
assert_eq!("123.456k", 123456.to_eng(0)); // automatic precision
assert_eq!("123k4", 123456.to_rkm(4));
```

# Limitations

* This crate only supports integers at the present time. The smaller multipliers (m, μ, n, p, f, a, z, y, r, q) are not currently supported.
* Multipliers which are not a power of 1000 (da, h, d, c) are not supported.

# Alternatives

* [human-repr](https://crates.io/crates/human-repr) is great for converting numbers to human-friendly representations.
* [humanize-rs](https://crates.io/crates/humanize-rs) is great for converting some human-friendly representations to numbers, though engineering-repr offers more flexibility.
