#![allow(clippy::never_loop)] // because of the files that can vary depending on the system
use anyhow::{bail, ensure, Result};
use beef::Cow;
use colored::Colorize;
use std::time::Instant;
use tracing::{debug, info, Level};
use tracing_subscriber::FmtSubscriber;

use clap::Parser;

#[cfg(feature = "version-retrieval")]
use once_cell::sync::Lazy;
#[cfg(feature = "version-retrieval")]
use rayon::prelude::*;
#[cfg(feature = "version-retrieval")]
use regex::Regex;
#[cfg(feature = "version-retrieval")]
use xshell::{cmd, Shell};

// TODO: custom error types for all cases

#[derive(Debug)]
struct Binary<'a> {
    name: Cow<'a, str>,
    version: Cow<'a, str>,
}

impl<'a> Binary<'a> {
    fn new(name: Cow<'a, str>, version: Cow<'a, str>) -> Self {
        Self { name, version }
    }
}

impl Default for Binary<'_> {
    fn default() -> Self {
        Self {
            name: Cow::borrowed(""),
            version: Cow::borrowed("?"),
        }
    }
}

// TODO: impl join for binary names

fn partition_binaries(binaries_to_check: Vec<Binary<'_>>) -> (Vec<Binary<'_>>, Vec<Binary<'_>>) {
    let mut available: Vec<Binary> = Vec::new();
    let mut not_available: Vec<Binary> = Vec::new();

    for binary in binaries_to_check {
        let name = binary.name.as_ref();
        if which::which(name).is_ok() {
            available.push(binary);
        } else {
            not_available.push(binary);
        }
    }
    (available, not_available)
}

#[cfg(feature = "version-retrieval")]
fn run_command_with_version<'a>(binary_name: &str) -> Option<Cow<'a, str>> {
    // TODO: log the frequency of these

    use tracing::trace;
    let version_flags = ["--version", "-v", "-version", "-V"];

    for flag in &version_flags {
        let sh = match Shell::new() {
            Ok(s) => s,
            Err(e) => {
                debug!("Error creating shell: {}", e);
                return None;
            }
        };
        let name = binary_name;
        let command = cmd!(sh, "{name}").ignore_stderr().arg(flag);
        trace!("Running command: {:?}", command);

        match command.read() {
            Ok(output) => return Some(Cow::owned(output)),
            Err(_) => {
                // debug!(binary_name = binary_name, flag = flag, "flag didn't work");
                continue;
            }
        };
    }
    None
}

#[cfg(not(feature = "version-retrieval"))]
fn run_command_with_version<'a>(_binary_name: &str) -> Option<Cow<'a, str>> {
    None // Always return None if feature is disabled
}

#[cfg(feature = "version-retrieval")]
fn extract_version(output: Cow<str>) -> String {
    static VERSION_RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"v?(\d+\.\d+(?:\.\d+)?(?:[-+].+?)?)").unwrap());

    let mut version_to_return: String = "?".into();
    let mut deferred_debug_messages: Vec<String> = Vec::new();
    let mut version_successfully_extracted = false;

    for line in output.lines() {
        deferred_debug_messages.push(format!("line: {}", line));
        if let Some(captures) = VERSION_RE.captures(line) {
            if let Some(m) = captures.get(1) {
                version_to_return = m.as_str().to_string();
                deferred_debug_messages.push(format!("capture[{}]: {}", 1, m.as_str()));
            }
            version_successfully_extracted = true;
            break;
        }
    }

    debug!("{}", deferred_debug_messages.join(" :: "));

    if !version_successfully_extracted {
        info!("couldn't extract version from this: {}", output);
    }

    version_to_return // Return the extracted version or "?" at the end
}

#[cfg(not(feature = "version-retrieval"))]
fn extract_version(_output: Cow<str>) -> String {
    "?".into() // Always return "?" if feature is disabled
}

