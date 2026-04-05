# Homebrew formula (Sprint 19). Install from a tap or local path:
#   brew install --formula ./packaging/homebrew/cantrik.rb
# Uses --HEAD to build from main; after tagging vX.Y.Z, add a `stable` block with
# `url` + `tag` + `revision` per https://docs.brew.sh/Formula-Cookbook
class Cantrik < Formula
  desc "Open-source AI CLI agent (Rust)"
  homepage "https://github.com/sangkan-dev/cantrik"
  license "MIT"
  head "https://github.com/sangkan-dev/cantrik.git", branch: "main"

  depends_on "protobuf" => :build
  depends_on "rust" => :build

  def install
    ENV["PROTOC"] = Formula["protobuf"].opt_bin/"protoc"
    system "cargo", "install", *std_cargo_args(path: "crates/cantrik-cli")
  end

  test do
    assert_match(/cantrik/i, shell_output("#{bin}/cantrik --version"))
  end
end
