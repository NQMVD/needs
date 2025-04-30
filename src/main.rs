use std::io::Write;
use std::time::Instant;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use tracing::{debug, info, Level};
use tracing_subscriber::FmtSubscriber;
use anyhow::{bail, ensure, Result};
use xshell::{cmd, Shell};

use clap::Parser;
use rayon::prelude::*;
use regex::Regex;
// TODO: custom error types for all cases

#[derive(Clone, Debug)]
struct Binary {
    name: String,
    version: String,
}

// TODO: impl join for binary names

fn run_command_with_version(binary_name: &str) -> Option<String> {
    // TODO: log the frequency of these
    let version_flags = ["--version", "-v", "-version", "-V"];

    for flag in &version_flags {
        let sh = Shell::new().unwrap(); // yep, we run these in separated shells
        let command = cmd!(sh, "{binary_name}")
            .ignore_stderr()
            .arg(flag);
        debug!("Running command: {:?}", command);

        match command.read() {
            Ok(output) => return Some(output),
            Err(_) => {
                debug!(binary_name = binary_name, flag = flag, "flag didn't work");
                continue
            },
        };
    }
    None
}

fn extract_version(output: String) -> String {
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

fn get_binary_names() -> Result<Vec<Binary>> {
    let cli = Cli::parse();

    match cli.bins {
        Some(bins) => {
            let binaries = bins.iter().map(
                |name| {
                Binary {
                    name: name.clone(),
                    version: "?".into()
                }
            });
            Ok(binaries.collect::<Vec<Binary>>())
        }
        None => {
            let file_paths = ["needsfile", ".needsfile", "needs", ".needs"];

            for path in file_paths {
                let names = match std::fs::read_to_string(path) {
                    Ok(content) => {
                        content
                            .split_whitespace()
                            .map(|s| s.to_owned())
                            .collect::<Vec<String>>()
                    }
                    Err(..) => bail!("Failed to read file: {}", path),
                };

                ensure!(!names.is_empty(), "needsfile empty.");
                let binaries = names.iter().map(|name| {
                    Binary {
                        name: name.clone(), version: "?".into()
                    }
                });
                return Ok(binaries.collect::<Vec<Binary>>());
            }

            bail!("No binaries specified and no needsfile found.");
        }
    }
}

fn sort_binaries(binaries: &mut Vec<Binary>) {
    binaries
        .sort_by(
            |a, b| a.name.cmp(&b.name)
        )
}

fn get_versions(binaries: Vec<Binary>) -> Vec<Binary> {
    let bins_with_versions = binaries
        .par_iter()
        .map(|binary| {
            let now = Instant::now();
            let name = &*binary.name; // IS THIS IT???

            match run_command_with_version(name) {
                Some(output) => {
                    let version = extract_version(output);
                    debug!(ms = now.elapsed().as_millis(), binary_name = name, "Took");
                    Binary {
                        name: binary.name.clone(),
                        version,
                    }
                },
                None => {
                    debug!(binary_name = name, "No version found for binary");
                    debug!(ms = now.elapsed().as_millis() , "Took");
                    Binary {
                        name: binary.name.clone(),
                        version: "?".into()
                    }
                }
            }
        })
        .collect();
    bins_with_versions
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
    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");


    let cli = Cli::parse();

    let binaries = get_binary_names()?;
    ensure!(!binaries.is_empty(), "binary sources are empty");

    let mut available: Vec<Binary> = Vec::new();
    let mut not_available: Vec<Binary> = Vec::new();

    binaries.iter()
        .for_each(|binary| {
            if which::which(binary.name.clone().as_str()).is_ok() {
                available.push(binary.clone());
            } else {
                not_available.push(binary.clone());
            }
        });

    sort_binaries(&mut available);
    sort_binaries(&mut not_available);

    if cli.quiet {
        if !not_available.is_empty() {
            // info!("quiet exit, missing: {}", not_available.join(", "));
            std::process::exit(1);
        }
        info!("quiet exit");
        std::process::exit(0);
    }

    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let mut color_spec = ColorSpec::new();

    if cli.no_versions {
        color_spec.set_fg(Some(Color::Green)).set_bold(true);
        for bin in available {
            stdout.set_color(&color_spec)?;
            write!(&mut stdout, "{}", bin.name)?;
            stdout.reset()?;
            writeln!(&mut stdout, " found")?;
        }
    } else {
        let mut bins_with_versions = get_versions(available);
        sort_binaries(&mut bins_with_versions);

        color_spec.set_fg(Some(Color::Green)).set_bold(true);
        for binary in bins_with_versions {
            stdout.set_color(&color_spec)?;
            write!(&mut stdout, "{}", binary.name)?;
            stdout.reset()?;
            writeln!(&mut stdout, " {}", binary.version)?;
        }
    }

    color_spec.set_fg(Some(Color::Red)).set_bold(true);
    for binary in not_available {
        stdout.set_color(&color_spec)?;
        write!(&mut stdout, "{}", binary.name)?;
        stdout.reset()?;
        writeln!(&mut stdout, " not found")?;
    }

    Ok(())
}
