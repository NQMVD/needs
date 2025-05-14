#![allow(clippy::never_loop)] // because of the files that can vary depending on the system
#![allow(unused_imports)]
use anyhow::{Result, bail, ensure};
use atty::{Stream, is};
use beef::Cow;
use chrono::Local;
use clap::Parser;
use colored::Colorize;
use log::kv::*;
use log::*;
use std::{collections::BTreeMap, fmt::Display, time::Instant};
// add semver
use semver::{BuildMetadata, Prerelease, Version};
// add miette

#[cfg(feature = "version-retrieval")]
use once_cell::sync::Lazy;
#[cfg(feature = "version-retrieval")]
use rayon::prelude::*;
#[cfg(feature = "version-retrieval")]
use regex::Regex;
#[cfg(feature = "version-retrieval")]
use xshell::{Shell, cmd};

// TODO: custom error types for all cases

#[derive(Debug)]
struct Binary<'a> {
  name: Cow<'a, str>,
  #[cfg(feature = "version-retrieval")]
  version: Version,
}

impl<'a> Binary<'a> {
  fn new(name: Cow<'a, str>, version: Version) -> Self {
    Self { name, version }
  }
}

impl Default for Binary<'_> {
  fn default() -> Self {
    Self {
      name: Cow::borrowed(""),
      version: unknown_version(),
    }
  }
}

impl Display for Binary<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} {}", self.name, self.version)
  }
}

fn unknown_version() -> Version {
  Version {
    major: 0,
    minor: 0,
    patch: 0,
    pre: Prerelease::EMPTY,
    build: BuildMetadata::new("unknown").unwrap(),
  }
}

fn partition_binaries(binaries_to_check: Vec<Binary<'_>>) -> (Vec<Binary<'_>>, Vec<Binary<'_>>) {
  let mut available: Vec<Binary> = Vec::new();
  let mut not_available: Vec<Binary> = Vec::new();

  for binary in binaries_to_check {
    let name = binary.name.as_ref();
    if which::which(name).is_ok() {
      debug!(bin = name; "found");
      available.push(binary);
    } else {
      debug!(bin = name; "not found");
      not_available.push(binary);
    }
  }
  (available, not_available)
}

#[cfg(feature = "version-retrieval")]
fn run_command_with_version<'a>(binary_name: &str) -> Option<Cow<'a, str>> {
  // TODO: log the frequency of these

  let version_flags = ["--version", "-v", "-version", "-V"];

  for flag in &version_flags {
    let sh = match Shell::new() {
      Ok(s) => s,
      Err(e) => {
        debug!(error:display = e; "Error creating shell");
        return None;
      }
    };
    let name = binary_name;
    let command = cmd!(sh, "{name}").ignore_stderr().arg(flag);
    trace!(command:display = command; "Running command");

    match command.read() {
      Ok(output) => return Some(Cow::owned(output)),
      Err(_) => {
        debug!(bin = binary_name, flag = flag; "flag didn't work, trying next...");
        continue;
      }
    };
  }
  debug!(bin = binary_name; "no version found");
  None
}

#[cfg(not(feature = "version-retrieval"))]
fn run_command_with_version<'a>(_binary_name: &str) -> Option<Cow<'a, str>> {
  None // Always return None if feature is disabled
}

#[cfg(feature = "version-retrieval")]
fn extract_version(output: Cow<str>) -> Result<Version> {
  // 1.2.3 999.999.999 >1.2.3 >=1.2.3 =1.2.3 <1.2.3 1.2.3-nightly 1.2.3-alpha 1.2.3-beta
  // TODO: implement

  let lines = output
    .lines()
    .filter(|l| l.chars().any(|c| c.is_ascii_digit()))
    .collect::<Vec<_>>();

  debug!(lines:debug = lines; "filtered lines:");

  for line in &lines {
    // split on whitespace and filter again
    let version_str = line
      .split_whitespace()
      .filter(|s| s.chars().any(|c| c.is_ascii_digit()))
      .collect::<Vec<_>>()
      .join(" ");
    debug!(version_str = version_str.as_str(), line = line; "version candidate");

    match Version::parse(version_str.as_str()) {
      Ok(version) => {
        debug!(version = version.to_string().as_str(), line = line; "version found");
        return Ok(version); // Return the parsed version
      }
      Err(e) => {
        debug!(error:display = e, line = line; "failed to parse version");
        continue;
      }
    }
  }

  // If we reach here, it means we didn't find a valid version
  warn!(output = output.as_ref(); "No valid version found in the output");
  bail!("No valid version found in the output");
}

