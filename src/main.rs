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
use semver::{BuildMetadata, Prerelease, Version};
use std::fmt;
use std::fmt::Arguments;
use std::{collections::BTreeMap, fmt::Display, time::Instant};
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
  // TODO: use a custom version type
  version: Option<Version>,
}

impl<'a> Binary<'a> {
  fn new(name: Cow<'a, str>) -> Self {
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

fn format_version(value: &Version, full_versions: bool) -> impl fmt::Display + '_ {
  struct Wrapper<'a>(&'a Version, bool);

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

// list known binaries that dont have a version
#[cfg(feature = "version-retrieval")]
fn known_binaries() -> Vec<Cow<'static, str>> {
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

fn unknown_version() -> Version {
  Version {
    major: 0,
    minor: 0,
    patch: 0,
    pre: Prerelease::EMPTY,
    build: BuildMetadata::new("unknown").unwrap(),
  }
}

#[cfg(feature = "version-retrieval")]
static _SEMVER_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"(?:^|\s)((?:[<>=~^]|>=|<=)?)(?:v)?((?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:\.(?:0|[1-9]\d*))?(?:-(?:[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\+(?:[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?)").unwrap()
});

#[cfg(feature = "version-retrieval")]
static VER_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"(\d+\.\d+(?:\.\d+)?(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?)").unwrap()
});

#[cfg(feature = "version-retrieval")]
fn clean_version_string(version_str: &str) -> String {
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

fn partition_binaries(binaries_to_check: Vec<Binary<'_>>) -> (Vec<Binary<'_>>, Vec<Binary<'_>>) {
  let mut available: Vec<Binary> = Vec::new();
  let mut not_available: Vec<Binary> = Vec::new();

  for binary in binaries_to_check {
    let name = binary.name.as_ref();
    if which::which(name).is_ok() {
      info!(SCOPE = "which", bin = name; "found");
      available.push(binary);
    } else {
      info!(SCOPE = "which", bin = name; "not found");
      not_available.push(binary);
    }
  }
  (available, not_available)
}

#[cfg(feature = "version-retrieval")]
fn execute_binary<'a>(binary_name: &str) -> Option<Cow<'a, str>> {
  // TODO: log the frequency of these

  let version_flags = ["--version", "-v", "-version", "-V"];

  for flag in &version_flags {
    let sh = match Shell::new() {
      Ok(s) => s,
      Err(e) => {
        error!(error:display = e; "Error creating shell");
        return None;
      }
    };
    let name = binary_name;
    let command = cmd!(sh, "{name}").ignore_stderr().arg(flag);
    trace!(command:display = command; "Running command");

    match command.read() {
      Ok(output) => return Some(Cow::owned(output)),
      Err(_) => {
        debug!(SCOPE = binary_name, flag = flag; "flag didn't work, trying next...");
        continue;
      }
    };
  }
  info!(scope = binary_name; "no version flag found, see --help or check builtins");
  None
}

#[cfg(feature = "version-retrieval")]
fn extract_version<'a>(output: Cow<'a, str>, binary_name: Cow<'a, str>) -> Option<Cow<'a, str>> {
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

      return Some(Cow::owned(version_string.to_string()));
    }
  }

  warn!(SCOPE = binary_name.as_ref(), output = output.as_ref(); "No valid version found in the output");
  None
}

#[cfg(feature = "version-retrieval")]
fn get_version(binary_name: Cow<str>) -> Option<Version> {
  let now = Instant::now();
  let output = execute_binary(binary_name.as_ref());
  trace!(
      SCOPE = binary_name.as_ref(),
      ms = now.elapsed().as_millis();
      "calling binary took"
  );
  match output {
    Some(output) => {
      trace!(
        SCOPE = binary_name.as_ref(), output = output.as_ref(); "command output");
      let version_string = match extract_version(output.clone(), binary_name.clone()) {
        Some(v) => v,
        None => {
          return None;
        }
      };
      let version = match Version::parse(version_string.as_ref()) {
        Ok(v) => {
          debug!(SCOPE = binary_name.as_ref(), version:debug = v; "version parsed");
          v
        }
        Err(e) => {
          warn!(SCOPE = binary_name.as_ref(), error:display = e; "error parsing version");
          return None;
        }
      };
      Some(version)
    }
    None => None,
  }
}

