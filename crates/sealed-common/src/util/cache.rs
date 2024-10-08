use {
    crate::error::{SealedError, SealedResult},
    sha1::Sha1,
    sha2::{Digest, Sha256},
    std::{
        collections::HashMap,
        io::{self, Read},
        path::{Path, PathBuf},
    },
    typed_path::{UnixPath, UnixPathBuf},
};

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;

#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

// Bump this if we need to invalidate all existing caches for some reason.
pub const CACHE_VERSION: usize = 0;

// This trait is implemented by things we can take a cryptographic hash of, such as strings and
// paths.
pub trait CryptoHash {
    // Compute a cryptographic hash. The guarantees:
    //   1. For all `x`, `hash_str(x)` = `hash_str(x)`.
    //   1. For all known `x` and `y`, `x` != `y` implies `hash_str(x)` != `hash_str(y)`.
    fn crypto_hash(&self) -> String;
}

impl CryptoHash for str {
    fn crypto_hash(&self) -> String {
        hex::encode(Sha256::digest(self.as_bytes()))
    }
}

impl CryptoHash for String {
    fn crypto_hash(&self) -> String {
        hex::encode(Sha256::digest(self.as_bytes()))
    }
}

#[cfg(unix)]
fn path_as_bytes(path: &Path) -> Vec<u8> {
    path.as_os_str().as_bytes().to_vec()
}

#[cfg(windows)]
fn path_as_bytes(path: &Path) -> Vec<u8> {
    path.as_os_str()
        .encode_wide()
        .flat_map(|c| c.to_le_bytes())
        .collect()
}

impl CryptoHash for Path {
    fn crypto_hash(&self) -> String {
        hex::encode(Sha256::digest(path_as_bytes(self)))
    }
}

impl CryptoHash for PathBuf {
    fn crypto_hash(&self) -> String {
        hex::encode(Sha256::digest(path_as_bytes(self)))
    }
}

impl CryptoHash for UnixPath {
    fn crypto_hash(&self) -> String {
        hex::encode(Sha256::digest(self.as_bytes()))
    }
}

impl CryptoHash for UnixPathBuf {
    fn crypto_hash(&self) -> String {
        hex::encode(Sha256::digest(self.as_bytes()))
    }
}

// Combine two strings into a hash. The guarantees:
//   1. For all `x` and `y`, `combine(x, y)` = `combine(x, y)`.
//   2. For all known `x1`, `x2`, `y1`, and `y2`,
//      `x1` != `x2` implies `combine(x1, y1)` != `combine(x2, y2)`.
//   3. For all known `x1`, `x2`, `y1`, and `y2`,
//      `y1` != `y2` implies `combine(x1, y1)` != `combine(x2, y2)`.
pub fn combine<X: CryptoHash + ?Sized, Y: CryptoHash + ?Sized>(x: &X, y: &Y) -> String {
    format!("{}{}", x.crypto_hash(), y.crypto_hash()).crypto_hash()
}

// Compute a cryptographic hash of a readable object (e.g., a file). This function does not need to
// load all the data in memory at the same time. The guarantees are the same as those of
// `crypto_hash`.
pub fn hash_read<R: Read>(input: &mut R) -> SealedResult<String> {
    let mut hasher = Sha256::new();
    io::copy(input, &mut hasher).map_err(|err| SealedError::System(err.to_string(), None))?;
    Ok(hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::{combine, hash_read, CryptoHash};
    use {
        std::{collections::HashMap, path::Path},
        typed_path::UnixPath,
    };

    #[test]
    fn hash_str_pure() {
        assert_eq!("foo".crypto_hash(), "foo".crypto_hash());
    }

    #[test]
    fn hash_str_not_constant() {
        assert_ne!("foo".crypto_hash(), "bar".crypto_hash());
    }

    #[test]
    fn hash_path_pure() {
        assert_eq!(
            Path::new("foo").crypto_hash(),
            Path::new("foo").crypto_hash(),
        );
    }

    #[test]
    fn hash_path_not_constant() {
        assert_ne!(
            Path::new("foo").crypto_hash(),
            Path::new("bar").crypto_hash(),
        );
    }

    #[test]
    fn hash_unix_path_pure() {
        assert_eq!(
            UnixPath::new("foo").crypto_hash(),
            UnixPath::new("foo").crypto_hash(),
        );
    }

    #[test]
    fn hash_unix_path_not_constant() {
        assert_ne!(
            UnixPath::new("foo").crypto_hash(),
            UnixPath::new("bar").crypto_hash(),
        );
    }

    #[test]
    fn combine_pure() {
        assert_eq!(combine("foo", "bar"), combine("foo", "bar"));
    }

    #[test]
    fn combine_first_different() {
        assert_ne!(combine("foo", "bar"), combine("foo", "baz"));
    }

    #[test]
    fn combine_second_different() {
        assert_ne!(combine("foo", "bar"), combine("baz", "bar"));
    }

    #[test]
    fn combine_concat() {
        assert_ne!(combine("foo", "bar"), combine("foob", "ar"));
    }

    #[test]
    fn hash_read_pure() {
        let mut str1 = b"foo" as &[u8];
        let mut str2 = b"foo" as &[u8];
        assert_eq!(hash_read(&mut str1).unwrap(), hash_read(&mut str2).unwrap());
    }

    #[test]
    fn hash_read_not_constant() {
        let mut str1 = b"foo" as &[u8];
        let mut str2 = b"bar" as &[u8];
        assert_ne!(hash_read(&mut str1).unwrap(), hash_read(&mut str2).unwrap());
    }
}
