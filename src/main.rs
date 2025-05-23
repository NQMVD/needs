#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(dead_code)]

mod binary;
mod cli;
mod discovery;
mod error;
mod io;
mod logging;
mod output;
mod versions;

use clap::Parser;
use colored::Colorize;
use log::{debug, error, info};
use miette::{Report, Result};

use crate::binary::{Binary, sort_binaries};
use crate::error::{AppError, DiscoveryError};

fn main() -> Result<()> {
  miette::set_panic_hook();
  let cli = cli::Cli::parse();
  logging::setup_logger(cli.verbosity)?;

  debug!("Starting needs with verbosity level {}", cli.verbosity);
  debug!("passed bins: {:?}", cli.bins);
  debug!("quiet: {:?}", cli.quiet);
  #[cfg(feature = "version-retrieval")]
  {
    debug!("Version retrieval enabled.");
    debug!("No versions flag: {:?}", cli.no_versions);
    debug!("Full versions flag: {:?}", cli.full_versions);
  }
  #[cfg(not(feature = "version-retrieval"))]
  {
    debug!("Version retrieval NOT enabled.");
  }
  // debug!("atty stdout: {}", is(Stream::Stdout));
  // debug!("atty stderr: {}", is(Stream::Stderr));
  // debug!("atty stdin: {}", is(Stream::Stdin));

  // TODO: split this up
  let binaries_from_source: Vec<Binary<'_>> = io::get_binary_names(&cli)?;
  if binaries_from_source.is_empty() {
    error!("No binaries found, binary sources are empty");
    return Err(DiscoveryError::NoBinariesSpecified.into());
  }

  // Calculate max_name_len from all initial binaries for consistent padding
  let global_max_name_len: usize = binaries_from_source
    .iter()
    .map(|bin| bin.name.len())
    .max()
    .unwrap_or(0);

  let (mut available, mut not_available): (Vec<Binary<'_>>, Vec<Binary<'_>>) =
    discovery::partition_binaries(binaries_from_source)?;

  let stay_quiet = cli.quiet;

  if stay_quiet {
    if !not_available.is_empty() {
      info!(not_available:debug = not_available; "quiet exit, not found:");
      std::process::exit(1);
    }
    info!("quiet exit, all found");
    std::process::exit(0);
  }

  available.sort();
  not_available.sort();

  let needs_separator = !available.is_empty() && !not_available.is_empty();

  if !available.is_empty() {
    #[cfg(feature = "version-retrieval")]
    {
      let retrieve_versions = !cli.no_versions;
      let processed_available = if retrieve_versions {
        let mut bins_with_versions = versions::get_versions_for_bins(available);
        sort_binaries(&mut bins_with_versions); // Re-sort after potential version changes
        bins_with_versions
      } else {
        available // Use as-is if not retrieving versions
      };

      output::print_center_aligned(
        processed_available,
        global_max_name_len,
        false,
        cli.full_versions,
      )?;
    }
    #[cfg(not(feature = "version-retrieval"))]
    {
      output::print_center_aligned(available, global_max_name_len)?;
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
