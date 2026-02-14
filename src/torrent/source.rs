use std::{fmt, ops::Deref, path::PathBuf};

use snafu::{ResultExt, Snafu, ensure};
use url::Url;

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub enum Source {
    #[cfg_attr(not(test), expect(unused))]
    Magnet(Magnet),
    #[cfg_attr(not(test), expect(unused))]
    FilePath(PathBuf),
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct Magnet(pub String);

impl fmt::Display for Magnet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Deref for Magnet {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum MagnetError {
    #[snafu(display("Failed to parse magnet link: {}", magnet))]
    ParseMagnet {
        source: url::ParseError,
        magnet: String,
    },
    #[snafu(display(r#"Invalid scheme: expected "magnet", got "{}""#, scheme))]
    InvalidScheme { scheme: String },
    #[snafu(display(r#"Missing required "xt" parameter"#))]
    MissingXtParameter,
}

impl Magnet {
    #[cfg_attr(not(test), expect(unused))]
    pub fn new(magnet: impl Into<String>) -> Result<Self, MagnetError> {
        let magnet = magnet.into();
        let magnet = Self::validate(magnet)?;
        Ok(Self(magnet))
    }

    fn validate(magnet: String) -> Result<String, MagnetError> {
        let url = Url::parse(&magnet).context(ParseMagnetSnafu { magnet: &magnet })?;

        let scheme = url.scheme();

        ensure!(scheme == "magnet", InvalidSchemeSnafu { scheme });

        ensure!(
            url.query_pairs().any(|(key, _)| key == "xt"),
            MissingXtParameterSnafu
        );

        Ok(magnet)
    }
}

#[cfg(test)]
mod test {
    use crate::torrent::InfoHash;

    use super::{Magnet, MagnetError};

    use assert_matches::assert_matches;
    use fake::{Dummy, Fake, Faker};
    use pretty_assertions::assert_eq;

    impl Dummy<Faker> for Magnet {
        fn dummy_with_rng<R: fake::Rng + ?Sized>(_config: &Faker, _rng: &mut R) -> Self {
            let info_hash = Faker.fake::<InfoHash>();
            let mut magnet = String::new();
            magnet.push_str("magnet:?xt=urn:btih:");
            magnet.push_str(&info_hash);
            magnet.push_str("&tr=http://tracker.example.com");
            Magnet(magnet)
        }
    }

    #[test]
    fn create_magnet() -> color_eyre::Result<()> {
        let magnet_str = "magnet:?xt=urn:btih:3E408407EB36C13F7C1CA999E686B1AD8C49A4CF&tr=http%3A%2F%2Fbt4.t-ru.org%2Fann%3Fmagnet&dn=%D0%94%D0%B6%D0%B0%D0%BD%D0%B3%D0%BE%20%D0%BE%D1%81%D0%B2%D0%BE%D0%B1%D0%BE%D0%B6%D0%B4%D0%B5%D0%BD%D0%BD%D1%8B%D0%B9%20%2F%20Django%20Unchained%20(%D0%9A%D0%B2%D0%B5%D0%BD%D1%82%D0%B8%D0%BD%20%D0%A2%D0%B0%D1%80%D0%B0%D0%BD%D1%82%D0%B8%D0%BD%D0%BE%20%2F%20Quentin%20Tarantino)%20%5B2012%2C%20%D0%A1%D0%A8%D0%90%2C%20%D0%B4%D1%80%D0%B0%D0%BC%D0%B0%2C%20%D0%B2%D0%B5%D1%81%D1%82%D0%B5%D1%80%D0%BD%2C%20%D0%BA%D0%BE%D0%BC%D0%B5%D0%B4%D0%B8%D1%8F%2C%20%D0%BF%D1%80%D0%B8%D0%BA%D0%BB%D1%8E%D1%87%D0%B5%D0%BD%D0%B8%D1%8F%2C%20BDRip%201080p%5D";
        let magnet = Magnet::new(magnet_str)?;
        assert_eq!(magnet.to_string(), magnet_str.to_string());
        Ok(())
    }

    #[test]
    fn validate_magnet_scheme() -> color_eyre::Result<()> {
        let bad_magnet_str = "https://example.com";
        let magnet = Magnet::new(bad_magnet_str);
        assert_matches!(
            magnet,
            Err(MagnetError::InvalidScheme { scheme }) => {
                assert_eq!(scheme, "https");
            },
            "Expected MagnetError::InvalidScheme, but got: {:?}",
            magnet
        );
        Ok(())
    }

    #[test]
    fn validate_magnet_xt() -> color_eyre::Result<()> {
        let bad_magnet_str = "magnet:?3E408407EB36C13F7C1CA999E68";
        let magnet = Magnet::new(bad_magnet_str);
        assert_matches!(
            magnet,
            Err(MagnetError::MissingXtParameter) => {},
            "Expected MangetError::MissingXtParameter, but got: {:?}",
            magnet
        );
        Ok(())
    }
}
