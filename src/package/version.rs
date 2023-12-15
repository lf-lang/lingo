use serde::{Deserializer, Serializer};
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Display for Version {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for Version {
    type Err = String;

    fn from_str(s: &str) -> Result<Version, Self::Err> {
        let parts: Vec<Result<u32, &str>> = s
            .split('.')
            .map(|elm| elm.parse::<u32>().map_err(|_| elm))
            .collect();

        if parts.len() != 3 {
            return Err(format!(
                "Invalid version format: expected 3 components, got {}.",
                parts.len()
            ));
        }

        for part in &parts {
            match part {
                &Err(err) => {
                    return Err(format!(
                        "Invalid version format: expected integer, got '{}'.",
                        err
                    ))
                }
                _ => {}
            }
        }

        Ok(Version {
            major: parts[0].unwrap(),
            minor: parts[1].unwrap(),
            patch: parts[2].unwrap(),
        })
    }
}

pub(crate) fn from_version_string<'de, D: Deserializer<'de>>(d: D) -> Result<Version, D::Error> {
    struct VersionVisitor;

    impl<'de> serde::de::Visitor<'de> for VersionVisitor {
        type Value = Version;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "cannot parse version")
        }

        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Version, E> {
            Version::from_str(s)
                .map_err(|_e| E::invalid_value(serde::de::Unexpected::Str(s), &self))
        }
    }

    d.deserialize_any(VersionVisitor)
}

pub(crate) fn to_version_string<S>(version: &Version, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let serialized_string = format!("{}", version);
    s.serialize_str(&serialized_string)
}
