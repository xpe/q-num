use crate::literal::{hex_literal, used_mask_literal};
use crate::math::total_bits;
use crate::parse::Input;
use crate::types::{core_primitive_type, signed_int_qualified, unsigned_int_qualified};
use proc_macro2::{Literal, TokenStream};
use quote::quote;
use syn::{Ident, Type};

struct Data {
    name: Ident,
    total_bits: u8,
    used_bits: u8,
    int_bits: u8,
    frac_bits: u8,
    pad_bits: u8,
    inner_type: Type,
    denominator: f64,
    conversion_factor: f64,
    signed: bool,
    q_notation: String,
    used_mask: Literal,
    min_float: f64,
    max_float: f64,
    min_inner: Literal,
    max_inner: Literal,
}

pub fn generate(input: Input) -> syn::Result<TokenStream> {
    let data = prepare_data(input)?;
    generate_from_data(data)
}

#[rustfmt::skip]
fn prepare_data(input: Input) -> syn::Result<Data> {
    assert!(input.int_bits >= 1);
    let int_bits = input.int_bits;
    let frac_bits = input.frac_bits;
    let used_bits = input.int_bits + input.frac_bits;
    let total_bits = total_bits(used_bits)?;
    let pad_bits = total_bits - used_bits;
    let denominator = (1 << frac_bits) as f64;
    let signed = input.signed;
    let (min_float, max_float) = if signed {
        let x = (1 << (int_bits - 1)) as f64;
        (-x, x - 1.0 / denominator)
    } else {
        (0.0, ((1 << used_bits) - 1) as f64 / denominator)
    };
    let (min_inner, max_inner) = if signed {
        let n = (1 << (used_bits - 1)) as u64;
        (hex_literal(n), hex_literal(n - 1))
    } else {
        let n = (1 << used_bits) as u64;
        (hex_literal(0), hex_literal(n - 1))
    };
    Ok(Data {
        name: input.name,
        total_bits, used_bits, int_bits, frac_bits, pad_bits,
        inner_type: if input.signed {
            signed_int_qualified(used_bits)?
        } else {
            unsigned_int_qualified(used_bits)?
        },
        denominator,
        conversion_factor: (1 << (frac_bits + pad_bits)) as f64,
        signed,
        q_notation: if signed {
            format!("Q{int_bits}.{frac_bits}")
        } else {
            format!("UQ{int_bits}.{frac_bits}")
        },
        min_float, max_float, min_inner, max_inner,
        used_mask: used_mask_literal(total_bits, pad_bits),
    })
}

fn generate_from_data(data: Data) -> syn::Result<TokenStream> {
    #[rustfmt::skip]
    let Data {
        name, total_bits, used_bits, int_bits, frac_bits, pad_bits,
        inner_type, denominator, conversion_factor, signed, q_notation,
        used_mask, min_float, max_float, min_inner, max_inner
    } = data;
    let u8 = core_primitive_type("u8")?;
    let f64 = core_primitive_type("f64")?;
    Ok(quote! {
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        pub struct #name(#inner_type);

        impl #name {
            pub const Q_NOTATION: &'static str = #q_notation;
            pub const SIGNED: bool = #signed;
            pub const TOTAL_BITS: #u8 = #total_bits;
            pub const USED_BITS: #u8 = #used_bits;
            pub const INT_BITS: #u8 = #int_bits;
            pub const FRAC_BITS: #u8 = #frac_bits;
            pub const PAD_BITS: #u8 = #pad_bits;
            pub const USED_MASK: #inner_type = #used_mask;
            pub const MIN_FLOAT: #f64 = #min_float;
            pub const MAX_FLOAT: #f64 = #max_float;
            pub const MIN: Self = Self(#min_inner);
            pub const MAX: Self = Self(#max_inner);
            pub const DENOMINATOR: #f64 = #denominator;
            pub const CONVERSION_FACTOR: #f64 = #conversion_factor;

            /// Returns the inner value.
            pub fn to_bits(self) -> #inner_type { self.0 }

            /// Builds a new instance using the provided bits;
            ///
            /// Note: ensures unused bits (the padding) are zeroed out.
            pub fn from_bits(bits: #inner_type) -> Self {
                Self(bits & Self::USED_MASK)
            }
        }

        impl TryFrom<#f64> for #name {
            type Error = Box<dyn core::error::Error + Send + Sync>;
            fn try_from(value: #f64) -> core::result::Result<Self, Self::Error> {
                if !(Self::MIN_FLOAT..=Self::MAX_FLOAT).contains(&value) {
                    Err(format!("{} is out of range for {}", value, Self::Q_NOTATION).into())
                }
                else {
                    let n = (value * Self::CONVERSION_FACTOR) as #inner_type;
                    Ok(Self(n & Self::USED_MASK))
                }
            }
        }

        impl From<#name> for #f64 {
            fn from(value: #name) -> Self {
                (value.0 as #f64) / #name::CONVERSION_FACTOR
            }
        }

        impl core::ops::Add for #name {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }
    })
}
