use std::fmt::Display;

use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Default, Serialize, Deserialize)]
pub struct GitHash(pub(crate) [u8; 20]);
pub trait CompHash {
    fn compute_hash(&self) -> GitHash;
}

impl Display for GitHash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_plain_str())
    }
}

impl GitHash {
    pub fn new(data: &Vec<u8>) -> Self {
        let mut hasher = Sha1::new();
        hasher.update(data);
        let hash_result = hasher.finalize();
        let result = <[u8; 20]>::from(hash_result);
        Self(result)
    }

    pub fn new_from_bytes(bytes: &[u8]) -> Self {
        let mut hash = GitHash::default();
        hash.0.copy_from_slice(bytes);
        hash
    }

    pub fn new_from_str(s: &str) -> Self {
        let mut h = GitHash::default();
        h.0.copy_from_slice(s.as_bytes());
        h
    }

    pub fn to_plain_str(self) -> String {
        hex::encode(self.0)
    }

    pub fn to_data(&self) -> Vec<u8> {
        self.0.repeat(1)
    }
}

#[cfg(test)]
mod tests {
    use crate::server::git::hash::GitHash;

    #[test]
    fn test_hash_new() {
        // [98, 108, 111, 98] = blob
        // [32] = Space
        // [49, 52] = 14
        // [0] = \x00
        // [72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 33, 10] = Hello, World! + LF
        // let hash = Hash::new(&vec![
        //     98, 108, 111, 98, 32, 49, 52, 0, 72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108,
        //     100, 33, 10,
        // ]);
        let hash = GitHash::new_from_bytes(&[
            0x8a, 0xb6, 0x86, 0xea, 0xfe, 0xb1, 0xf4, 0x47, 0x02, 0x73, 0x8c, 0x8b, 0x0f, 0x24,
            0xf2, 0x56, 0x7c, 0x36, 0xda, 0x6d,
        ]);
        assert_eq!(
            hash.to_plain_str(),
            "8ab686eafeb1f44702738c8b0f24f2567c36da6d"
        );
    }

    #[test]
    fn test_hash_new_from_str() {
        println!("8ab686eafeb1f44702738c8b0f24f2567c36da6d");
        let hash = GitHash::new_from_str("8ab686eafeb1f44702738c8b0f24f2567c36da6d");
        assert_eq!(
            hash.to_plain_str(),
            "8ab686eafeb1f44702738c8b0f24f2567c36da6d"
        );
    }

    #[test]
    fn test_hash_to_data() {
        let hash = GitHash::new_from_str("8ab686eafeb1f44702738c8b0f24f2567c36da6d");
        assert_eq!(
            hash.to_data(),
            vec![
                0x8a, 0xb6, 0x86, 0xea, 0xfe, 0xb1, 0xf4, 0x47, 0x02, 0x73, 0x8c, 0x8b, 0x0f, 0x24,
                0xf2, 0x56, 0x7c, 0x36, 0xda, 0x6d
            ]
        );
    }

    #[test]
    fn test_hash_from_bytes() {
        let hash = GitHash::new_from_bytes(&[
            0x8a, 0xb6, 0x86, 0xea, 0xfe, 0xb1, 0xf4, 0x47, 0x02, 0x73, 0x8c, 0x8b, 0x0f, 0x24,
            0xf2, 0x56, 0x7c, 0x36, 0xda, 0x6d,
        ]);
        assert_eq!(
            hash.to_plain_str(),
            "8ab686eafeb1f44702738c8b0f24f2567c36da6d"
        );
    }
}
