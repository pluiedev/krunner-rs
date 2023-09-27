{
  inputs.nixpkgs.url = "nixpkgs";

  outputs = {nixpkgs, ...}: let
    systems = ["x86_64-linux" "aarch64-linux"];
    forAllSystems = f: nixpkgs.lib.genAttrs systems (sys: f nixpkgs.legacyPackages.${sys});
  in {
    devShells = forAllSystems (pkgs: {
      default = with pkgs;
        mkShell {
          nativeBuildInputs = [pkg-config];
          buildInputs = [dbus];

          LD_LIBRARY_PATH = lib.makeLibraryPath [dbus];
        };
    });
  };
}
