use proc_macro2::Literal;

/// Converts the provided value to a hex literal.
///
/// Note: this function will panic if the value cannot be parsed as a
/// `proc_macro2::Literal`. This is the best choice, because that would be a bug
/// outside of this crate's control.
pub fn hex_literal(value: u64) -> Literal {
    format!("{:#X}", value).parse().unwrap()
}

/// e.g. 0b1111_1000 if used_bits is 5 and pad_bits is 3
pub fn used_mask_literal(total_bits: u8, pad_bits: u8) -> Literal {
    match total_bits {
        8 => {
            let mut x: u8 = 0xFF;
            x >>= pad_bits;
            x <<= pad_bits;
            hex_literal(x as u64)
        }
        16 => {
            let mut x: u16 = 0xFFFF;
            x >>= pad_bits;
            x <<= pad_bits;
            hex_literal(x as u64)
        }
        32 => {
            let mut x: u32 = 0xFFFF_FFFF;
            x >>= pad_bits;
            x <<= pad_bits;
            hex_literal(x as u64)
        }
        64 => {
            let mut x: u64 = 0xFFFF_FFFF_FFFF_FFFF;
            x >>= pad_bits;
            x <<= pad_bits;
            hex_literal(x)
        }
        _ => panic!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_used_mask_literal_8_2() {
        let literal = used_mask_literal(8, 2);
        let s = quote! { #literal }.to_string();
        assert_eq!(s, "0xFC");
    }

    #[test]
    fn test_used_mask_literal_16_6() {
        let literal = used_mask_literal(16, 6);
        let s = quote! { #literal }.to_string();
        assert_eq!(s, "0xFFC0");
    }

    #[test]
    fn test_used_mask_literal_32_11() {
        let literal = used_mask_literal(32, 11);
        let s = quote! { #literal }.to_string();
        assert_eq!(s, "0xFFFFF800");
    }
}
