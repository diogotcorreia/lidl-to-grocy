{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
    in {
      packages = rec {
        lidl-to-grocy = pkgs.callPackage ./nix/package.nix {};
        default = lidl-to-grocy;
      };

      devShell = pkgs.mkShell {
        buildInputs = with pkgs; [
          cargo
          openssl
          pkg-config
          rustc
          rustfmt
          rust-analyzer
          clippy
        ];

        shellHook = ''
          export RUST_LOG=debug
        '';
      };
    });
}
