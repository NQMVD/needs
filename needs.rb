class Needs < Formula
  desc "Check if given bin(s) are available in the PATH and get their versions"
  homepage "https://github.com/NQMVD/needs"
  url "https://github.com/NQMVD/needs.git", branch: "master"
  version "0.6.0"
  license "GPL-3.0-or-later"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--locked", "--root", prefix, "--path", "."
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
  end
end