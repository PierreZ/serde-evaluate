{
  description = "A Rust development flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [
            # Use the rust toolchain provided by the overlay
            rust-bin.stable.latest.default

            # Add other development tools if needed
            pkg-config
            openssl
            # Example: add native dependencies
            # openssl.dev
            # sqlite
          ];

          # Set environment variables if needed
          # RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
        };
      });
}