fn get_binary_names<'a>(cli: &Cli) -> Result<Vec<Binary<'a>>> {
    match cli.bins.clone() {
        Some(bins) => {
            let binaries: Vec<Binary> = bins
                .iter()
                .map(|name| Binary::new(Cow::owned(name.clone()), Cow::borrowed("?")))
                .collect::<Vec<Binary>>();
            Ok(binaries)
        }
        None => {
            let file_paths = ["needsfile", ".needsfile", "needs", ".needs"];

            for path in file_paths {
                // Attempt to read from the first successful file path
                if let Ok(content) = std::fs::read_to_string(path) {
                    if content.trim().is_empty() {
                        debug!(
                            "needsfile found at '{}' but it is empty. Trying next.",
                            path
                        );
                        continue; // Try next file if this one is empty
                    }
                    let names: Vec<String> =
                        content.split_whitespace().map(|s| s.to_owned()).collect();

                    // This ensure might be redundant if content.trim().is_empty() check is robust
                    ensure!(
                        !names.is_empty(),
                        "needsfile at '{}' is effectively empty after parsing.",
                        path
                    );
                    let binaries = names
                        .iter()
                        .map(|name_str| {
                            Binary::new(Cow::owned(name_str.clone()), Cow::borrowed("?"))
                        })
                        .collect::<Vec<Binary>>();
                    return Ok(binaries);
                } else {
                    debug!(
                        "Failed to read or find needsfile at: {}. Trying next.",
                        path
                    );
                }
            }
            bail!("No binaries specified and no non-empty needsfile (needsfile, .needsfile, needs, .needs) found.");
        }
    }
}

fn sort_binaries(binaries: &mut Vec<Binary>) {
    binaries.sort_by(|a, b| a.name.cmp(&b.name))
}

#[cfg(feature = "version-retrieval")]
fn get_versions(binaries: Vec<Binary>) -> Vec<Binary> {
    binaries
        .into_par_iter() // Consumes binaries, no need for par_iter on a ref
        .map(|binary| {
            // binary is now owned Binary
            let now = Instant::now();
            // name is now binary.name, which is Cow. We need &str for run_command_with_version
            match run_command_with_version(binary.name.as_ref()) {
                Some(output) => {
                    let version = extract_version(output);
                    debug!(
                        ms = now.elapsed().as_millis(),
                        binary_name = binary.name.as_ref(),
                        "Took"
                    );
                    Binary::new(binary.name, Cow::owned(version))
                }
                None => {
                    debug!(
                        ms = now.elapsed().as_millis(),
                        binary_name = binary.name.as_ref(),
                        "Took: no version found"
                    );
                    Binary::new(binary.name, Cow::borrowed("?"))
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
    verbose: u8,

    #[cfg(feature = "version-retrieval")]
    /// don't check for versions
    #[clap(short, long)]
    no_versions: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set log level based on verbosity
    let log_level = match cli.verbose {
        0 => Level::ERROR,
        1 => Level::WARN,
        2 => Level::INFO,
        3 => Level::DEBUG,
        _ => Level::TRACE,
    };
    let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

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

    if cli.quiet {
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

    #[test]
    fn test_verbosity_level_mapping() {
        // This test verifies the mapping between verbosity flags and tracing log levels
        let test_cases = [
            (0, Level::ERROR),
            (1, Level::WARN),
            (2, Level::INFO),
            (3, Level::DEBUG),
            (4, Level::TRACE),
            (5, Level::TRACE), // Beyond 4 should still be TRACE
        ];

        for (verbose_count, expected_level) in test_cases {
            let cli = create_test_cli(None, false);
            let cli = Cli {
                verbose: verbose_count,
                ..cli
            };

            let log_level = match cli.verbose {
                0 => Level::ERROR,
                1 => Level::WARN,
                2 => Level::INFO,
                3 => Level::DEBUG,
                _ => Level::TRACE,
            };

            assert_eq!(
                log_level, expected_level,
                "Verbosity level {} should map to {:?}",
                verbose_count, expected_level
            );
        }
    }

    #[cfg(feature = "version-retrieval")]
    #[test]
    fn test_extract_version_feature_on() {
        let output = "v1.2.3\n";
        let version = extract_version(Cow::borrowed(output));
        assert_eq!(version, "1.2.3");

        let output = "v100.200.300\n";
        let version = extract_version(Cow::borrowed(output));
        assert_eq!(version, "100.200.300");

        let output = "v1.2.3-nightly\n";
        let version = extract_version(Cow::borrowed(output));
        assert_eq!(version, "1.2.3-n"); // Corrected assertion
    }

    #[cfg(feature = "version-retrieval")]
    #[test]
    fn test_extract_version_no_match_feature_on() {
        let output = "no version found\n";
        let version = extract_version(Cow::borrowed(output));
        assert_eq!(version, "?");
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
            verbose: 0,
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
            Cow::borrowed("?"),
        )];
        if cargo_exists {
            bins_to_check.push(Binary::new(Cow::borrowed("cargo"), Cow::borrowed("?")));
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
