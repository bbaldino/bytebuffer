pub trait SizedByteBuffer {
    fn bytes_remaining(&self) -> usize;
}