#[cfg(feature = "version-retrieval")]
fn get_versions_for_bins(binaries: Vec<Binary>) -> Vec<Binary> {
  binaries
    .into_par_iter()
    .map(|binary| {
      // filter out known binaries that don't have a version
      if known_binaries().contains(&binary.name) {
        return Binary {
          name: binary.name,
          version: None
        };
      }
      
      let version = get_version(binary.name.clone());
      Binary {
        name: binary.name,
        version,
      }
    })
    .collect()
}

fn get_binary_names<'a>(cli: &Cli) -> Result<Vec<Binary<'a>>> {
  let mut bail_cause =
    "No valid needsfile found. Please provide a list of binaries or create a needsfile.";
  let bins = match cli.bins.clone() {
    Some(bins) => {
      debug!(bins:debug = bins; "got bins from args");
      bins
    }
    None => {
      debug!("no bins from args, trying to read from needsfiles");
      let file_paths = ["needsfile", ".needsfile", "needs", ".needs"];
      let mut bins = Vec::new();

      for path in file_paths {
        // Attempt to read from the first successful file path
        let read_to_string = std::fs::read_to_string(path);
        match read_to_string {
          Ok(content) => {
            if content.trim().is_empty() {
              bail_cause = "needsfile found but it is empty";
              warn!(path = path; "needsfile found but it is empty, trying next.");
              continue; // Try next file if this one is empty
            }
            let names: Vec<String> = content.split_whitespace().map(|s| s.to_owned()).collect();
            if names.is_empty() {
              bail_cause = "needsfile found but it is empty";
              warn!(path = path; "needsfile found but it is empty, trying next.");
              continue; // Try next file if this one is empty
            }

            debug!(path:debug = path, binaries:debug = &names; "found needsfile");
            bins.extend(names);
            break;
          }
          Err(_) => {
            debug!(path = path; "Failed to read or find needsfile, trying next.");
          }
        }
      }
      if bins.is_empty() {
        warn!("No valid needsfile found");
        bail!(bail_cause);
      } else {
        bins
      }
    }
  };

  let binaries: Vec<Binary> = bins
    .iter()
    // filter out empty strings
    .filter(|name| !name.is_empty())
    .map(|name| Binary::new(Cow::owned(name.clone())))
    .collect::<Vec<Binary>>();
  
  // LEAVE this here because sometimes collecting the binaries fails
  if binaries.is_empty() {
    error!(binaries:debug = &binaries; "binary collection failed");
  }
  Ok(binaries)
}

fn sort_binaries(binaries: &mut Vec<Binary>) {
  binaries.sort_by(|a, b| a.name.cmp(&b.name))
}

fn print_center_aligned(
  binaries: Vec<Binary>,
  max_len: usize,
  always_found: bool,
  full_versions: bool,
) -> Result<()> {
  for bin in &binaries {
    let padding_needed = max_len.saturating_sub(bin.name.len());
    let padding = " ".repeat(padding_needed);
    let version_display = if always_found {
      "found".to_string()
    } else {
      match bin.version {
        Some(ref version) => format!("{}", format_version(version, full_versions)),
        None => "?".to_string(),
      }
    };
    println!("{}{} {}", padding, bin.name.green(), version_display);
  }
  Ok(())
}

struct Collect<'kvs>(BTreeMap<Cow<'kvs, str>, Cow<'kvs, str>>);

