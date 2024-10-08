use clap::{
    builder::{styling::AnsiColor, Styles},
    crate_description, crate_version, Arg, ArgAction, Command as CliCommand,
};
use regex::Regex;
use std::io::Write;
use std::process::Command;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

fn main() {
    // Define command-line arguments using clap
    let matches = CliCommand::new("has")
        .about(crate_description!())
        .version(crate_version!())
        .styles(
            Styles::styled()
                .header(AnsiColor::Green.on_default().bold())
                .usage(AnsiColor::Green.on_default().bold())
                .literal(AnsiColor::Cyan.on_default().bold())
                .placeholder(AnsiColor::Cyan.on_default()),
        )
        .arg_required_else_help(true)
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .action(ArgAction::SetTrue)
                .help("Silent mode"),
        )
        .arg(
            Arg::new("safe")
                .short('s')
                .long("safe")
                .action(ArgAction::SetTrue)
                .help("Only check for known commands"),
        )
        .arg(
            Arg::new("color")
                .long("color")
                .default_value("auto")
                .help("Colorize the output"),
        )
        .arg(
            Arg::new("commands")
                .num_args(1..)
                .help("Command names to check"),
        )
        .get_matches();

    // Get the list of commands
    let commands: Vec<&str> = match matches.get_many::<String>("commands") {
        Some(vals) => vals.map(|v| v.as_str()).collect(),
        None => {
            eprintln!("No commands provided.");
            std::process::exit(1);
        }
    };

    // Determine color choice
    let color_choice = ColorChoice::Auto;
    // match matches.get_one("color").unwrap_or("auto") {
    //     "auto" => {
    //         if atty::is(atty::Stream::Stdout) {
    //             ColorChoice::Auto
    //         } else {
    //             ColorChoice::Never
    //         }
    //     }
    //     "never" => ColorChoice::Never,
    //     "always" => ColorChoice::Always,
    //     _ => ColorChoice::Auto,
    // };

    let allow_unsafe = !matches.get_flag("safe");

    // Create a StandardStream for colored output
    let mut stdout = StandardStream::stdout(color_choice);

    // Initialize counters
    let mut ok = 0;
    let mut ko = 0;

    // Regex for simple version extraction
    let regex_simple_version = Regex::new(r"(\d+\.?){2,3}").unwrap();

    // For each command, check if it exists and try to extract version
    for cmd in commands {
        // Handle aliases
        let command = match_alias(cmd);

        // Check if command exists
        if which::which(command).is_err() {
            // Command not found
            ko += 1;
            if !matches.get_flag("quiet") {
                write_failure(&mut stdout, command);
            }
            continue;
        }

        // Try to get the version
        let (status, version) = get_command_version(command, &regex_simple_version, allow_unsafe);

        if status == -1 {
            // Command not understood
            ko += 1;
            if !matches.get_flag("quiet") {
                write_failure_not_understood(&mut stdout, command);
            }
        } else if status == 127 {
            // Command not installed
            ko += 1;
            if !matches.get_flag("quiet") {
                write_failure(&mut stdout, command);
            }
        } else if status == 0 || status == 141 {
            // Successfully executed
            ok += 1;
            if !matches.get_flag("quiet") {
                write_success(&mut stdout, command, version.as_deref());
            }
        } else {
            // Command is there but we might not have been able to extract version
            ok += 1;
            if !matches.get_flag("quiet") {
                write_success(&mut stdout, command, None);
            }
        }
    }

    // Exit with number of failed commands
    std::process::exit(if ko > 126 { 126 } else { ko });
}

// Function to map aliases to actual command names
fn match_alias(cmd: &str) -> &str {
    match cmd {
        "rust" => "rustc",
        "ssl" => "openssl",
        "openssh" => "ssh",
        "golang" => "go",
        "jre" => "java",
        "jdk" => "javac",
        "nodejs" => "node",
        "httpie" => "http",
        "homebrew" => "brew",
        "postgresql" => "psql",
        _ => cmd,
    }
}

