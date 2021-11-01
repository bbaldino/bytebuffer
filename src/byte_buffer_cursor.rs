use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use crate::bit_read::BitRead;
use crate::bit_write::BitWrite;
use crate::byte_buffer_exts::ByteBufferExts;
use crate::error::Error;
use crate::helpers::get_u8_mask;
use crate::sized_buffer::SizedByteBuffer;

/// Similar to |std::io::Cursor| but designed to keep track of a buffer of bytes where amounts less
/// than a single byte (i.e. some number of bits) can be read.
#[derive(Debug)]
pub struct ByteBufferCursor {
    byte_cursor: Cursor<Vec<u8>>,
    bit_pos: u8,
}

impl ByteBufferCursor {
    pub fn new(data: Vec<u8>) -> Self {
        ByteBufferCursor {
            byte_cursor: Cursor::new(data),
            bit_pos: 0,
        }
    }

    fn increment_bit_pos(&mut self, num_bits: usize) {
        self.bit_pos += num_bits as u8;
        if self.bit_pos >= 8 {
            self.bit_pos %= 8;
            self.byte_cursor
                .set_position(self.byte_cursor.position() + 1)
        }
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.byte_cursor.into_inner()
    }
}

impl Seek for ByteBufferCursor {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, std::io::Error> {
        self.bit_pos = 0;
        self.byte_cursor.seek(pos)
    }
}

impl SizedByteBuffer for ByteBufferCursor {
    fn bytes_remaining(&self) -> usize {
        match self.bit_pos {
            0 => self.byte_cursor.get_ref().len() - self.byte_cursor.position() as usize,
            // If we're in the middle of a byte, don't count that as a full byte remaining
            // (Note that this is a somewhat arbitrary decision, but it's what makes more sense
            // to me as of now)
            _ => self.byte_cursor.get_ref().len() - self.byte_cursor.position() as usize - 1,
        }
    }
}

impl ByteBufferCursor {
    /// Return a copy of the byte at the byte cursor's current position. |bit_pos|
    /// refers to the current position within this byte.
    fn get_current_byte(&self) -> Result<u8, Error> {
        Ok(self.byte_cursor.get_ref()[self.byte_cursor.position() as usize])
    }

    /// Return a mutable reference to the byte at the byte cursor's current position. |bit_pos|
    /// refers to the current position within this byte.
    fn get_current_byte_mut(&mut self) -> Result<&mut u8, Error> {
        let curr_pos = self.byte_cursor.position() as usize;
        Ok(&mut self.byte_cursor.get_mut()[curr_pos])
    }
}

impl Read for ByteBufferCursor {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self.bit_pos {
            0 => self.byte_cursor.read(buf),
            bp => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Cannot do a byte-level read; cursor is currently on bit {}",
                    bp
                ),
            )),
        }
    }
}

impl BitRead for ByteBufferCursor {
    fn read_bit(&mut self) -> Result<u8, Error> {
        match self.bit_pos {
            8 => {
                unreachable!("We shouldn't get here anymore!")
            }
            _ => {
                let mask = 0b10000000 >> self.bit_pos;
                let masked_byte = self.get_current_byte()? & mask;
                // Calculate the shift amount before we advance self.bit_pos
                let shift_amount = 7 - self.bit_pos;
                self.increment_bit_pos(1);
                Ok(masked_byte >> shift_amount)
            }
        }
    }

    fn read_bit_as_bool(&mut self) -> Result<bool, Error> {
        Ok(self.read_bit()? > 0)
    }

    fn read_bits_as_u8(&mut self, num_bits: usize) -> Result<u8, Error> {
        if self.bit_pos as usize + num_bits > 8 {
            // We don't support reading bits across byte boundaries
            return Err(Error::InvalidCursorPosition(format!(
                "requested to read {} bits, but cursor is currently in bit position {}, can't read bits across byte boundaries", num_bits, self.bit_pos)).into());
        }
        let mask = get_u8_mask(self.bit_pos as usize, num_bits)?;
        let masked_byte = self.get_current_byte()? & mask;
        let result = masked_byte >> (8 - num_bits as u8 - self.bit_pos);
        self.increment_bit_pos(num_bits);
        Ok(result)
    }
}