impl<'kvs> VisitSource<'kvs> for Collect<'kvs> {
  fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), kv::Error> {
    self
      .0
      .insert(key.to_string().into(), value.to_string().into());
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

        // check if theres a key named SCOPE, if so pop it out and print infront of message
        let scope = single
          .iter()
          .find(|(k, _)| *k == "SCOPE")
          .map(|(k, v)| (k.to_string(), v.to_string()));

        let formatted_pairs = single
          .iter()
          .filter(|(k, _)| *k != "SCOPE")
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

        // let mut final_format_string = "{time} {lvl_colored} ".to_owned();

        // if let Some((_, v)) = scope {
        //   final_format_string.push_str(&format!("({}) ", v.bold()));
        // }

        // final_format_string.push_str("{message} ");

        // if !single.is_empty() {
        //   final_format_string.push_str("{formatted_pairs}", );
        // }

        // if !multiline.is_empty() {
        //   final_format_string.push_str("\n  {formatted_multiline_pairs}");
        // }

        out.finish(format_args!(
          "{time} {lvl_colored} {}{message}{}{}",
          if let Some((_, v)) = scope {
            format!("[{}] ", v.bold())
          } else {
            "".to_string()
          },
          if !single.is_empty() {
            format!(" {}", formatted_pairs)
          } else {
            "".to_string()
          },
          if !multiline.is_empty() {
            format!("\n  {}", formatted_multiline_pairs)
          } else {
            "".to_string()
          }
        ))
      } else {
        out.finish(format_args!("{time} {lvl_colored} {message}"))
      }
    })
    .level(log_level)
    .chain(std::io::stdout())
    .apply()?;

  //   let tracing_level = match verbosity {
  //   0 => tracing::Level::ERROR,
  //   1 => tracing::Level::WARN,
  //   2 => tracing::Level::INFO,
  //   3 => tracing::Level::DEBUG,
  //   _ => tracing::Level::TRACE,
  // };
  // let subscriber = FmtSubscriber::builder()
  //   .with_max_level(tracing_level)
  //   .finish();
  // tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

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

  #[cfg(feature = "version-retrieval")]
  /// show the full version string TODO: implement
  #[clap(short, long)]
  full_versions: bool,
}

