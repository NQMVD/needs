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
    let matches = CliCommand::new("needs")
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

    // Create a StandardStream for colored output
    let mut stdout = StandardStream::stdout(color_choice);

    // Regex for simple version extraction
    let regex_simple_version = Regex::new(r"(\d+\.?){2,3}").unwrap();

    // For each command, check if it exists and try to extract version
    for cmd in commands {
        // Handle aliases
        let command = match_alias(cmd);

        // Check if command exists
        if which::which(command).is_err() {
            // Command not found
            if matches.get_flag("quiet") {
                // already exit if in quiet mode
                std::process::exit(1);
            } else {
                write_failure(&mut stdout, command);
                continue;
            }
        }

        // Try to get the version
        let version = get_command_version(command, &regex_simple_version);

        if version.is_some() {
            // Command found and version found
            if !matches.get_flag("quiet") {
                write_success(&mut stdout, command, version.as_deref());
            }
        } else {
            // Command found but version not found
            if !matches.get_flag("quiet") {
                write_failure(&mut stdout, command);
                continue;
            }
        }
    }
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
fn get_command_version(command: &str, regex: &Regex) -> Option<String> {
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
                return version;
            }
        }
    }
    // If none of the flags worked, return status 0 without version
    None
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
