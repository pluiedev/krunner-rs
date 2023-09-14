{
  lib,
  pkg-config,
  dbus,
  fetchFromGitHub,
  rustPlatform,
}:
rustPlatform.buildRustPackage rec {
  pname = "krunner_nix";
  version = "0.1.0";

  src = ./.;

  nativeBuildInputs = [pkg-config];
  buildInputs = [dbus];

  cargoLock.lockFile = ./Cargo.lock;

  meta = with lib; {
    description = "Write KRunner plugins with dbus-rs.";
    homepage = "https://github.com/pluiedev/krunner_nix";
    license = licenses.mit;
    maintainers = [];
  };
}
