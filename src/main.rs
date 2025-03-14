use std::collections::HashMap;
use std::io::Write;
use std::process::Command;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use which::which;

use clap::Parser;
use rayon::prelude::*;

struct VersionInfo {
    version: String,
    binary_name: String,
}

fn try_version_command(bin_name: &str) -> Option<std::process::Output> {
    // Try different version flags
    let version_flags = ["--version", "-version", "-V", "version", "--ver", "-v"];

    for flag in &version_flags {
        match Command::new(bin_name).arg(flag).output() {
            Ok(output) if output.status.success() => {
                return Some(output);
            }
            _ => continue,
        }
    }

    // Try with no flags as a last resort
    match Command::new(bin_name).output() {
        Ok(output) if output.status.success() => Some(output),
        _ => None,
    }
}

fn extract_version(output: std::process::Output) -> String {
    let output_str = String::from_utf8_lossy(&output.stdout);
    let re = regex::Regex::new(r"v?(\d+\.\d+(?:\.\d+)?(?:[-+].+?)?)").unwrap();

    let mut version = String::new();
    for line in output_str.lines() {
        if let Some(cap) = re.captures(line) {
            version = cap[1].to_string();
            break;
        }
    }
    version
}

fn get_binaries() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(bins) = cli.bins {
        Ok(bins)
    } else {
        let file_paths = ["needsfile", ".needsfile", "needs", ".needs"];
        for path in file_paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                let bins: Vec<String> = content.split_whitespace().map(|s| s.to_owned()).collect();
                if !bins.is_empty() {
                    return Ok(bins);
                }
            }
        }

        Err("No binaries specified and no needsfile found.".into())
    }
}

fn get_versions(available: Vec<&str>) -> HashMap<String, String> {
    let bins_with_versions: Vec<VersionInfo> = available
        .par_iter()
        .map(|binary_name| {
            let output = match try_version_command(binary_name) {
                Some(out) => out,
                None => {
                    println!("{}: Failed to get version information", binary_name);
                    return VersionInfo {
                        version: "?".into(),
                        binary_name: binary_name.to_string(),
                    };
                }
            };

            let version = extract_version(output);

            VersionInfo {
                version: if version.is_empty() {
                    "unknown".into()
                } else {
                    version
                },
                binary_name: binary_name.to_string(),
            }
        })
        .collect();

    let mut version_map = HashMap::new();
    for bin in bins_with_versions {
        version_map.insert(bin.binary_name, bin.version);
    }
    version_map
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

    /// only return with 0 or 1 exit code
    #[clap(short, long)]
    quiet: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let quiet = Cli::parse().quiet;

    let binaries = match get_binaries() {
        Ok(it) => it,
        Err(err) => {
            eprintln!("Run `needs --help` for more information.");
            return Err(err);
        }
    };
    let binary_names: Vec<&str> = binaries.iter().map(|s| s.as_str()).collect();

    let (available, not_available): (Vec<&str>, Vec<&str>) = binary_names
        .par_iter()
        .partition(|binary_name| which(binary_name).is_ok());

    if quiet {
        if !not_available.is_empty() {
            std::process::exit(1);
        }
        std::process::exit(0);
    }

    let bins_with_versions = get_versions(available);

    let mut sorted_bins_with_versions: Vec<(String, String)> = Vec::new();
    for binary_name in &binary_names {
        if let Some(version) = bins_with_versions.get(*binary_name) {
            sorted_bins_with_versions.push((binary_name.to_string(), version.clone()));
        }
    }

    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let mut color_spec = ColorSpec::new();

    color_spec.set_fg(Some(Color::Green)).set_bold(true);
    for (bin, version) in sorted_bins_with_versions {
        stdout.set_color(&color_spec)?;
        write!(&mut stdout, "{}", bin)?;
        stdout.reset()?;
        writeln!(&mut stdout, " {}", version)?;
    }

    color_spec.set_fg(Some(Color::Red)).set_bold(true);
    for bin in not_available {
        stdout.set_color(&color_spec)?;
        write!(&mut stdout, "{}", bin)?;
        stdout.reset()?;
        writeln!(&mut stdout, " not found")?;
    }

    Ok(())
}
