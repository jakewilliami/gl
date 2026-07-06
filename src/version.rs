use anyhow::{anyhow, Error};
use std::{fmt, ops::Deref, sync::LazyLock};

// Modified semver regex from:
//   <https://semver.org/#is-there-a-suggested-regular-expression-regex-to-check-a-semver-string>
// TODO: implement less hacky version of avoiding duplicate capture group names
pub static SEMVER_PAT1: LazyLock<String> = LazyLock::new(|| {
    String::from(
        r"v(?P<major1>0|[1-9]\d*)\.(?P<minor1>0|[1-9]\d*)(?:\.(?P<patch1>0|[1-9]\d*))?(?:-(?P<prerelease1>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?P<buildmetadata1>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?",
    )
});
pub static SEMVER_PAT2: LazyLock<String> = LazyLock::new(|| {
    String::from(
        r"v(?P<major2>0|[1-9]\d*)\.(?P<minor2>0|[1-9]\d*)(?:\.(?P<patch2>0|[1-9]\d*))?(?:-(?P<prerelease2>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?P<buildmetadata2>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?",
    )
});

// Wrapper struct around semver::Version for our own Display and other implementations
#[derive(PartialEq, Clone)]
pub struct Version {
    semver: semver::Version,
}

impl Version {
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            semver: semver::Version::new(major, minor, patch),
        }
    }
}

// Convert from semver's Version
impl From<semver::Version> for Version {
    fn from(semver: semver::Version) -> Self {
        Self { semver }
    }
}

// Always format our version using a leading v
impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}", self.semver)
    }
}

// Debug in the same was as the underlying semver::Version
impl fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.semver.fmt(f)
    }
}

// This allows us to access semver::Version's underlying fields
impl Deref for Version {
    type Target = semver::Version;
    fn deref(&self) -> &Self::Target {
        &self.semver
    }
}

// Convenience trait for lenient version parsing
//   <https://github.com/knutwalker/lenient-semver/tree/3585c592#version_semver>
pub trait AsVersion {
    fn as_version(&self) -> Result<Version, Error>;
}

impl AsVersion for str {
    fn as_version(&self) -> Result<Version, Error> {
        lenient_semver::Version::parse(self)
            .map(semver::Version::from)
            .map(Version::from)
            .map_err(|e| anyhow!("invalid semver {self:?}: {e}"))
    }
}
