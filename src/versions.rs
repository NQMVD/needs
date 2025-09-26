use crate::binary::Binary;
use crate::error::VersionError;
use beef::Cow;
use log::{debug, error, info, trace, warn};
use miette::Result;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use regex::Regex;
use semver::{BuildMetadata, Prerelease, Version as SemVersion};
use std::fmt;
use std::time::Instant;

pub fn format_version(value: &SemVersion, full_versions: bool) -> impl fmt::Display + '_ {
  struct Wrapper<'a>(&'a SemVersion, bool);

  impl fmt::Display for Wrapper<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      if self.1 {
        write!(
          f,
          "{}.{}.{}{}{}",
          self.0.major,
          self.0.minor,
          self.0.patch,
          if self.0.pre.is_empty() {
            "".to_string()
          } else {
            format!("-{}", self.0.pre)
          },
          if self.0.build.is_empty() {
            "".to_string()
          } else {
            format!("+{}", self.0.build)
          }
        )
      } else {
        write!(f, "{}.{}.{}", self.0.major, self.0.minor, self.0.patch)
      }
    }
  }

  Wrapper(value, full_versions)
}

pub fn unknown_version() -> SemVersion {
  SemVersion {
    major: 0,
    minor: 0,
    patch: 0,
    pre: Prerelease::EMPTY,
    build: BuildMetadata::new("unknown").unwrap(),
  }
}
// list known binaries that dont have a version
#[cfg(feature = "version-retrieval")]
pub fn known_binaries() -> Vec<Cow<'static, str>> {
  vec![
    // shell builtins
    Cow::borrowed("ls"),
    Cow::borrowed("cd"),
    Cow::borrowed("pwd"),
    Cow::borrowed("echo"),
    Cow::borrowed("cat"),
    Cow::borrowed("find"),
    Cow::borrowed("awk"), // no semver, just the date
    Cow::borrowed("sed"),
    Cow::borrowed("cut"),
    //Cow::borrowed("sort"), // 2.3-Apple (195)
    Cow::borrowed("uniq"),
    Cow::borrowed("wc"),
    Cow::borrowed("head"),
    Cow::borrowed("tail"),
    Cow::borrowed("chmod"),
    Cow::borrowed("chown"),
    Cow::borrowed("ln"),
    Cow::borrowed("mkdir"),
    Cow::borrowed("rmdir"),
    Cow::borrowed("rm"),
    Cow::borrowed("cp"),
    Cow::borrowed("mv"),
    Cow::borrowed("touch"),
    Cow::borrowed("ssh"),
    Cow::borrowed("nice"),
  ]
}

#[cfg(feature = "version-retrieval")]
pub static _SEMVER_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"(?:^|\s)((?:[<>=~^]|>=|<=)?)(?:v)?((?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:\.(?:0|[1-9]\d*))?(?:-(?:[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\+(?:[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?)").unwrap()
});

#[cfg(feature = "version-retrieval")]
pub static VER_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"(\d+\.\d+(?:\.\d+)?(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?)").unwrap()
});

