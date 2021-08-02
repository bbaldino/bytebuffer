use std::error::Error;
use std::io::{Cursor, Read, Seek, SeekFrom};

use crate::bit_read::BitRead;
use crate::byte_buffer_exts::ByteBufferExts;
use crate::helpers::get_u8_mask;
use crate::sized_buffer::SizedByteBuffer;

/// Similar to |std::io::Cursor| but designed to keep track of a buffer of bytes where amounts less
/// than a single byte (i.e. some number of bits) can be read.
#[derive(Debug)]
pub struct ByteBufferCursor {
    byte_cursor: Cursor<Vec<u8>>,
    curr_byte: Option<u8>,
    bit_pos: u8,
}

impl ByteBufferCursor {
    pub fn new(data: Vec<u8>) -> Self {
        ByteBufferCursor {
            byte_cursor: Cursor::new(data),
            curr_byte: None,
            bit_pos: 0,
        }
    }

    fn increment_bit_pos(&mut self, num_bits: usize) {
        self.bit_pos += num_bits as u8;
        if self.bit_pos >= 8 {
            self.bit_pos %= 8;
            self.curr_byte = None;
        }
    }
}

impl Seek for ByteBufferCursor {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, std::io::Error> {
        self.bit_pos = 0;
        self.curr_byte = None;
        self.byte_cursor.seek(pos)
    }
}

impl SizedByteBuffer for ByteBufferCursor {
    fn bytes_remaining(&self) -> usize {
        self.byte_cursor.get_ref().len() - self.byte_cursor.position() as usize
    }
}

impl ByteBufferCursor {
    fn update_curr_byte(&mut self) -> Result<(), Box<dyn Error>> {
        match self.curr_byte {
            Some(_) => Ok(()),
            None => {
                let mut buf = [0; 1];
                self.byte_cursor.read_exact(&mut buf)?;
                self.curr_byte = Some(buf[0]);
                Ok(())
            }
        }
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
    fn read_bit(&mut self) -> Result<u8, Box<dyn Error>> {
        match self.bit_pos {
            8 => {
                unreachable!("We shouldn't get here anymore!")
            }
            _ => {
                self.update_curr_byte()?;
                let mask = 0b10000000 >> self.bit_pos;
                let masked_byte = self.curr_byte.unwrap() & mask;
                // Calculate the shift amount before we advance self.bit_pos
                let shift_amount = 7 - self.bit_pos;
                self.increment_bit_pos(1);
                Ok(masked_byte >> shift_amount)
            }
        }
    }

    fn read_bit_as_bool(&mut self) -> Result<bool, Box<dyn Error>> {
        Ok(self.read_bit()? > 0)
    }

    fn read_bits_as_u8(&mut self, num_bits: usize) -> Result<u8, Box<dyn Error>> {
        if self.bit_pos as usize + num_bits > 8 {
            // We don't support reading bits across byte boundaries
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Cannot read bit across byte boundaries",
            )
            .into());
        }
        self.update_curr_byte()?;
        let mask = get_u8_mask(self.bit_pos as usize, num_bits)?;
        let masked_byte = self.curr_byte.unwrap() & mask;
        let result = masked_byte >> (8 - num_bits as u8 - self.bit_pos);
        self.increment_bit_pos(num_bits);
        Ok(result)
    }
}

impl ByteBufferExts for ByteBufferCursor {
    fn peek_u8(&self) -> Result<u8, Box<dyn Error>> {
        Ok(self.byte_cursor.get_ref()[self.byte_cursor.position() as usize])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_buffer_cursor() {
        let data: Vec<u8> = vec![1, 2, 3, 0b11110000, 0b00001111];

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

        assert_eq!(cursor.read_bit().unwrap(), 1);
        assert_eq!(cursor.read_bit_as_bool().unwrap(), true);
        assert_eq!(cursor.read_bits_as_u8(4).unwrap(), 0b1100);
        // There's only 1 'full' byte remaining
        assert_eq!(cursor.bytes_remaining(), 1);

        // Can't read a byte while in the middle of a byte
        assert!(cursor.read(&mut buf).is_err());
        assert_eq!(cursor.bytes_remaining(), 1);

        assert_eq!(cursor.read_bits_as_u8(2).unwrap(), 0);
        assert_eq!(cursor.bytes_remaining(), 1);

        assert_eq!(cursor.read_bits_as_u8(8).unwrap(), 0b00001111);
    }
}
