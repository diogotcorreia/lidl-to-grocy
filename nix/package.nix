{
  rustPlatform,
  openssl,
  pkg-config,
}:
rustPlatform.buildRustPackage {
  pname = "lidl-to-grocy";
  version = "1.3.1";

  nativeBuildInputs = [pkg-config];

  buildInputs = [openssl];

  src = ../.;
  cargoLock.lockFile = ../Cargo.lock;
}
