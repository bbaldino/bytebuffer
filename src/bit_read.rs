use crate::error::Error;

/// BitRead is similar to |std::io::Read|, but it allows reading amounts of data less than a single
/// byte
pub trait BitRead {
    fn read_bit(&mut self) -> Result<u8, Error>;
    fn read_bit_as_bool(&mut self) -> Result<bool, Error>;
    fn read_bits_as_u8(&mut self, num_bits: usize) -> Result<u8, Error>;
}