// #[cfg(feature = "version-retrieval")]
// fn parse_version(output: Cow<str>) -> String {
//   let mut version_to_return: String = "?".into();
//   let lines = output.lines().collect::<Vec<_>>();

//   for line in &lines {
//     // jump to first digit character
//     let first_digit = line
//       .chars()
//       .position(|c| c.is_digit(10))
//       .unwrap_or(line.len());
//     match Version::parse(line) {
//       Ok(version) => {
//         debug!(version = version.to_string().as_str(), line = line; "version found");
//       }
//       Err(e) => {
//         debug!(error:display = e, line = line; "failed to parse version");
//         continue;
//       }
//     }
//   }

//   version_to_return // Return the extracted version or "?" at the end
// }

#[cfg(not(feature = "version-retrieval"))]
fn extract_version(_output: Cow<str>) -> String {
  "?".into() // Always return "?" if feature is disabled
}

fn get_binary_names<'a>(cli: &Cli) -> Result<Vec<Binary<'a>>> {
  match cli.bins.clone() {
    Some(bins) => {
      let binaries: Vec<Binary> = bins
        .iter()
        .map(|name| Binary::new(Cow::owned(name.clone()), unknown_version()))
        .collect::<Vec<Binary>>();
      Ok(binaries)
    }
    None => {
      let file_paths = ["needsfile", ".needsfile", "needs", ".needs"];

      for path in file_paths {
        // Attempt to read from the first successful file path
        if let Ok(content) = std::fs::read_to_string(path) {
          if content.trim().is_empty() {
            debug!(path = path; "needsfile found but it is empty, trying next.");
            continue; // Try next file if this one is empty
          }
          let names: Vec<String> = content.split_whitespace().map(|s| s.to_owned()).collect();

          // This ensure might be redundant if content.trim().is_empty() check is robust
          ensure!(
            !names.is_empty(),
            "needsfile at '{}' is effectively empty after parsing.",
            path
          );
          let binaries = names
            .iter()
            .map(|name_str| Binary::new(Cow::owned(name_str.clone()), unknown_version()))
            .collect::<Vec<Binary>>();
          return Ok(binaries);
        } else {
          debug!(path = path; "Failed to read or find needsfile, trying next.");
        }
      }
      bail!(
        "No binaries specified and no non-empty needsfile (needsfile, .needsfile, needs, .needs) found."
      );
    }
  }
}

fn sort_binaries(binaries: &mut Vec<Binary>) {
  binaries.sort_by(|a, b| a.name.cmp(&b.name))
}

#[cfg(feature = "version-retrieval")]
fn get_versions(binaries: Vec<Binary>) -> Vec<Binary> {
  binaries
    .into_par_iter()
    .map(|binary| {
      let now = Instant::now();
      match run_command_with_version(binary.name.as_ref()) {
        Some(output) => {
          debug!(
              bin = binary.name.as_ref(),
              ms = now.elapsed().as_millis();
              "calling binary took"
          );
          debug!(
            bin = binary.name.as_ref(), output = output.as_ref(); "command output for");
          let version: Version = extract_version(output.clone());
          // let version = parse_version(output.clone());
          Binary::new(binary.name, version)
        }
        None => {
          debug!(
              bin = binary.name.as_ref(),
              ms = now.elapsed().as_millis();
              "calling binary took"
          );
          Binary::new(binary.name, unknown_version())
        }
      }
    })
    .collect()
}

