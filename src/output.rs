use crate::binary::Binary;
use crate::error::ValidationError;
use crate::versions::format_version;
use colored::Colorize;
use miette::Result;

#[cfg(feature = "version-retrieval")]
pub fn print_center_aligned(
  binaries: Vec<Binary>,
  max_len: usize,
  always_found: bool,
  full_versions: bool,
) -> Result<()> {
  for bin in &binaries {
    let padding_needed = max_len.saturating_sub(bin.name.len());
    let padding = " ".repeat(padding_needed);
    let version_display = if always_found {
      if let Some(ref pm) = bin.package_manager {
        format!("found {}", format!("via {}", pm).dimmed())
      } else {
        "found".to_string()
      }
    } else {
      match bin.version {
        Some(ref version) => {
          let version_str = format!("{}", format_version(version, full_versions));
          if let Some(ref pm) = bin.package_manager {
            format!("{} {}", version_str, format!("via {}", pm).dimmed())
          } else {
            version_str
          }
        }
        None => {
          if let Some(ref pm) = bin.package_manager {
            format!("? {}", format!("via {}", pm).dimmed())
          } else {
            "?".to_string()
          }
        }
      }
    };
    println!("{}{} {}", padding, bin.name.green(), version_display);
  }
  Ok(())
}

#[cfg(not(feature = "version-retrieval"))]
pub fn print_center_aligned(binaries: Vec<Binary>, max_len: usize) -> Result<()> {
  for bin in &binaries {
    let padding_needed = max_len.saturating_sub(bin.name.len());
    let padding = " ".repeat(padding_needed);
    let display_text = if let Some(ref pm) = bin.package_manager {
      format!("found {}", format!("via {}", pm).dimmed())
    } else {
      "found".to_string()
    };
    println!("{}{} {}", padding, bin.name.green(), display_text);
  }
  Ok(())
}
