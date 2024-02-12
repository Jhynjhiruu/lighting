use std::hash::{BuildHasher, Hasher};

#[derive(Default)]
pub struct PassthroughHasher {
    buf: [u8; 8],
}

impl Hasher for PassthroughHasher {
    fn finish(&self) -> u64 {
        u64::from_le_bytes(self.buf)
    }

    fn write(&mut self, bytes: &[u8]) {
        if bytes.len() <= 8 {
            let mut keep = self.buf[bytes.len()..].to_owned();
            keep.extend(bytes);
            self.buf = keep.try_into().unwrap();
        } else {
            self.buf = bytes.split_at(8).0.try_into().unwrap()
        }
    }
}

pub struct PassthroughHasherBuilder;

impl BuildHasher for PassthroughHasherBuilder {
    type Hasher = PassthroughHasher;

    fn build_hasher(&self) -> Self::Hasher {
        Default::default()
    }
}