// Function to get command version
fn get_command_version(command: &str, regex: &Regex, allow_unsafe: bool) -> (i32, Option<String>) {
    // Handle special cases similar to the Bash script
    let output = match command {
        // Commands that use '--version'
        "bash" | "zsh" | "fish" | "git" | "hg" | "svn" | "bzr" | "curl" | "wget" | "http"
        | "vim" | "emacs" | "nano" | "brew" | "sed" | "awk" | "grep" | "file" | "sudo" | "gzip"
        | "xz" | "bzip2" | "tar" | "pv" | "docker" | "podman" | "psql" | "gcc" | "make"
        | "cmake" | "g++" | "clang" | "ccache" | "ninja" | "rustc" | "cargo" | "aws" | "eb"
        | "heroku" | "terraform" | "packer" | "vagrant" | "consul" | "nomad" | "unzip" | "pip"
        | "pip3" | "node" | "npm" | "yarn" | "pnpm" | "sqlite3" | "just" => {
            Command::new(command).arg("--version").output()
        }
        // Commands that use '-version'
        "ant" | "java" | "javac" | "scala" | "kotlin" => {
            Command::new(command).arg("-version").output()
        }
        // Commands that use '-v'
        "unzip" | "screen" | "firefox" | "lua" | "luajit" => {
            Command::new(command).arg("-v").output()
        }
        // Commands that use 'version' argument
        "go" | "hugo" => Command::new(command).arg("version").output(),
        // Commands with custom processing
        "openssl" => {
            let output = Command::new(command).arg("version").output();
            if let Ok(output) = &output {
                let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                let version = regex.find(&stdout).map(|m| m.as_str().to_string());
                return (0, version);
            }
            output
        }
        // Fallback to dynamic detection if allowed
        _ => {
            if allow_unsafe {
                // Try common version flags
                let flags = ["--version", "-version", "-v", "-V", "version"];
                for flag in &flags {
                    let output = Command::new(command).arg(flag).output();
                    if let Ok(output) = &output {
                        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
                        let combined_output = stdout + &stderr;
                        if let Some(caps) = regex.captures(&combined_output) {
                            let version = caps.get(0).map(|m| m.as_str().to_string());
                            return (0, version);
                        }
                    }
                }
                // If none of the flags worked, return status 0 without version
                return (0, None);
            } else {
                // Command not understood
                return (-1, None);
            }
        }
    };

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            let combined_output = stdout + &stderr;
            if let Some(caps) = regex.captures(&combined_output) {
                let version = caps.get(0).map(|m| m.as_str().to_string());
                (0, version)
            } else {
                (0, None)
            }
        }
        Err(_) => (127, None),
    }
}

// Functions to write success and failure messages
fn write_success(stdout: &mut StandardStream, command: &str, version: Option<&str>) {
    let checkmark = if atty::is(atty::Stream::Stdout) {
        "✓"
    } else {
        "v"
    };
    stdout
        .set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))
        .unwrap();
    write!(stdout, "{}", checkmark).unwrap();
    stdout.reset().unwrap();
    write!(stdout, " {}", command).unwrap();
    if let Some(version) = version {
        stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true))
            .unwrap();
        write!(stdout, " {}", version).unwrap();
        stdout.reset().unwrap();
    }
    writeln!(stdout).unwrap();
}

fn write_failure(stdout: &mut StandardStream, command: &str) {
    let cross = if atty::is(atty::Stream::Stdout) {
        "✗"
    } else {
        "x"
    };
    stdout
        .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
        .unwrap();
    write!(stdout, "{}", cross).unwrap();
    stdout.reset().unwrap();
    writeln!(stdout, " {}", command).unwrap();
}

fn write_failure_not_understood(stdout: &mut StandardStream, command: &str) {
    let cross = if atty::is(atty::Stream::Stdout) {
        "✗"
    } else {
        "x"
    };
    stdout
        .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))
        .unwrap();
    write!(stdout, "{}", cross).unwrap();
    stdout.reset().unwrap();
    writeln!(stdout, " {} not understood", command).unwrap();
}
