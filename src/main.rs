use anyhow::{bail, ensure, Result};
use beef::Cow;
use colored::Colorize;
use std::time::Instant;
use tracing::{debug, info, Level};
use tracing_subscriber::FmtSubscriber;
use xshell::{cmd, Shell};

use clap::Parser;
use rayon::prelude::*;
use regex::Regex;
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

impl<'a> Default for Binary<'a> {
    fn default() -> Self {
        Self {
            name: Cow::borrowed(""),
            version: Cow::borrowed("?"),
        }
    }
}

// TODO: impl join for binary names

fn run_command_with_version<'a>(binary_name: &str) -> Option<Cow<'a, str>> {
    // TODO: log the frequency of these
    let version_flags = ["--version", "-v", "-version", "-V"];

    for flag in &version_flags {
        let sh = Shell::new().unwrap(); // yep, we run these in separated shells
        let name = binary_name;
        let command = cmd!(sh, "{name}").ignore_stderr().arg(flag);
        debug!("Running command: {:?}", command);

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

fn extract_version(output: Cow<str>) -> String {
    let re = Regex::new(r"v?(\d+\.\d+(?:\.\d+)?(?:[-+].+?)?)").unwrap();

    for line in output.lines() {
        if let Some(captures) = re.captures(line) {
            for i in 0..captures.len() {
                if let Some(m) = captures.get(i) {
                    debug!("capture[{}]: {}", i, m.as_str());
                }
            }
            return captures[1].to_string();
        }
    }
    info!("couldn't extract version from this: {}", output);
    "?".into()
}

fn get_binary_names<'a>(cli: &Cli) -> Result<Vec<Binary<'a>>> {
    match cli.bins.clone() {
        Some(bins) => {
            let binaries: Vec<Binary> = bins
                .iter()
                .map(|name| {
                    Binary::new(Cow::owned(name.to_string()), Cow::owned(String::from("?")))
                })
                .collect::<Vec<Binary>>();
            Ok(binaries)
        }
        None => {
            let file_paths = ["needsfile", ".needsfile", "needs", ".needs"];

            for path in file_paths {
                let names = match std::fs::read_to_string(path) {
                    Ok(content) => content
                        .split_whitespace()
                        .map(|s| s.to_owned())
                        .collect::<Vec<String>>(),
                    Err(..) => bail!("Failed to read file: {}", path),
                };

                ensure!(!names.is_empty(), "needsfile empty.");
                let binaries = names
                    .iter()
                    .map(|name| {
                        Binary::new(Cow::owned(name.to_string()), Cow::owned(String::from("?")))
                    })
                    .collect::<Vec<Binary>>();
                return Ok(binaries);
            }

            bail!("No binaries specified and no needsfile found.");
        }
    }
}

fn sort_binaries(binaries: &mut Vec<Binary>) {
    binaries.sort_by(|a, b| a.name.cmp(&b.name))
}

fn get_versions(binaries: Vec<Binary>) -> Vec<Binary> {
    let bins_with_versions = binaries
        .par_iter()
        .map(|binary| {
            let now = Instant::now();
            let name = &binary.name;

            match run_command_with_version(name) {
                Some(output) => {
                    let version = extract_version(output);
                    // debug!(ms = now.elapsed().as_millis(), binary_name = name, "Took");
                    Binary::new(Cow::owned(name.to_string()), Cow::owned(version))
                }
                None => {
                    // debug!(binary_name = name, "No version found for binary");
                    debug!(ms = now.elapsed().as_millis(), "Took");
                    Binary::new(Cow::owned(name.to_string()), Cow::owned("?".into()))
                }
            }
        })
        .collect();
    bins_with_versions
}

fn print_center_aligned(binaries: Vec<Binary>, max_len: usize, no_versions: bool) -> Result<()> {
    for bin in &binaries {
        let padding_needed = max_len - bin.name.len();
        let padding = " ".repeat(padding_needed);
        println!(
            "{}{} {}",
            padding,
            bin.name.bright_green(),
            if no_versions { "found" } else { &bin.version }
        );
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

    /// don't check for versions
    #[clap(short, long)]
    no_versions: bool,
}

fn main() -> Result<()> {
    let log_level = if cfg!(debug_assertions) {
        Level::DEBUG
    } else {
        Level::ERROR
    };
    let subscriber = FmtSubscriber::builder().with_max_level(log_level).finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let cli = Cli::parse();

    let binaries = get_binary_names(&cli)?;
    ensure!(!binaries.is_empty(), "binary sources are empty");
    let max_name_len = binaries.iter().map(|bin| bin.name.len()).max().unwrap_or(0);

    let mut available: Vec<Binary> = Vec::new();
    let mut not_available: Vec<Binary> = Vec::new();

    for binary in binaries {
        let name = binary.name.as_ref();
        if which::which(name).is_ok() {
            available.push(binary);
        } else {
            not_available.push(binary);
        }
    }

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

    if cli.no_versions {
        print_center_aligned(available, max_name_len, true)?;
    } else {
        let mut bins_with_versions = get_versions(available);
        sort_binaries(&mut bins_with_versions);
        print_center_aligned(bins_with_versions, max_name_len, false)?;
    }

    if needs_separator {
        let padding = " ".repeat(max_name_len - 1);
        println!("{}---", padding);
    }

    if !not_available.is_empty() {
        for binary in not_available {
            println!("{} not found", binary.name.bright_red());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_version() {
        let output = "v1.2.3\n";
        let version = extract_version(Cow::borrowed(output));
        assert_eq!(version, "1.2.3");

        let output = "v100.200.300\n";
        let version = extract_version(Cow::borrowed(output));
        assert_eq!(version, "100.200.300");

        let output = "v1.2.3-nightly\n";
        let version = extract_version(Cow::borrowed(output));
        assert_eq!(version, "1.2.3-n");
    }

    #[test]
    fn test_extract_version_no_match() {
        let output = "no version found\n";
        let version = extract_version(Cow::borrowed(output));
        assert_eq!(version, "?");
    }

    #[test]
    fn test_get_binary_names() {
        let args = vec!["PLACEHOLDER", "bat", "btm", "needs", "apfelkuchen"];
        let cli = Cli::parse_from(args);
        let binaries = get_binary_names(&cli).unwrap();
        assert_eq!(binaries.len(), 4);
        assert_eq!(binaries[0].name, Cow::borrowed("bat"));
        assert_eq!(binaries[1].name, Cow::borrowed("btm"));
        assert_eq!(binaries[2].name, Cow::borrowed("needs"));
        assert_eq!(binaries[3].name, Cow::borrowed("apfelkuchen"));
    }

    #[test]
    fn test_run_command_with_version() {
        let binary_name = "bat";
        let version = run_command_with_version(binary_name);
        println!("Version: {:?}", version);
        assert!(version.is_some());
        assert!(version.unwrap().contains("bat"));
    }
}