#[cfg(not(feature = "version-retrieval"))]
fn get_versions(binaries: Vec<Binary>) -> Vec<Binary> {
  binaries // Just return them as is, versions will remain "?"
}

fn print_center_aligned(binaries: Vec<Binary>, max_len: usize, always_found: bool) -> Result<()> {
  for bin in &binaries {
    let padding_needed = max_len.saturating_sub(bin.name.len());
    let padding = " ".repeat(padding_needed);
    let version_display = if always_found {
      "found".to_string()
    } else {
      bin.version.to_string()
    };
    println!("{}{} {}", padding, bin.name.bright_green(), version_display);
  }
  Ok(())
}

struct Collect<'kvs>(BTreeMap<Key<'kvs>, Value<'kvs>>);

impl<'kvs> VisitSource<'kvs> for Collect<'kvs> {
  fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), kv::Error> {
    self.0.insert(key, value);
    Ok(())
  }
}

fn setup_logger(verbosity: u8) -> Result<(), fern::InitError> {
  let log_level = match verbosity {
    0 => LevelFilter::Error,
    1 => LevelFilter::Warn,
    2 => LevelFilter::Info,
    3 => LevelFilter::Debug,
    _ => LevelFilter::Trace,
  };

  // Gum log colors
  const TRACE_RGB: (u8, u8, u8) = (144, 144, 144);
  const DEBUG_RGB: (u8, u8, u8) = (95, 96, 255);
  const INFO_RGB: (u8, u8, u8) = (99, 254, 218);
  const WARN_RGB: (u8, u8, u8) = (219, 254, 143);
  const ERROR_RGB: (u8, u8, u8) = (254, 95, 136);

  fern::Dispatch::new()
    .format(move |out, message, record| {
      let time = Local::now().format("%H:%M");
      let lvl_plain = format!("{:>5}", record.level());
      let (r, g, b) = match record.level() {
        Level::Trace => TRACE_RGB,
        Level::Debug => DEBUG_RGB,
        Level::Info => INFO_RGB,
        Level::Warn => WARN_RGB,
        Level::Error => ERROR_RGB,
      };
      let lvl_colored = lvl_plain.truecolor(r, g, b);

      let has_kvs = record.key_values().count() > 0;
      if has_kvs {
        let mut visitor = Collect(BTreeMap::new());
        let _ = record.key_values().visit(&mut visitor);
        let collected = visitor.0;
        let (single, multiline) = collected
          .iter()
          .partition::<Vec<_>, _>(|(_, v)| !v.to_string().contains('\n'));

        let formatted_pairs = single
          .iter()
          .map(|(k, v)| {
            let k = k.to_string().as_str().truecolor(142, 142, 142);
            let eq = "=".truecolor(142, 142, 142);
            let v = v.to_string();
            format!("{k}{eq}{v}")
          })
          .collect::<Vec<_>>()
          .join(" ");

        let formatted_multiline_pairs = multiline
          .iter()
          .map(|(k, v)| {
            let k = k.to_string().as_str().truecolor(142, 142, 142);
            let eq = "=".truecolor(142, 142, 142);
            let vb = "┊".truecolor(142, 142, 142);
            let v = v.to_string();

            format!(
              "{k}{eq}\n  {vb} {}",
              v.to_string()
                .lines()
                .collect::<Vec<_>>()
                .join(format!("\n  {vb} ").as_str())
            )
          })
          .collect::<Vec<_>>()
          .join("\n  ");

        if multiline.is_empty() {
          out.finish(format_args!(
            "{time} {lvl_colored} {message} {formatted_pairs}"
          ))
        } else {
          out.finish(format_args!(
            "{time} {lvl_colored} {message} {formatted_pairs}\n  {formatted_multiline_pairs}"
          ))
        }
      } else {
        out.finish(format_args!("{time} {lvl_colored} {message}"))
      }
    })
    .level(log_level)
    .chain(std::io::stdout())
    .apply()?;
  Ok(())
}

