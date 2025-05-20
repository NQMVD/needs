use crate::binary::Binary;
use crate::error::DiscoveryError;
use log::{info, warn};
use miette::Result;

pub fn partition_binaries(
  binaries_to_check: Vec<Binary<'_>>,
) -> Result<(Vec<Binary<'_>>, Vec<Binary<'_>>)> {
  if binaries_to_check.is_empty() {
    return Err(DiscoveryError::NoBinariesSpecified.into());
  }

  let mut available: Vec<Binary> = Vec::new();
  let mut not_available: Vec<Binary> = Vec::new();

  for binary in binaries_to_check {
    let name = binary.name.as_ref();
    match which::which(name) {
      Ok(_) => {
        info!(SCOPE = "which", bin = name; "found");
        available.push(binary);
      }
      Err(err) => {
        info!(SCOPE = "which", bin = name; "not found");
        // Check if it's a permission issue or other IO error that we should report
        if let which::Error::CannotFindBinaryPath = err {
          not_available.push(binary);
        } else {
          warn!(SCOPE = "which", bin = name, error:display = err; "error during binary check");
          return Err(
            DiscoveryError::BinaryCheck {
              name: name.to_string(),
              source: std::io::Error::new(std::io::ErrorKind::Other, err),
            }
            .into(),
          );
        }
      }
    }
  }
  Ok((available, not_available))
}

#[cfg(test)]
mod tests {
  use beef::Cow;

  use super::*;

  #[test]
  fn test_partition_binaries() {
    let cargo_exists = which::which("cargo").is_ok();

    let mut bins_to_check = vec![Binary::new(Cow::borrowed(
      "hopefully_non_existent_binary_dsfargeg",
    ))];
    if cargo_exists {
      bins_to_check.push(Binary::new(Cow::borrowed("cargo")));
    }

    let result = partition_binaries(bins_to_check);
    assert!(
      result.is_ok(),
      "partition_binaries should not fail with valid input"
    );

    let (available, not_available) = result.unwrap();

    if cargo_exists {
      assert_eq!(available.len(), 1);
      assert_eq!(available[0].name, "cargo");
      assert_eq!(not_available.len(), 1);
      assert_eq!(
        not_available[0].name,
        "hopefully_non_existent_binary_dsfargeg"
      );
    } else {
      assert_eq!(available.len(), 0);
      assert_eq!(not_available.len(), 1);
      assert_eq!(
        not_available[0].name,
        "hopefully_non_existent_binary_dsfargeg"
      );
    }
  }

  #[test]
  fn test_partition_binaries_empty() {
    let result = partition_binaries(vec![]);
    assert!(
      result.is_err(),
      "partition_binaries should fail with empty input"
    );
    if let Err(err) = result {
      // This is a bit of a hack to check the error type without direct access
      let err_string = format!("{:?}", err);
      println!("Error string: {}", err_string);
      assert!(err_string.contains("needs::discovery::no_binaries"));
    }
  }
}
