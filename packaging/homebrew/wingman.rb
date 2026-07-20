# Homebrew formula template for Wingman.
#
# Placeholders (__VERSION__, __SHA_*) are filled by scripts/stamp-packaging.ps1
# from a release tag, then the result is submitted to a tap
# (e.g. homebrew-wingman) so users can `brew install vedantnimbarte/wingman/wingman`.
class Wingman < Formula
  desc "Multi-provider, terminal-first, self-improving coding agent in Rust"
  homepage "https://github.com/vedantnimbarte/Wingman"
  version "__VERSION__"
  license "Apache-2.0"

  on_macos do
    on_arm do
      url "https://github.com/vedantnimbarte/Wingman/releases/download/v__VERSION__/wingman-aarch64-apple-darwin.tar.gz"
      sha256 "__SHA_MACOS_ARM__"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/vedantnimbarte/Wingman/releases/download/v__VERSION__/wingman-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "__SHA_LINUX_X64__"
    end
    on_arm do
      url "https://github.com/vedantnimbarte/Wingman/releases/download/v__VERSION__/wingman-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "__SHA_LINUX_ARM__"
    end
  end

  def install
    bin.install "wingman"
  end

  test do
    assert_match "wingman", shell_output("#{bin}/wingman --version")
  end
end