pub const CLAP_STYLING: clap::builder::styling::Styles = clap::builder::styling::Styles::styled()
  .header(clap_cargo::style::HEADER)
  .usage(clap_cargo::style::USAGE)
  .literal(clap_cargo::style::LITERAL)
  .placeholder(clap_cargo::style::PLACEHOLDER)
  .error(clap_cargo::style::ERROR)
  .valid(clap_cargo::style::VALID)
  .invalid(clap_cargo::style::INVALID);

#[derive(Parser)]
#[command(version, about, long_about)]
#[command(styles = CLAP_STYLING)]
/// Check if given bin(s) are available in the PATH
///
/// If no binaries are specified, it will look for a file named `needsfile` or `.needsfile` in the current directory.
struct Cli {
  /// List of binaries to check
  bins: Option<Vec<String>>,

  /// stay quiet, exit with 0 or 1
  #[clap(short, long)]
  quiet: bool,

  /// Verbosity level (can be repeated, e.g. -vvv)
  #[clap(short, long, action = clap::ArgAction::Count)]
  verbosity: u8,

  #[cfg(feature = "version-retrieval")]
  /// don't check for versions
  #[clap(short, long)]
  no_versions: bool,
}

fn main() -> Result<()> {
  let cli = Cli::parse();

  setup_logger(cli.verbosity).expect("Failed to set up logger");

  debug!("Starting needs with verbosity level {}", cli.verbosity);
  debug!("passed bins: {:?}", cli.bins);
  debug!("quiet: {:?}", cli.quiet);
  debug!("version retrieval: {:?}", cli.no_versions);
  debug!("atty stdout: {}", is(Stream::Stdout));
  debug!("atty stderr: {}", is(Stream::Stderr));
  debug!("atty stdin: {}", is(Stream::Stdin));

  let binaries_from_source = get_binary_names(&cli)?;
  ensure!(!binaries_from_source.is_empty(), "binary sources are empty");

  // Calculate max_name_len from all initial binaries for consistent padding
  let global_max_name_len = binaries_from_source
    .iter()
    .map(|bin| bin.name.len())
    .max()
    .unwrap_or(0);

  let (mut available, mut not_available) = partition_binaries(binaries_from_source);

  sort_binaries(&mut available);
  sort_binaries(&mut not_available);

  let stay_quiet = cli.quiet;

  if stay_quiet {
    if !not_available.is_empty() {
      std::process::exit(1);
    }
    info!("quiet exit");
    std::process::exit(0);
  }

  let needs_separator = !available.is_empty() && !not_available.is_empty();

  let should_skip_versions = {
    #[cfg(feature = "version-retrieval")]
    {
      cli.no_versions
    }
    #[cfg(not(feature = "version-retrieval"))]
    {
      true
    }
  };

  if !available.is_empty() {
    if should_skip_versions {
      print_center_aligned(available, global_max_name_len, true)?;
    } else {
      #[cfg(feature = "version-retrieval")]
      {
        let mut bins_with_versions = get_versions(available);
        sort_binaries(&mut bins_with_versions);
        print_center_aligned(bins_with_versions, global_max_name_len, false)?;
      }
      #[cfg(not(feature = "version-retrieval"))]
      {
        // This case should not be reached if should_skip_versions is true,
        // but as a fallback, print with 'found'
        print_center_aligned(available, global_max_name_len, true)?;
      }
    }
  }

  if needs_separator {
    let padding = " ".repeat(global_max_name_len.saturating_sub(1));
    println!("{}---", padding);
  }

  if !not_available.is_empty() {
    for binary in not_available {
      // Align "not found" with the names of "found" items.
      let padding_needed = global_max_name_len.saturating_sub(binary.name.len());
      let padding = " ".repeat(padding_needed);
      println!("{}{} not found", padding, binary.name.bright_red());
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[cfg(feature = "version-retrieval")]
  #[test]
  fn test_extract_version_feature_on() {
    let output = "1.2.3\n";
    let version = extract_version(Cow::borrowed(output));
    assert_eq!(version.to_string(), "1.2.3");

    let output = "100.200.300\n";
    let version = extract_version(Cow::borrowed(output));
    assert_eq!(version.to_string(), "100.200.300");

    let output = "1.2.3-nightly\n";
    let version = extract_version(Cow::borrowed(output));
    assert_eq!(version.to_string(), "1.2.3-nightly");
  }

  #[cfg(feature = "version-retrieval")]
  #[test]
  fn test_extract_version_no_match_feature_on() {
    let output = "no version found\n";
    let version = extract_version(Cow::borrowed(output));
    assert_eq!(version.to_string(), "0.0.0+unknown");
  }

  #[cfg(not(feature = "version-retrieval"))]
  #[test]
  fn test_extract_version_feature_off() {
    let output = "v1.2.3\n";
    let version = extract_version(Cow::borrowed(output));
    assert_eq!(version, "?");
  }

  fn create_test_cli(bins: Option<Vec<String>>, no_versions_flag: bool) -> Cli {
    Cli {
      bins,
      quiet: false,
      verbosity: 0,
      #[cfg(feature = "version-retrieval")]
      no_versions: no_versions_flag,
    }
  }

  #[test]
  fn test_get_binary_names_from_args() {
    let cli = create_test_cli(Some(vec!["bat".to_string(), "btm".to_string()]), false);
    let binaries = get_binary_names(&cli).unwrap();
    assert_eq!(binaries.len(), 2);
    assert_eq!(binaries[0].name, Cow::borrowed("bat"));
    assert_eq!(binaries[1].name, Cow::borrowed("btm"));
  }

  // To test get_binary_names with files, you'd need to create mock files.
  // For simplicity, this is omitted here but would be good for comprehensive testing.
  // Example:
  // fn setup_needs_file(content: &str) -> std::path::PathBuf { ... }
  // fn cleanup_needs_file(path: &std::path::PathBuf) { ... }

  #[cfg(feature = "version-retrieval")]
  #[test]
  fn test_run_command_with_version_feature_on() {
    // This test assumes a command like `echo` is universally available.
    // For real binaries like `bat`, it's better to mock or ensure presence.
    // For this example, let's assume a 'true' command or similar exists
    // or mock xshell if this test needs to be fully hermetic.
    // For now, we'll test with a common command if possible, or skip if too flaky.
    // For instance, if 'cargo' is available in the test environment:
    let binary_name = "cargo"; // A binary likely present in dev environment
    let version_output = run_command_with_version(binary_name);
    println!("Version output for {}: {:?}", binary_name, version_output);
    if which::which(binary_name).is_ok() {
      // Only assert if cargo is actually found
      assert!(
        version_output.is_some(),
        "cargo --version should return some output if cargo is present"
      );
      if let Some(vo) = version_output {
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

  #[cfg(not(feature = "version-retrieval"))]
  #[test]
  fn test_run_command_with_version_feature_off() {
    let binary_name = "any_binary";
    let version_output = run_command_with_version(binary_name);
    assert!(
      version_output.is_none(),
      "run_command_with_version should return None when feature is off"
    );
  }

  #[test]
  fn test_partition_binaries() {
    // This test requires `which` to work correctly.
    // It's generally fine, but depends on the test environment's PATH.
    // Let's assume 'cargo' (if building with cargo) and a non-existent binary.
    let cargo_exists = which::which("cargo").is_ok();

    let mut bins_to_check = vec![Binary::new(
      Cow::borrowed("hopefully_non_existent_binary_dsfargeg"),
      unknown_version(),
    )];
    if cargo_exists {
      bins_to_check.push(Binary::new(Cow::borrowed("cargo"), unknown_version()));
    }

    let (available, not_available) = partition_binaries(bins_to_check);

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
}
