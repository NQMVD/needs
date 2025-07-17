# Homebrew Formula for needs

This directory contains Homebrew formulas for the `needs` CLI tool.

## Files

- `needs.rb` - Development formula that builds from the current master branch
- `needs-release.rb` - Production formula template for use with GitHub releases

## Usage

### For Development/Testing

The `needs.rb` formula builds from the current master branch and is suitable for development and testing:

```bash
# Install directly from the local formula
brew install --build-from-source ./needs.rb

# Or if you want to test the formula
brew install --formula ./needs.rb
```

### For Production Use

The `needs-release.rb` formula is designed for use with proper GitHub releases:

1. Create a GitHub release with a tag (e.g., `v0.6.0`)
2. Calculate the SHA256 hash of the release tarball
3. Update the `sha256` field in the formula
4. Submit to homebrew-core or create your own tap

## Platform Support

Both formulas work on:
- **macOS**: Intel and Apple Silicon
- **Linux**: x86_64 (via Homebrew on Linux)

## Dependencies

- **Rust**: Required at build time to compile the Rust source code
- **Cargo**: Comes with Rust and is used for building

## Installation Process

The formula:
1. Downloads the source code
2. Uses `cargo install` to build and install the binary
3. Places the binary in the appropriate Homebrew bin directory

## Testing

The formula includes comprehensive tests that verify:
- Binary installation
- Help text display
- Version output
- Basic functionality with common binaries
- Multi-binary checking

## Usage After Installation

Once installed via Homebrew, you can use `needs` from anywhere:

```bash
# Check if specific binaries are available
needs git cargo rust

# Check without version retrieval (faster)
needs --no-versions git cargo rust

# Quiet mode (exit codes only)
needs --quiet git cargo rust

# Show full version strings
needs --full-versions git cargo rust

# Use a needsfile
echo "git\ncargo\nrust" > needsfile
needs
```

## Troubleshooting

If you encounter issues:

1. **Build fails**: Ensure you have Rust installed (`brew install rust`)
2. **Binary not found**: Check if Homebrew's bin directory is in your PATH
3. **Permission issues**: Make sure you have write permissions to Homebrew directories

## Contributing

To update the formula:
1. Update the version in `Cargo.toml`
2. Update the version in the formula file
3. If using releases, update the URL and SHA256 hash
4. Test the formula locally before submitting

## Future Improvements

- Add support for precompiled binaries once GitHub releases with assets are available
- Consider creating a homebrew tap for easier distribution
- Add support for different installation options (with/without version-retrieval feature)