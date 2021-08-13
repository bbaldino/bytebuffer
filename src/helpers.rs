use std::error::Error;

/// Generate a u8 mask to retrieve only |num_bits| bits starting at |start_position|, where
/// position 0 is the MSB of the byte and position 7 is the LSB.
pub(crate) fn get_u8_mask(start_position: usize, num_bits: usize) -> Result<u8, Box<dyn Error>> {
    if start_position + num_bits > 8 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Number of bits would exceed byte boundary",
        )
        .into());
    }
    if num_bits > 8 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid number of bits requested for u8 mask: {}", num_bits),
        )
        .into());
    }
    if num_bits == 0 {
        return Ok(0);
    }
    let mut mask = match num_bits {
        0 => 0,
        1 => 0b1,
        2 => 0b11,
        3 => 0b111,
        4 => 0b1111,
        5 => 0b11111,
        6 => 0b111111,
        7 => 0b1111111,
        8 => 0b11111111,
        _ => unreachable!(),
    };

    mask <<= 8 - start_position - num_bits;
    Ok(mask)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_u8_mask() {
        assert_eq!(get_u8_mask(0, 8).unwrap(), 0b11111111);
        assert_eq!(get_u8_mask(8, 0).unwrap(), 0);
        assert_eq!(get_u8_mask(4, 4).unwrap(), 0b00001111);
        assert!(get_u8_mask(4, 5).is_err());
    }
}