fn main() -> Result<()> {
  let cli = Cli::parse();

  setup_logger(cli.verbosity).expect("Failed to set up logger");

  debug!("Starting needs with verbosity level {}", cli.verbosity);
  debug!("passed bins: {:?}", cli.bins);
  debug!("quiet: {:?}", cli.quiet);
  // debug!("atty stdout: {}", is(Stream::Stdout));
  // debug!("atty stderr: {}", is(Stream::Stderr));
  // debug!("atty stdin: {}", is(Stream::Stdin));

  let binaries_from_source = match get_binary_names(&cli) {
    Ok(bins) => {
      debug!(binaries:debug = &bins; "got binaries from source");
      bins
    }
    Err(err) => {
      error!(error:display = err; "Error getting binaries");
      std::process::exit(1);
    }
  };
  if binaries_from_source.is_empty() {
    error!("No binaries found, binary sources are empty");
    std::process::exit(1);
  }

  // Calculate max_name_len from all initial binaries for consistent padding
  let global_max_name_len = binaries_from_source
    .iter()
    .map(|bin| bin.name.len())
    .max()
    .unwrap_or(0);

  let (mut available, mut not_available) = partition_binaries(binaries_from_source);

  let stay_quiet = cli.quiet;

  if stay_quiet {
    if !not_available.is_empty() {
      info!(not_available:debug = not_available; "quiet exit, not found:");
      std::process::exit(1);
    }
    info!("quiet exit, all found");
    std::process::exit(0);
  }

  sort_binaries(&mut available);
  sort_binaries(&mut not_available);

  let needs_separator = !available.is_empty() && !not_available.is_empty();

  if !available.is_empty() {
    #[cfg(feature = "version-retrieval")]
    {
      if !cli.no_versions {
        let mut bins_with_versions = get_versions_for_bins(available);
        sort_binaries(&mut bins_with_versions);
        print_center_aligned(
          bins_with_versions,
          global_max_name_len,
          false,
          cli.full_versions,
        )?;
      }
    }
    #[cfg(not(feature = "version-retrieval"))]
    {
      print_center_aligned(available, global_max_name_len, true)?;
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
      println!("{}{} not found", padding, binary.name.red());
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[cfg(feature = "version-retrieval")]
  #[test]
  fn test_version_regex() {
    // separate list just for operator test
    let _operator_test_strings = [
      ("v1.0.0"),
      ("=1.0.0"),
      ("=v1.0.0"),
      (">1.0.0"),
      (">v1.0.0"),
      ("<1.0.0"),
      (">=1.0.0"),
      ("<=1.0.0"),
      ("~1.0.0"),
      ("^1.0.0"),
      ("1.0.0-alpha"),
      ("v1.0.0-alpha"),
      ("1.0.0-alpha.1"),
      ("1.0.0+build.1"),
      ("1.0.0-alpha+beta"),
      (">1.0.0-alpha+beta"),
    ];

    let test_strings = [
      ("1.0.0", "1.0.0"),
      // eza
      ("v1.0.0", "1.0.0"),
      // luajit
      ("2.1.1713773202", "2.1.1713773202"),
      // viddy
      ("1.3.0-VERGEN_IDEMPOTENT_OUTPUT", "1.3.0-VERGEN"),
      // helix
      ("25.01.1", "25.1.1"),
      // love2d
      ("11.5", "11.5.0"),
    ];

    for (output, expected) in test_strings {
      let captures = VER_REGEX.captures(output);
      assert!(captures.is_some(), "Failed to match: {}", output);
      if let Some(captures) = captures {
        let version_str = &captures[1];

        let version_string_cleaned = clean_version_string(version_str);
        println!(
          "Version string cleaned: {} -> {}",
          version_str, version_string_cleaned
        );

        let version = Version::parse(&version_string_cleaned);
        let expected = Version::parse(expected).unwrap();
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
  fn test_extract_version_feature_on() {
    let output = "1.2.3\n";
    let version = extract_version(Cow::borrowed(output), Cow::borrowed("test_binary"));
    assert_eq!(version.as_ref(), Some(&Cow::borrowed("1.2.3")));
    let output = "100.200.300\n";
    let version = extract_version(Cow::borrowed(output), Cow::borrowed("test_binary"));
    assert_eq!(version.as_ref(), Some(&Cow::borrowed("100.200.300")));

    let output = "1.2.3-nightly\n";
    let version = extract_version(Cow::borrowed(output), Cow::borrowed("test_binary"));
    assert_eq!(version.as_ref(), Some(&Cow::borrowed("1.2.3-nightly")));
  }

  #[cfg(feature = "version-retrieval")]
  #[test]
  fn test_extract_version_no_match_feature_on() {
    let output = "no version found\n";
    let version = extract_version(Cow::borrowed(output), Cow::borrowed("test_binary"));
    assert_eq!(version, None);
  }

  #[cfg(feature = "version-retrieval")]
  #[test]
  fn test_ls() {}

  #[cfg(feature = "version-retrieval")]
  fn create_test_cli(bins: Option<Vec<String>>, no_versions_flag: bool) -> Cli {
    Cli {
      bins,
      quiet: false,
      verbosity: 0,
      no_versions: no_versions_flag,
      full_versions: false,
    }
  }

  #[cfg(not(feature = "version-retrieval"))]
  fn create_test_cli(bins: Option<Vec<String>>, _no_versions_flag: bool) -> Cli {
    Cli {
      bins,
      quiet: false,
      verbosity: 0,
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
    let version_output = execute_binary(binary_name);
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

  #[test]
  fn test_partition_binaries() {
    // This test requires `which` to work correctly.
    // It's generally fine, but depends on the test environment's PATH.
    // Let's assume 'cargo' (if building with cargo) and a non-existent binary.
    let cargo_exists = which::which("cargo").is_ok();

    let mut bins_to_check = vec![Binary::new(Cow::borrowed(
      "hopefully_non_existent_binary_dsfargeg",
    ))];
    if cargo_exists {
      bins_to_check.push(Binary::new(Cow::borrowed("cargo")));
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
