Numeric conversions for [engineering notation](https://en.wikipedia.org/wiki/Engineering_notation)
and the [RKM code](https://en.wikipedia.org/wiki/RKM_code) variant.

## Overview

This crate is centred around the `EngineeringQuantity` type.
The intended use case is reading user-entered strings from configuration files.

* When using `EngineeringQuantity` you must specify the storage parameter type to use.
  This can be any primitive integer except for `i8` or `u8`.
* Conversion from string autodetects both standard and RKM code variants.
* An `EngineeringQuantity` may be converted to/from most primitive integer types.
* You can convert directly to string, or via the `DisplayAdapter` type for more control.
* Primitive integers may be converted directly to string via the `EngineeringRepr` convenience trait.

Or, if you prefer, the type relations in diagram form:

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
              │  DisplayAdapter<T>  │   ◀┘                 │          │
              └─────────────────────┘                      │          │
                │ Display                                  │          │
                ▼                                          │          │
              ╭─────────────────────╮                      │          │
              │       String        │   ───────────────────┘          │
              ╰─────────────────────╯   ◀─────────────────────────────┘
```

## Examples

### String to number
```rust
use engineering_repr::EngineeringQuantity as EQ;
use std::str::FromStr as _;
let eq = EQ::<i64>::from_str("1.5k").unwrap();
assert_eq!(i64::try_from(eq).unwrap(), 1500);
// RKM style strings
let eq2 = EQ::<i64>::from_str("1k5").unwrap();
assert_eq!(eq, eq2);
```

### Number to string
```rust
use engineering_repr::EngineeringQuantity as EQ;
let ee1 = EQ::<i32>::from(1200);
// default precision (3 places)
assert_eq!(ee1.to_string(), "1.20k");
// explicit precision
let ee2 = EQ::<i32>::from(1234567);
assert_eq!(ee2.with_precision(2).to_string(), "1.2M");
// RKM style
assert_eq!(ee2.rkm_with_precision(2).to_string(), "1M2");
```

#### Convenience traits
```rust
use engineering_repr::EngineeringRepr as _;
assert_eq!("123.45k", 123456.to_eng(5));
assert_eq!("123k4", 123456.to_rkm(4));
```

### Limitations

* This crate only supports integers at the present time. The smaller multipliers (m, μ, n, p, f, a, z, y, r, q) are not supported.
* Multipliers which are not a power of 1000 (da, h, d, c) are not supported.

## Alternatives

* [human-repr](https://crates.io/crates/human-repr) is great for converting numbers to human-friendly representations.
* [humanize-rs](https://crates.io/crates/humanize-rs) is great for converting some human-friendly representations to numbers, though engineering-repr offers more flexibility.
