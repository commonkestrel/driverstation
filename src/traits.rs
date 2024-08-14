pub trait Bytes {
    fn write_bytes(&self, out: &mut Vec<u8>);
}