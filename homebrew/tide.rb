class Tide < Formula
  desc "🌊 An opinionated macOS maintenance orchestrator with an iocraft-powered interface"
  homepage "https://github.com/BreathCodeFlow/tide"
  version "1.2.2"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/BreathCodeFlow/tide/releases/download/v#{version}/tide-aarch64-apple-darwin.tar.gz"
      sha256 "SHA256_AARCH64_PLACEHOLDER" # aarch64
    else
      url "https://github.com/BreathCodeFlow/tide/releases/download/v#{version}/tide-x86_64-apple-darwin.tar.gz"
      sha256 "SHA256_X86_64_PLACEHOLDER" # x86_64
    end
  end

  def install
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
