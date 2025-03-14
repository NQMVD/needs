use std::collections::HashMap;
use std::io::Write;
use std::process::Command;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use which::which;

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let binary_names = vec!["hx", "gum", "rg", "btm", "deno", "pueue", "lua", "luajit"];

    let (available, not_available): (Vec<&str>, Vec<&str>) = binary_names
        .par_iter()
        .partition(|binary_name| which(binary_name).is_ok());

    let bins_with_versions = get_versions(available);

    let mut version_map = HashMap::new();
    for bin in bins_with_versions {
        version_map.insert(bin.binary_name, bin.version);
    }

    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let mut color_spec = ColorSpec::new();

    color_spec.set_fg(Some(Color::Green)).set_bold(true);
    for (bin, version) in version_map {
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

fn get_versions(available: Vec<&str>) -> Vec<VersionInfo> {
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
                    "?".into()
                } else {
                    version
                },
                binary_name: binary_name.to_string(),
            }
        })
        .collect();
    bins_with_versions
}