#[cfg(feature = "version-retrieval")]
pub fn clean_version_string(version_str: &str) -> String {
  // Split the version into main components: version numbers, prerelease, and build metadata
  let parts: Vec<&str> = version_str.splitn(2, '+').collect();
  let version_and_prerelease = parts[0];
  let build_metadata = if parts.len() > 1 {
    Some(parts[1])
  } else {
    None
  };

  let pre_parts: Vec<&str> = version_and_prerelease.splitn(2, '-').collect();
  let version_numbers = pre_parts[0];
  let prerelease = if pre_parts.len() > 1 {
    Some(pre_parts[1])
  } else {
    None
  };

  // Clean version numbers by removing leading zeros
  let mut cleaned_version = Vec::new();
  for segment in version_numbers.split('.') {
    if segment.starts_with('0') && segment.len() > 1 {
      // Remove leading zeros (e.g., "01" -> "1")
      let cleaned = segment.trim_start_matches('0');
      if cleaned.is_empty() {
        cleaned_version.push("0".to_string()); // All zeros case
      } else {
        cleaned_version.push(cleaned.to_string());
      }
    } else {
      cleaned_version.push(segment.to_string());
    }
  }

  // Ensure we have at least 2 segments (major.minor)
  while cleaned_version.len() == 2 {
    cleaned_version.push("0".to_string());
  }

  // Clean prerelease by removing invalid characters
  let cleaned_prerelease = prerelease
    .map(|pre| {
      let re = Regex::new(r"[^0-9A-Za-z-.]").unwrap();
      let replaced = re.replace_all(pre, "");

      // Split by dots and remove any empty segments
      let segments: Vec<&str> = replaced.split('.').filter(|s| !s.is_empty()).collect();
      segments.join(".")
    })
    .filter(|s| !s.is_empty());

  // Clean build metadata by removing invalid characters
  let cleaned_build = build_metadata
    .map(|build| {
      let re = Regex::new(r"[^0-9A-Za-z-.]").unwrap();
      let replaced = re.replace_all(build, "");

      // Split by dots and remove any empty segments
      let segments: Vec<&str> = replaced.split('.').filter(|s| !s.is_empty()).collect();
      segments.join(".")
    })
    .filter(|s| !s.is_empty());

  // Construct the final cleaned version string
  let mut result = cleaned_version.join(".");

  if let Some(pre) = cleaned_prerelease {
    result.push_str(&format!("-{}", pre));
  }

  if let Some(build) = cleaned_build {
    result.push_str(&format!("+{}", build));
  }

  result
}

#[cfg(feature = "version-retrieval")]
pub fn execute_binary<'a>(binary_name: &str) -> Result<Cow<'a, str>> {
  // TODO: log the frequency of these

  use xshell::{Shell, cmd};

  let version_flags = ["--version", "-v", "-version", "-V"];

  for flag in &version_flags {
    let sh = match Shell::new() {
      Ok(s) => s,
      Err(e) => {
        error!(error:display = e; "Error creating shell");
        return Err(
          VersionError::Execution {
            name: binary_name.to_string(),
            source: std::io::Error::other(e),
          }
          .into(),
        );
      }
    };
    let name = binary_name;
    let command = cmd!(sh, "{name}").ignore_stderr().arg(flag);
    trace!(command:display = command; "Running command");

    match command.read() {
      Ok(output) => return Ok(Cow::owned(output)),
      Err(err) => {
        trace!(SCOPE = binary_name, err:display = err; "flag didn't work, error for tracing:");
        debug!(SCOPE = binary_name, flag = flag; "flag didn't work, trying next...");
        // Continue trying other flags - we only report error if all flags fail
        continue;
      }
    };
  }
  info!(scope = binary_name; "no version flag found, see --help or check builtins");
  Err(
    VersionError::Execution {
      name: binary_name.to_string(),
      source: std::io::Error::new(std::io::ErrorKind::NotFound, "No valid version flag found"),
    }
    .into(),
  )
}

#[cfg(feature = "version-retrieval")]
pub fn extract_version<'a>(
  output: Cow<'a, str>,
  binary_name: Cow<'a, str>,
) -> Result<Cow<'a, str>> {
  let lines = output
    .lines()
    .filter(|l| l.chars().any(|c| c.is_ascii_digit()))
    .collect::<Vec<_>>();

  trace!(SCOPE = binary_name.as_ref(), lines:debug = lines; "filtered lines:");

  for line in &lines {
    // TODO: check all lines for more info (deno e.g.)
    if let Some(captures) = VER_REGEX.captures(line) {
      let version_string = &captures[1];
      info!(SCOPE = binary_name.as_ref(), version:debug = version_string, line = line; "version found");

      let version_string = clean_version_string(version_string);
      debug!(SCOPE = binary_name.as_ref(), version:debug = version_string; "cleaned version");

      // #[cfg(not(test))]
      // {
      //   let mut file = std::fs::OpenOptions::new()
      //     .append(true)
      //     .create(true)
      //     .open("version_output.txt")
      //     .unwrap();
      //   writeln!(file, "(\"{}\", \"{}\"),", line, version_string).unwrap();
      // }

      return Ok(Cow::owned(version_string.to_string()));
    }
  }

  warn!(SCOPE = binary_name.as_ref(), output = output.as_ref(); "No valid version found in the output");
  Err(
    VersionError::VersionParse {
      name: binary_name.to_string(),
      output: output.to_string(),
    }
    .into(),
  )
}

