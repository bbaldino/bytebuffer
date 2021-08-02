pub trait ByteBufferExts {
    fn peek_u8(&self) -> Result<u8, Box<dyn std::error::Error>>;
}
