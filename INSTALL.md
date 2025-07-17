# Installing needs via Homebrew

This guide explains how to install the `needs` CLI tool using Homebrew on both macOS and Linux.

## Prerequisites

- **Homebrew**: Install from [brew.sh](https://brew.sh/)
- **Rust**: Will be automatically installed by Homebrew as a build dependency

## Installation Options

### Option 1: Install from Local Formula (Current)

Since the project doesn't have GitHub releases yet, you can install directly from the local formula:

```bash
# Clone the repository
git clone https://github.com/NQMVD/needs.git
cd needs

# Install using the local formula
brew install --build-from-source ./needs.rb
```

### Option 2: Install from GitHub (Future)

Once GitHub releases are available, you can install directly:

```bash
# This will work once releases are published
brew install https://raw.githubusercontent.com/NQMVD/needs/master/needs-release.rb
```

### Option 3: Create a Homebrew Tap (Recommended for Distribution)

For easier distribution, consider creating a Homebrew tap:

```bash
# Create a tap repository
brew tap NQMVD/needs https://github.com/NQMVD/homebrew-needs

# Install from the tap
brew install needs
```

## Platform Support

The formula supports:
- **macOS**: Intel (x86_64) and Apple Silicon (ARM64)
- **Linux**: x86_64 (via Homebrew on Linux)

## Usage After Installation

Once installed, `needs` will be available globally:

```bash
# Check if binaries are available
needs git cargo rust

# Quick check without versions
needs --no-versions git cargo rust

# Check from a needsfile
echo "git\ncargo\nrust" > needsfile
needs

# Quiet mode (exit codes only)
needs --quiet git cargo rust
```

## Uninstallation

To remove the installed package:

```bash
brew uninstall needs
```

## Troubleshooting

### Common Issues

1. **Permission denied**: Make sure you have write access to Homebrew directories
2. **Build failures**: Ensure Rust is properly installed
3. **Binary not in PATH**: Check if Homebrew's bin directory is in your PATH

### Build from Source Issues

If you encounter issues with the build process:

```bash
# Clean up and try again
brew cleanup
brew install --build-from-source ./needs.rb --verbose
```

### Getting Help

- Check the [main README](README.md) for usage instructions
- View the [Homebrew documentation](HOMEBREW.md) for formula details
- Open an issue on the [GitHub repository](https://github.com/NQMVD/needs/issues)

## Development

To modify the formula:

1. Edit `needs.rb` or `needs-release.rb`
2. Test with `./test-formula.sh`
3. Install and test: `brew install --build-from-source ./needs.rb`

## Contributing

Contributions to improve the Homebrew formula are welcome! Please:

1. Test your changes on both macOS and Linux
2. Run the test script to verify functionality
3. Update documentation as needed
4. Submit a pull request