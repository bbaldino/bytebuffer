use std::error::Error;

pub trait BitWrite {
    fn write_bit(&mut self, bit: u8) -> Result<(), Box<dyn Error>>;
    fn write_u8_as_bits(&mut self, v: u8, num_bits: usize) -> Result<(), Box<dyn Error>>;
    fn write_bool(&mut self, b: bool) -> Result<(), Box<dyn Error>>;
}
