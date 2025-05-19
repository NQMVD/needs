use crate::error::ConfigError;
use clap::Parser;
use miette::Result;

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
pub struct Cli {
  /// List of binaries to check
  pub bins: Option<Vec<String>>,

  /// stay quiet, exit with 0 or 1
  #[clap(short, long)]
  pub quiet: bool,

  /// Verbosity level (can be repeated, e.g. -vvv)
  #[clap(short, long, action = clap::ArgAction::Count)]
  pub verbosity: u8,

  #[cfg(feature = "version-retrieval")]
  /// don't check for versions
  #[clap(short, long)]
  pub no_versions: bool,

  #[cfg(feature = "version-retrieval")]
  /// show the full version string
  #[clap(short, long)]
  pub full_versions: bool,
}
