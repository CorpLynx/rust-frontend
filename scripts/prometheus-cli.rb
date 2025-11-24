# Homebrew Formula for Prometheus CLI
# To use: brew install --formula ./scripts/prometheus-cli.rb

class PrometheusCli < Formula
  desc "Terminal-based AI chat interface for Prometheus"
  homepage "https://github.com/your-username/prometheus"
  url "https://github.com/your-username/prometheus/archive/refs/tags/v0.2.0.tar.gz"
  sha256 "YOUR_SHA256_HERE"
  license "MIT"
  head "https://github.com/your-username/prometheus.git", branch: "main"

  depends_on "rust" => :build
  depends_on "ollama"

  def install
    # Build the CLI
    system "cargo", "build", "--release", "-p", "prometheus-cli"
    
    # Install binary
    bin.install "target/release/prometheus-cli"
    
    # Install man page
    man1.install "docs/prometheus-cli.1" if File.exist?("docs/prometheus-cli.1")
    
    # Install default config
    (etc/"prometheus").mkpath
    (etc/"prometheus/config.toml").write <<~EOS
      [app]
      window_title = "Prometheus v0.2.0"

      [backend]
      url = "http://localhost:11434"
      ollama_url = "http://localhost:11434"
      timeout_seconds = 30

      [ui]
      font_size = 16
      max_chat_history = 1000
      theme = "Hacker Green"
    EOS
  end

  def post_install
    # Create conversations directory
    (var/"prometheus/conversations").mkpath
    
    # Copy config to user directory if it doesn't exist
    config_dir = Pathname.new(Dir.home)/".config/prometheus"
    config_dir.mkpath
    
    unless (config_dir/"config.toml").exist?
      cp etc/"prometheus/config.toml", config_dir/"config.toml"
    end
  end

  def caveats
    <<~EOS
      Prometheus CLI has been installed!

      Configuration file: ~/.config/prometheus/config.toml
      Conversations: #{var}/prometheus/conversations

      To get started:
        1. Start Ollama: ollama serve
        2. Run: prometheus-cli
        3. Type /help for available commands

      For more information:
        man prometheus-cli
        prometheus-cli --help
    EOS
  end

  test do
    assert_match "prometheus-cli", shell_output("#{bin}/prometheus-cli --version")
  end
end
