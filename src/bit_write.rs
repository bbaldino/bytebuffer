use crate::error::Error;

/// BitWrite is similar to |std::io::Write|, but it writing reading amounts of data less than a single
/// byte
pub trait BitWrite {
    fn write_bit(&mut self, bit: u8) -> Result<(), Error>;
    fn write_u8_as_bits(&mut self, v: u8, num_bits: usize) -> Result<(), Error>;
    fn write_bool(&mut self, b: bool) -> Result<(), Error>;
}