impl Write for ByteBufferCursor {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        match self.bit_pos {
            0 => self.byte_cursor.write(buf),
            bp => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Cannot do a byte-level write; cursor is currently on bit {}",
                    bp
                ),
            )),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl BitWrite for ByteBufferCursor {
    fn write_bit(&mut self, bit: u8) -> Result<(), Error> {
        let mask = !(0b10000000 >> self.bit_pos);
        let shift_amount = 7 - self.bit_pos;
        let curr_byte = self.get_current_byte_mut()?;
        // First zero out the current bit in this position
        *curr_byte &= mask;
        // Now shift the given bit so the bit to set (the LSB) is in the right place
        let shifted_bit = bit << shift_amount;
        // Now 'or' the current byte with the shifted bit
        *curr_byte |= shifted_bit;
        self.increment_bit_pos(1);
        Ok(())
    }

    fn write_bool(&mut self, b: bool) -> Result<(), Error> {
        self.write_bit(b.into())
    }

    fn write_u8_as_bits(&mut self, v: u8, num_bits: usize) -> Result<(), Error> {
        println!("Writing the right-most {} bits of {:#010b}", num_bits, v);
        let bit_pos = self.bit_pos;
        if bit_pos as usize + num_bits > 8 {
            // We don't support writing bits across byte boundaries
            return Err(Error::InvalidCursorPosition(format!(
                "requested to write {} bits, but cursor is currently in bit position {}, can't write bits across byte boundaries", num_bits, self.bit_pos)).into());
        }
        let current_byte = self.get_current_byte_mut()?;
        println!("Current byte is {:#010b}", *current_byte);

        // First clear the bits we're going to write to in the current byte
        // (We invert the mask because we want to _clear_ these bits and leave all others
        // untouched)
        let curr_byte_mask = !get_u8_mask(bit_pos as usize, num_bits)?;
        println!("Curr byte mask: {:#010b}", curr_byte_mask);
        *current_byte &= curr_byte_mask;
        println!("Current byte after masking {:#010b}", *current_byte);

        // Now clear all bits in the given value except for the ones we'll be using (the given
        // value may have more bits set than |num_bits| describes)
        let value_mask = get_u8_mask(8 - num_bits, num_bits)?;
        println!(
            "Original value: {:#010b}, value mask: {:#010b}, masked value: {:#010b}",
            v,
            value_mask,
            v & value_mask
        );
        let v = v & value_mask;

        // Now we need to shift the value to put the bits in the proper position to be written to
        // the current byte
        let v = v << (8 - bit_pos - num_bits as u8);
        *current_byte |= v;

        println!("current byte is now: {:#10b}", *current_byte);
        self.increment_bit_pos(num_bits);
        Ok(())
    }
}

