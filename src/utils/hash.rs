// utils/hash.rs
use sha1::{Sha1, Digest};
use hex;

pub fn sha1(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}