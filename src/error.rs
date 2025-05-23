use miette::{Diagnostic, NamedSource, SourceSpan};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum AppError {
  #[error(transparent)]
  Io(#[from] IoError),

  #[error(transparent)]
  Discovery(#[from] DiscoveryError),

  #[error(transparent)]
  Version(#[from] VersionError),

  #[error(transparent)]
  Validation(#[from] ValidationError),

  #[error(transparent)]
  Config(#[from] ConfigError),
}

#[derive(Error, Debug, Diagnostic)]
pub enum IoError {
  #[error("Failed to read file: {path}")]
  #[diagnostic(code(needs::io::read_failed))]
  FileRead {
    path: PathBuf,
    #[source]
    source: std::io::Error,
  },

  #[error("Failed to write file: {path}")]
  #[diagnostic(code(needs::io::write_failed))]
  FileWrite {
    path: PathBuf,
    #[source]
    source: std::io::Error,
  },

  #[error("No valid needsfile found")]
  #[diagnostic(
    code(needs::io::needsfile_missing),
    help("Provide a list of binaries or create a needsfile.")
  )]
  NeedsfileMissing,

  #[error("Needsfile is empty: {path}")]
  #[diagnostic(
    code(needs::io::needsfile_empty),
    help("Add binary names to your needsfile, one per line or space-separated.")
  )]
  NeedsfileEmpty { path: String },
}

#[derive(Error, Debug, Diagnostic)]
pub enum DiscoveryError {
  #[error("No binaries found")]
  #[diagnostic(
    code(needs::discovery::no_binaries),
    help("Please specify at least one binary to check.")
  )]
  NoBinariesSpecified,

  #[error("Failed to check if binary exists: {name}")]
  #[diagnostic(code(needs::discovery::binary_check_failed))]
  BinaryCheck {
    name: String,
    #[source]
    source: std::io::Error,
  },
}

#[derive(Error, Debug, Diagnostic)]
pub enum VersionError {
  #[error("Failed to execute binary: {name}")]
  #[diagnostic(code(needs::version::execution_failed))]
  Execution {
    name: String,
    #[source]
    source: std::io::Error,
  },

  #[error("Failed to parse version from output: {name}")]
  #[diagnostic(
    code(needs::version::parse_failed),
    help("The version output format may not be recognized.")
  )]
  VersionParse { name: String, output: String },

  #[error("Failed to parse semver: {version_string}")]
  #[diagnostic(
    code(needs::version::semver_parse_failed),
    help("The version string is not a valid semantic version.")
  )]
  SemverParse {
    version_string: String,
    #[source]
    source: semver::Error,
  },
}

#[derive(Error, Debug, Diagnostic)]
pub enum ValidationError {
  #[error("Invalid content in '{filename}'")]
  #[diagnostic(code(needs::validation::invalid_content))]
  InvalidContent {
    filename: String,
    token: String,
    #[source_code]
    source_code: NamedSource<String>,
    #[label("This token: '{token}' is problematic")]
    span: SourceSpan,
    #[help]
    advice: Option<String>,
  },
}

#[derive(Error, Debug, Diagnostic)]
pub enum ConfigError {
  #[error("Invalid configuration: {reason}")]
  #[diagnostic(code(needs::config::invalid_config), help("{advice}"))]
  Invalid { reason: String, advice: String },

  #[error("Failed to set up logger")]
  #[diagnostic(code(needs::config::logger_setup_failed))]
  LoggerSetup {
    #[source]
    source: log::SetLoggerError,
  },
}
