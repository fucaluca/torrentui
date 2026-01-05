use std::{fmt, ops::Deref};

use serde::Deserialize;
use snafu::{Snafu, ensure};

#[derive(Clone, Debug)]
#[cfg_attr(test, derive(serde::Serialize, PartialEq, Eq))]
pub struct InfoHash(pub String);

impl fmt::Display for InfoHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Deref for InfoHash {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for InfoHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let info_hash = String::deserialize(deserializer)?;
        InfoHash::new(info_hash).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Snafu)]
pub enum InfoHashError {
    #[snafu(display("Invalid length: {}. Expected 40 (SHA-1) characters", length))]
    InvalidLength { length: usize },
    #[snafu(display("Invalid characters. Must be hexadecimal (0-9, a-f)"))]
    InvalidChars,
}

impl InfoHash {
    pub fn new(info_hash: impl Into<String>) -> Result<Self, InfoHashError> {
        let info_hash = info_hash.into().to_lowercase();
        let info_hash = Self::validate(info_hash)?;

        Ok(Self(info_hash))
    }

    const SHA1_LENGTH: usize = 40;
    fn validate(info_hash: String) -> Result<String, InfoHashError> {
        ensure!(
            info_hash.len() == Self::SHA1_LENGTH,
            InvalidLengthSnafu {
                length: info_hash.len()
            }
        );
        ensure!(
            info_hash.bytes().all(|c| c.is_ascii_hexdigit()),
            InvalidCharsSnafu
        );

        Ok(info_hash)
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;

    use super::{InfoHash, InfoHashError};
    use fake::{Dummy, Faker};
    use sha1::{Digest, Sha1};

    #[cfg(test)]
    impl Dummy<Faker> for InfoHash {
        fn dummy_with_rng<R: fake::Rng + ?Sized>(_config: &Faker, rng: &mut R) -> Self {
            let mut dst = [0u8; 32];
            rng.fill_bytes(&mut dst);
            let mut hasher = Sha1::new();
            hasher.update(dst);
            let hash_bytes = hasher.finalize();
            let hash = hex::encode(hash_bytes);
            InfoHash(hash)
        }
    }

    #[test]
    fn create_info_hash() -> color_eyre::Result<()> {
        let info_hash_raw = "3E408407EB36C13F7C1CA999E686B1AD8C49A4CF";
        let info_hash = InfoHash::new(info_hash_raw)?;
        assert_eq!(info_hash.to_string(), info_hash_raw.to_lowercase());
        Ok(())
    }

    #[test]
    fn validate_info_hash_invalid_length() -> color_eyre::Result<()> {
        let info_hash_short = "3e4bf301";
        let info_hash = InfoHash::new(info_hash_short);
        assert_matches!(
            info_hash,
            Err(InfoHashError::InvalidLength { length }) => {
                assert_eq!(length, info_hash_short.len());
            },
            "Expected InfoHashError::InvalidLength, but got: {:?}",
            info_hash
        );

        let info_hash_long = "3e4bf3013e4bf3013e4bf3013e4bf3013e4bf3013e4bf301";
        let info_hash = InfoHash::new(info_hash_long);
        assert_matches!(
            info_hash,
            Err(InfoHashError::InvalidLength { length }) => {
                assert_eq!(length, info_hash_long.len());
            },
            "Expected InfoHashError::InvalidLength, but got: {:?}",
            info_hash
        );
        Ok(())
    }

    #[test]
    fn validate_info_hash_invalid_chars() -> color_eyre::Result<()> {
        let bad_info_hash = "3j4kf3013w4mf3013e4uf3013e4b+3013e4bf301";
        let info_hash = InfoHash::new(bad_info_hash);

        assert_matches!(
            info_hash,
            Err(InfoHashError::InvalidChars) => {},
            "Expected InfoHashError::InvalidChars, but got: {:?}",
            info_hash
        );
        Ok(())
    }
}
