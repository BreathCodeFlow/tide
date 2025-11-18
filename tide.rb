class Tide < Formula
  desc "🌊 An opinionated macOS maintenance orchestrator"
  homepage "https://github.com/BreathCodeFlow/tide"
  version "1.3.1"
  license "MIT"

  # This in-repo formula is for local testing only. The official tap at
  # https://github.com/BreathCodeFlow/homebrew-tap always carries the
  # pinned SHA256 values after each release.
  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/BreathCodeFlow/tide/releases/download/v#{version}/tide-aarch64-apple-darwin.tar.gz"
      sha256 :no_check
    else
      url "https://github.com/BreathCodeFlow/tide/releases/download/v#{version}/tide-x86_64-apple-darwin.tar.gz"
      sha256 :no_check
    end
    bin.install "tide"
  end

  def caveats
    <<~EOS
      To get started with tide:
        tide --init          # Create default config
        tide --list          # List all tasks
        tide                 # Run interactively

      Configuration file: ~/.config/tide/config.toml
    EOS
  end

  test do
    system "#{bin}/tide", "--version"
  end
end
