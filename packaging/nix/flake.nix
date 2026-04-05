{
  description = "Cantrik: Nix dev shell (Rust + protoc). Packaged binary via nixpkgs is deferred; see README.md in this directory.";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      forAll = f: nixpkgs.lib.genAttrs systems (system: f (import nixpkgs { inherit system; }));
    in
    {
      devShells = forAll (pkgs: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            rustc
            cargo
            rustfmt
            clippy
            protobuf
            pkg-config
            openssl
          ];
          PROTOC = "${pkgs.protobuf}/bin/protoc";
          RUST_BACKTRACE = "1";
        };
      });
    };
}
