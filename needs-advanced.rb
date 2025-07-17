# Advanced Homebrew Formula for needs

class Needs < Formula
  desc "Check if given bin(s) are available in the PATH and get their versions"
  homepage "https://github.com/NQMVD/needs"
  url "https://github.com/NQMVD/needs.git", branch: "master"
  version "0.6.0"
  license "GPL-3.0-or-later"

  depends_on "rust" => :build

  def install
    # Install with locked dependencies for reproducible builds
    system "cargo", "install", "--locked", "--root", prefix, "--path", "."
    
    # Generate shell completions if supported in future versions
    # (This is a placeholder for future enhancement)
    
    # Install documentation
    doc.install "README.md"
    doc.install "CHANGELOG.md" if File.exist?("CHANGELOG.md")
    
    # Install additional files
    pkgshare.install "needsfile" if File.exist?("needsfile")
    pkgshare.install "needsfiles" if Dir.exist?("needsfiles")
  end

  def post_install
    # Create a sample needsfile if one doesn't exist
    (var/"needs").mkpath
    sample_needsfile = var/"needs/needsfile.example"
    sample_needsfile.write <<~EOS
      # Example needsfile
      # List one binary per line
      git
      cargo
      rust
      node
      python
    EOS
  end

  test do
    # Test that the binary was installed and shows help
    assert_match "Check if given bin(s) are available in the PATH", shell_output("#{bin}/needs --help")
    
    # Test version flag
    assert_match "needs #{version}", shell_output("#{bin}/needs --version")
    
    # Test basic functionality with a binary that should exist on both macOS and Linux
    system "#{bin}/needs", "--quiet", "sh"
    
    # Test with a binary that typically exists and check output format
    output = shell_output("#{bin}/needs --no-versions sh")
    assert_match "sh", output
    
    # Test with multiple binaries
    output = shell_output("#{bin}/needs --no-versions sh ls")
    assert_match "sh", output
    assert_match "ls", output
    
    # Test error handling with non-existent binary
    assert_match "not found", shell_output("#{bin}/needs --no-versions nonexistent_binary_xyz")
    
    # Test quiet mode exits with correct code
    system "#{bin}/needs", "--quiet", "sh"
    assert_equal 0, $CHILD_STATUS.exitstatus
  end

  def caveats
    <<~EOS
      To get started with needs:
      
      1. Check if specific binaries are available:
         needs git cargo rust
      
      2. Use a needsfile for project dependencies:
         echo "git\ncargo\nrust" > needsfile
         needs
      
      3. For faster checks, disable version retrieval:
         needs --no-versions git cargo rust
      
      Example needsfile installed to:
        #{var}/needs/needsfile.example
      
      For more information, see:
        #{doc}/README.md
    EOS
  end
end