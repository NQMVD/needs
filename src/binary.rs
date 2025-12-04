use beef::Cow;
use semver::Version as SemVersion;
use std::fmt::Display;

use crate::versions::{format_version, unknown_version};

#[derive(Debug)]
pub struct Binary<'a> {
  pub name: Cow<'a, str>,
  // TODO: use a custom version type
  pub version: Option<SemVersion>,
  pub package_manager: Option<String>,
}

impl<'a> Binary<'a> {
  pub fn new(name: Cow<'a, str>) -> Self {
    Self {
      name,
      version: None,
      package_manager: None,
    }
  }

  pub fn new_with_package_manager(name: Cow<'a, str>, package_manager: Option<String>) -> Self {
    Self {
      name,
      version: None,
      package_manager,
    }
  }
}

impl Default for Binary<'_> {
  fn default() -> Self {
    Self {
      name: Cow::borrowed(""),
      version: Some(unknown_version()),
      package_manager: None,
    }
  }
}

impl Display for Binary<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.version.is_none() {
      if let Some(ref pm) = self.package_manager {
        write!(f, "{} ? ({})", self.name, pm)
      } else {
        write!(f, "{} ?", self.name)
      }
    } else {
      let version_str = format_version(self.version.as_ref().unwrap(), false);
      if let Some(ref pm) = self.package_manager {
        write!(f, "{} {} ({})", self.name, version_str, pm)
      } else {
        write!(f, "{} {}", self.name, version_str)
      }
    }
  }
}

pub fn sort_binaries(binaries: &mut Vec<Binary>) {
  binaries.sort_by(|a, b| a.name.cmp(&b.name))
}
