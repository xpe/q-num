# q-num

This library provides the `define_q_num!` procedural macro (evaluated at compile
time) to define a signed/unsigned binary fixed-point number type. It uses
ARM-style Q notation: `Qm.n` (signed) or `UQm.n` (unsigned), where:

- **m** is the number of integer bits, and
- **n** is the number of fractional bits.

Internally, the macro chooses the narrowest primitive integer type that can hold
m + n bits, up to `u64` (unsigned) and `i64` (signed). More internal details
are discussed below.

## Q Number Value

A Q number's value is the ratio of the stored number (having n + m bits)
and a fixed denominator (equal to 2 ^ n).

For example, using the UQ3.2 specification, the bit pattern 0b10111
represents the value 5.75. Keeping in mind the denominator is 2 ^ 2 = 4,
there are two ways to see this:

- 0b10111 / 4 == 23 / 4 == 5.75
- 0b101 + 0b11 / 4 == 5 + 3/4 == 5.75

## Example Macro Usage

Here is one example:

```rs
define_q_num!(MyQ, Q6.2);
let a = MyQ::tryFrom(13.75).unwrap();
let b = MyQ::tryFrom(-2.25).unwrap();
let c = a + b; // 11.5
```