#[cfg(feature = "version-retrieval")]
pub fn get_version(binary_name: Cow<str>) -> Result<Option<SemVersion>> {
  let now = Instant::now();
  let output = execute_binary(binary_name.as_ref());
  trace!(
      SCOPE = binary_name.as_ref(),
      ms = now.elapsed().as_millis();
      "calling binary took"
  );
  match output {
    Ok(output) => {
      trace!(
        SCOPE = binary_name.as_ref(), output = output.as_ref(); "command output");
      let version_string = extract_version(output.clone(), binary_name.clone())?;

      match SemVersion::parse(version_string.as_ref()) {
        Ok(v) => {
          debug!(SCOPE = binary_name.as_ref(), version:debug = v; "version parsed");
          Ok(Some(v))
        }
        Err(e) => {
          warn!(SCOPE = binary_name.as_ref(), error:display = e; "error parsing version");
          Err(
            VersionError::SemverParse {
              version_string: version_string.to_string(),
              source: e,
            }
            .into(),
          )
        }
      }
    }
    Err(e) => {
      // For binaries known not to have version flags, this is expected
      if known_binaries().contains(&binary_name) {
        Ok(None)
      } else {
        Err(e)
      }
    }
  }
}

#[cfg(feature = "version-retrieval")]
pub fn get_versions_for_bins(binaries: Vec<Binary>) -> Vec<Binary> {
  binaries
    // .into_iter()
    .into_par_iter()
    .map(|binary| {
      // filter out known binaries that don't have a version
      if known_binaries().contains(&binary.name) {
        return Binary {
          name: binary.name,
          version: None,
          package_manager: binary.package_manager,
        };
      }

      let version = match get_version(binary.name.clone()) {
        Ok(v) => v,
        Err(e) => {
          // Log the error but don't fail the entire process
          warn!(SCOPE = binary.name.as_ref(), error:display = e; "error getting version");
          None
        }
      };

      Binary {
        name: binary.name,
        version,
        package_manager: binary.package_manager,
      }
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use crate::{cli::Cli, io::get_binary_names};

  use super::*;

  #[cfg(feature = "version-retrieval")]
  #[test]
  fn test_version_regex() {
    let test_strings = [
      ("1.0.0", "1.0.0"),
      // eza
      ("v1.0.0", "1.0.0"),
      // luajit
      ("2.1.1713773202", "2.1.1713773202"),
      // viddy
      ("1.3.0-VERGEN_IDEMPOTENT_OUTPUT", "1.3.0-VERGEN"),
      // helix
      ("25.01.01", "25.1.1"),
      // love2d
      ("11.5", "11.5.0"),
    ];

    for (output, expected) in test_strings {
      let captures = VER_REGEX.captures(output);
      assert!(captures.is_some(), "Failed to match: {}", output);
      if let Some(captures) = captures {
        let version_str = Cow::from(&captures[1]);

        let version_string_cleaned = clean_version_string(version_str.as_ref());
        println!(
          "Version string cleaned: {} -> {}",
          version_str, version_string_cleaned
        );

        let version = SemVersion::parse(&version_string_cleaned);
        let expected = SemVersion::parse(expected).unwrap();
        match version {
          Ok(v) => {
            assert_eq!(v, expected);
          }
          Err(e) => panic!(
            "Failed to parse version: {}\nExpected: {}\n     Got: {}",
            e, expected, version_str
          ),
        }
      }
    }
  }

  #[cfg(feature = "version-retrieval")]
  #[test]
  fn test_extract_version() {
    let test_strings = [
      ("bacon 3.10.0", "3.10.0"),
      ("bat 0.25.0 (25f4f96)", "0.25.0"),
      ("boss 0.6.2", "0.6.2"),
      ("bottom 0.10.2", "0.10.2"),
      ("cargo 1.85.0 (d73d2caf9 2024-12-31)", "1.85.0"),
      (
        "deno 2.2.2 (stable, release, aarch64-apple-darwin)",
        "2.2.2",
      ),
      ("eget version v1.3.4", "1.3.4"),
      ("v0.20.22 [+git]", "0.20.22"),
      ("fd 10.2.0", "10.2.0"),
      ("glow version 2.0.0", "2.0.0"),
      ("gum version 0.15.2", "0.15.2"),
      ("helix 25.01.1 (e7ac2fcd)", "25.1.1"),
      ("LOVE 11.5 (Mysterious Mysteries)", "11.5.0"),
      (
        "Lua 5.2.4  Copyright (C) 1994-2015 Lua.org, PUC-Rio",
        "5.2.4",
      ),
      (
        "LuaJIT 2.1.1713773202 -- Copyright (C) 2005-2023 Mike Pall. https://luajit.org/",
        "2.1.1713773202",
      ),
      ("pls 0.0.1-beta.9", "0.0.1-beta.9"),
      ("pueue 3.4.1", "3.4.1"),
      ("ripgrep 14.1.1 (rev 4649aa9700)", "14.1.1"),
      ("taplo 0.9.3", "0.9.3"),
      ("tealdeer 1.7.1", "1.7.1"),
      ("topgrade 16.0.2", "16.0.2"),
      (
        "viddy 1.3.0-VERGEN_IDEMPOTENT_OUTPUT (2024-11-29)",
        "1.3.0-VERGEN",
      ),
      ("Yazi 25.2.11 (ce9092e 2025-02-11)", "25.2.11"),
      ("zoxide 0.9.7", "0.9.7"),
    ];
    for (output, expected) in test_strings {
      let version = extract_version(Cow::borrowed(output), Cow::borrowed("cargo"));
      assert!(version.is_ok(), "Failed to match: {}", output);
      if let Ok(version) = version {
        let version_string_cleaned = clean_version_string(version.as_ref());
        println!(
          "Version string cleaned: {} -> {}",
          version.as_ref(),
          version_string_cleaned
        );

        let version = SemVersion::parse(&version_string_cleaned);
        let expected = SemVersion::parse(expected).unwrap();
        match version {
          Ok(v) => {
            assert_eq!(v, expected);
          }
          Err(e) => panic!(
            "Failed to parse version: {}\nExpected: {}\n     Got: {}",
            e, expected, version_string_cleaned
          ),
        }
      }
    }
  }

  #[cfg(feature = "version-retrieval")]
  #[test]
  fn test_extract_version_no_match_feature_on() {
    let output = "no version found\n";
    let version = extract_version(Cow::borrowed(output), Cow::borrowed("test_binary"));
    assert!(
      version.is_err(),
      "Should return an error when no version is found"
    );
    if let Err(e) = version {
      let err_string = format!("{:?}", e);
      println!("Error string: {}", err_string);
      assert!(err_string.contains("needs::version::parse_failed"));
    }
  }

  #[test]
  fn test_get_binary_names_from_args() {
    let cli = {
      let bins = Some(vec!["bat".to_string(), "btm".to_string()]);
      #[cfg(feature = "version-retrieval")]
      {
        Cli {
          bins,
          quiet: false,
          verbosity: 0,
          no_versions: false,
          full_versions: false,
        }
      }
      #[cfg(not(feature = "version-retrieval"))]
      {
        Cli {
          bins,
          quiet: false,
          verbosity: 0,
        }
      }
    };
    let binaries = get_binary_names(&cli).unwrap();
    assert_eq!(binaries.len(), 2);
    assert_eq!(binaries[0].name, Cow::borrowed("bat"));
    assert_eq!(binaries[1].name, Cow::borrowed("btm"));
  }

  #[cfg(feature = "version-retrieval")]
  #[test]
  fn test_run_command_with_version_feature_on() {
    let binary_name = "cargo"; // A binary likely present in dev environment
    let version_output = execute_binary(binary_name);
    println!("Version output for {}: {:?}", binary_name, version_output);
    if which::which(binary_name).is_ok() {
      // Only assert if cargo is actually found
      assert!(
        version_output.is_ok(),
        "cargo --version should return Ok result if cargo is present"
      );
      if let Ok(vo) = version_output {
        assert!(
          vo.to_lowercase().contains("cargo"),
          "cargo version output should contain 'cargo'"
        );
      }
    } else {
      println!(
        "Skipping {} version check as it's not found in PATH",
        binary_name
      );
    }
  }
}
