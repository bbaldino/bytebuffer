# bytebuffer

bytebuffer adds helper methods to make parsing fields smaller than a single byte from a buffer easier.

`BitRead` and `BitWrite` traits are analogous to the `std::io::Read` and `std::io::Write` traits, 
but have methods that operate on the bit level instead of the byte level.

`ByteBufferCursor` is analogous to `std::io::Cursor`, but the granularity of the cursor position is
at the bit level instead of the byte level.


### examples

```
let data: Vec<u8> = vec![0b11110000];
let mut cursor = ByteBufferCursor::new(data);

assert_eq!(cursor.read_bit().unwrap(), 1);
assert_eq!(cursor.read_bit_as_bool().unwrap(), true);
assert_eq!(cursor.read_bits_as_u8(6).unwrap(), 0b110000);
```
