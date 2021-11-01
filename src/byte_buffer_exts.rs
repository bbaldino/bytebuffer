use crate::error::Error;

pub trait ByteBufferExts {
    fn peek_u8(&self) -> Result<u8, Error>;
}
