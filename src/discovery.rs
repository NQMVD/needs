use crate::binary::Binary;
use crate::error::DiscoveryError;
use log::{info, warn};
use miette::Result;
use std::path::Path;

/// Detect which package manager is responsible for managing a binary based on its path
fn detect_package_manager(binary_path: &Path) -> Option<String> {
  let path_str = binary_path.to_string_lossy();
  let path_str_lower = path_str.to_lowercase();

  // Check for rustup/cargo
  if path_str_lower.contains("/.cargo/bin/") || path_str_lower.contains("/.rustup/") {
    return Some("rustup".to_string());
  }

  // Check for Homebrew (macOS/Linux)
  if path_str_lower.starts_with("/opt/homebrew/") || 
     path_str_lower.starts_with("/usr/local/cellar/") ||
     (path_str_lower.contains("/usr/local/") && path_str_lower.contains("homebrew")) {
    return Some("homebrew".to_string());
  }

  // Check for npm global installs
  if path_str_lower.contains("/node_modules/.bin/") || 
     path_str_lower.contains("/npm/") ||
     path_str_lower.contains("/.npm/") ||
     path_str_lower.contains("/npm-global/") {
    return Some("npm".to_string());
  }

  // Check for Go binaries
  if path_str_lower.contains("/go/bin/") || path_str_lower.contains("/gopath/bin/") {
    return Some("go".to_string());
  }

  // Check for Python pip/pipx installs
  if path_str_lower.contains("/.local/bin/") ||
     path_str_lower.contains("/python") ||
     path_str_lower.contains("/pip/") ||
     path_str_lower.contains("/pipx/") {
    return Some("pip".to_string());
  }

  // Check for snap packages
  if path_str_lower.contains("/snap/") {
    return Some("snap".to_string());
  }

  // Check for flatpak
  if path_str_lower.contains("/flatpak/") {
    return Some("flatpak".to_string());
  }

  // Check for AppImage
  if path_str_lower.contains("/appimage/") || path_str_lower.ends_with(".appimage") {
    return Some("appimage".to_string());
  }

  // Check for system package managers (dpkg/apt on Debian/Ubuntu, etc.)
  // This should be last as it's the most generic
  if path_str_lower.starts_with("/usr/bin/") || 
     path_str_lower.starts_with("/bin/") ||
     path_str_lower.starts_with("/usr/local/bin/") ||
     path_str_lower.starts_with("/sbin/") ||
     path_str_lower.starts_with("/usr/sbin/") {
    return Some("system".to_string());
  }

  None
}

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
      Ok(path) => {
        info!(SCOPE = "which", bin = name; "found");
        let package_manager = detect_package_manager(&path);
        let updated_binary = Binary::new_with_package_manager(binary.name, package_manager);
        available.push(updated_binary);
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
  use std::path::Path;

  use super::*;

  #[test]
  fn test_detect_package_manager() {
    // Test rustup/cargo detection
    assert_eq!(
      detect_package_manager(Path::new("/home/user/.cargo/bin/cargo")),
      Some("rustup".to_string())
    );
    assert_eq!(
      detect_package_manager(Path::new("/home/user/.rustup/toolchains/stable/bin/rustc")),
      Some("rustup".to_string())
    );

    // Test system package detection
    assert_eq!(
      detect_package_manager(Path::new("/usr/bin/grep")),
      Some("system".to_string())
    );
    assert_eq!(
      detect_package_manager(Path::new("/bin/ls")),
      Some("system".to_string())
    );

    // Test homebrew detection
    assert_eq!(
      detect_package_manager(Path::new("/opt/homebrew/bin/brew")),
      Some("homebrew".to_string())
    );

    // Test npm detection
    assert_eq!(
      detect_package_manager(Path::new("/usr/local/lib/node_modules/.bin/npm")),
      Some("npm".to_string())
    );

    // Test go detection
    assert_eq!(
      detect_package_manager(Path::new("/home/user/go/bin/gofmt")),
      Some("go".to_string())
    );

    // Test pip detection
    assert_eq!(
      detect_package_manager(Path::new("/home/user/.local/bin/pip")),
      Some("pip".to_string())
    );

    // Test unknown path
    assert_eq!(
      detect_package_manager(Path::new("/some/unknown/path/binary")),
      None
    );
  }

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
      // Check that package manager was detected
      assert!(available[0].package_manager.is_some());
      assert_eq!(available[0].package_manager.as_ref().unwrap(), "rustup");
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