impl ByteBufferExts for ByteBufferCursor {
    fn peek_u8(&self) -> Result<u8, Error> {
        Ok(self.byte_cursor.get_ref()[self.byte_cursor.position() as usize])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {
        let data: Vec<u8> = vec![1, 2, 3, 4, 5];
        let mut cursor = ByteBufferCursor::new(data);

        assert_eq!(cursor.bytes_remaining(), 5);
        let mut buf = [0; 1];
        assert!(cursor.read(&mut buf).is_ok());
        assert_eq!(buf[0], 1);
        assert_eq!(cursor.bytes_remaining(), 4);

        assert!(cursor.read(&mut buf).is_ok());
        assert_eq!(buf[0], 2);
        assert_eq!(cursor.bytes_remaining(), 3);

        assert!(cursor.read(&mut buf).is_ok());
        assert_eq!(buf[0], 3);
        assert_eq!(cursor.bytes_remaining(), 2);
    }

    #[test]
    fn test_bit_read() {
        let data: Vec<u8> = vec![0b11110000, 0b00001111];
        let mut cursor = ByteBufferCursor::new(data);

        assert_eq!(cursor.read_bit().unwrap(), 1);
        // There's only 1 'full' byte remaining
        assert_eq!(cursor.bytes_remaining(), 1);
        assert_eq!(cursor.read_bit().unwrap(), 1);
        assert_eq!(cursor.read_bit().unwrap(), 1);
        assert_eq!(cursor.read_bit().unwrap(), 1);
        assert_eq!(cursor.read_bit().unwrap(), 0);
        assert_eq!(cursor.read_bit().unwrap(), 0);
        assert_eq!(cursor.read_bit().unwrap(), 0);
        assert_eq!(cursor.read_bit().unwrap(), 0);

        assert_eq!(cursor.bytes_remaining(), 1);

        assert_eq!(cursor.read_bit().unwrap(), 0);
        // There are no 'full' bytes remaining
        assert_eq!(cursor.bytes_remaining(), 0);

        assert_eq!(cursor.read_bit().unwrap(), 0);
        assert_eq!(cursor.read_bit().unwrap(), 0);
        assert_eq!(cursor.read_bit().unwrap(), 0);
        assert_eq!(cursor.read_bit().unwrap(), 1);
        assert_eq!(cursor.read_bit().unwrap(), 1);
        assert_eq!(cursor.read_bit().unwrap(), 1);
        assert_eq!(cursor.read_bit().unwrap(), 1);
    }

    #[test]
    fn test_read_byte_in_middle_of_byte() {
        // Reading a byte while the cursor is in the middle of a byte is not allowed
        let data: Vec<u8> = vec![0b11110000, 2, 3];
        let mut cursor = ByteBufferCursor::new(data);

        cursor.read_bit().unwrap();
        let mut buf = [0; 1];
        assert!(cursor.read(&mut buf).is_err());
    }

    #[test]
    fn test_read_bool() {
        let data: Vec<u8> = vec![0b11110000];
        let mut cursor = ByteBufferCursor::new(data);

        assert_eq!(cursor.read_bit_as_bool().unwrap(), true);
        assert_eq!(cursor.read_bit_as_bool().unwrap(), true);
        assert_eq!(cursor.read_bit_as_bool().unwrap(), true);
        assert_eq!(cursor.read_bit_as_bool().unwrap(), true);
        assert_eq!(cursor.read_bit_as_bool().unwrap(), false);
        assert_eq!(cursor.read_bit_as_bool().unwrap(), false);
        assert_eq!(cursor.read_bit_as_bool().unwrap(), false);
        assert_eq!(cursor.read_bit_as_bool().unwrap(), false);
    }

    #[test]
    fn test_read_bits_as_u8() {
        let data: Vec<u8> = vec![0b11110000, 0b00001111];
        let mut cursor = ByteBufferCursor::new(data);

        assert_eq!(cursor.read_bits_as_u8(4).unwrap(), 0b1111);
        // Can't read bits beyond a byte boundary
        assert!(cursor.read_bits_as_u8(5).is_err());
        assert_eq!(cursor.read_bits_as_u8(4).unwrap(), 0b0000);

        assert_eq!(cursor.bytes_remaining(), 1);

        assert_eq!(cursor.read_bits_as_u8(4).unwrap(), 0b0000);
        assert_eq!(cursor.read_bits_as_u8(4).unwrap(), 0b1111);
    }

    #[test]
    fn test_write() {
        let data: Vec<u8> = vec![0, 0, 0, 0];
        let mut _cursor = ByteBufferCursor::new(data);
        // TODO
    }

    #[test]
    fn test_write_bit() {
        let data: Vec<u8> = vec![0, 0, 0, 0];
        let mut cursor = ByteBufferCursor::new(data);

        cursor.write_bit(1).unwrap();
        cursor.write_bit(1).unwrap();
        cursor.write_bit(1).unwrap();
        cursor.write_bool(true).unwrap();

        let data = cursor.into_vec();
        assert_eq!(data[0], 0b11110000);
    }

    #[test]
    fn test_write_bit_only_clears_bit_being_written() {
        let data: Vec<u8> = vec![0b11111111];
        let mut cursor = ByteBufferCursor::new(data);

        cursor.write_bit(0).unwrap();

        assert_eq!(0b01111111, cursor.into_vec()[0]);
    }

    #[test]
    fn test_write_u8_as_bits() {
        let data: Vec<u8> = vec![0b10101010, 0, 0, 0];
        let mut cursor = ByteBufferCursor::new(data);

        cursor.write_u8_as_bits(0b111u8, 3).unwrap();
        cursor.write_u8_as_bits(0b10101u8, 5).unwrap();

        cursor.write_u8_as_bits(2, 2).unwrap();

        let data = cursor.into_vec();
        assert_eq!(0b11110101, data[0]);
        assert_eq!(0b10000000, data[1]);
    }

    #[test]
    fn test_write_u8_as_bits_value_has_extra_data() {
        // Test that passing a value to write_u8_as_bits with 'extra' bits (bits outside the
        // right-most 'num_bits') set doesn't set those bits
        let data: Vec<u8> = vec![0];
        let mut cursor = ByteBufferCursor::new(data);

        cursor.write_u8_as_bits(0b11111111, 3).unwrap();

        assert_eq!(0b11100000, cursor.into_vec()[0]);
    }

    #[test]
    fn test_write_u8_as_bits_only_clears_bits_being_written() {
        // If we're writing bits 1-3, we shouldn't clear any other bits in the buffer
        let data: Vec<u8> = vec![0b11111111];
        let mut cursor = ByteBufferCursor::new(data);

        cursor.write_u8_as_bits(0, 3).unwrap();

        assert_eq!(0b00011111, cursor.into_vec()[0]);
    }
}
