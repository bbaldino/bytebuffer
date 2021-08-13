use std::io::Read;

use crate::{
    bit_read::BitRead, byte_buffer_cursor::ByteBufferCursor, byte_buffer_exts::ByteBufferExts,
    sized_buffer::SizedByteBuffer,
};

pub trait ByteBuffer: Read + BitRead + SizedByteBuffer + ByteBufferExts {}

impl ByteBuffer for ByteBufferCursor {}
