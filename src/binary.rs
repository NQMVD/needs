use beef::Cow;
use semver::Version as SemVersion;
use std::fmt::Display;

use crate::versions::{format_version, unknown_version};

#[derive(Debug)]
pub struct Binary<'a> {
  pub name: Cow<'a, str>,
  // TODO: use a custom version type
  pub version: Option<SemVersion>,
}

impl<'a> Binary<'a> {
  pub fn new(name: Cow<'a, str>) -> Self {
    Self {
      name,
      version: None,
    }
  }
}

impl Default for Binary<'_> {
  fn default() -> Self {
    Self {
      name: Cow::borrowed(""),
      version: Some(unknown_version()),
    }
  }
}

impl Display for Binary<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.version.is_none() {
      write!(f, "{} ?", self.name)
    } else {
      write!(
        f,
        "{} {}",
        self.name,
        format_version(self.version.as_ref().unwrap(), false)
      )
    }
  }
}

pub fn sort_binaries(binaries: &mut Vec<Binary>) {
  binaries.sort_by(|a, b| a.name.cmp(&b.name))
}
