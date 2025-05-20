use crate::cli::Cli;
use crate::error::IoError;

use crate::binary::Binary;
use beef::Cow;
use log::{debug, error, warn};
use miette::Result;
use std::path::PathBuf;

pub fn get_binary_names<'a>(cli: &Cli) -> Result<Vec<Binary<'a>>> {
  let mut bail_cause =
    "No valid needsfile found.\nPlease provide a list of binaries or create a needsfile.";
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
        match std::fs::read_to_string(path) {
          Ok(content) => {
            if content.trim().is_empty() {
              warn!(path = path; "needsfile found but it is empty, trying next.");
              continue; // Try next file if this one is empty
            }
            let names: Vec<String> = content.split_whitespace().map(|s| s.to_owned()).collect();
            if names.is_empty() {
              warn!(path = path; "needsfile found but it is empty, trying next.");
              continue; // Try next file if this one is empty
            }

            debug!(path:debug = path, binaries:debug = &names; "found needsfile");
            bins.extend(names);
            break;
          }
          Err(err) => {
            debug!(path = path, error:display = err; "Failed to read or find needsfile, trying next.");
          }
        }
      }
      if bins.is_empty() {
        warn!("No valid needsfile found");
        return Err(IoError::NeedsfileMissing.into());
      } else {
        bins
      }
    }
  };

  let binaries: Vec<Binary> = bins
    .iter()
    .filter(|name| !name.is_empty())
    .map(|name| Binary::new(Cow::owned(name.clone())))
    .collect::<Vec<Binary>>();

  // LEAVE this here because sometimes collecting the binaries fails
  if binaries.is_empty() {
    error!(binaries:debug = &binaries; "binary collection failed");
    return Err(
      IoError::NeedsfileEmpty {
        path: "command line arguments".into(),
      }
      .into(),
    );
  }
  Ok(binaries)
}
