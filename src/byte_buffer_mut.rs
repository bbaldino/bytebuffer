use std::io::Write;

use crate::{bit_write::BitWrite, byte_buffer::ByteBuffer, byte_buffer_cursor::ByteBufferCursor};

pub trait ByteBufferMut: ByteBuffer + Write + BitWrite {}

impl ByteBufferMut for ByteBufferCursor {}
