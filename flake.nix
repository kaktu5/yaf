{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = {
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {inherit system overlays;};
    in {
      formatter = pkgs.alejandra;
      devShells.default = pkgs.mkShell {
        buildInputs = [
          (pkgs.rust-bin.selectLatestNightlyWith (toolchain:
              toolchain.default.override {extensions = ["rust-analyzer" "rust-src"];}))
        ];
      };
      packages.yaf = pkgs.rustPlatform.buildRustPackage {
        name = "yaf";
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;
      };
    });
}
