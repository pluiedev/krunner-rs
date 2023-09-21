{
  inputs.nixpkgs.url = "nixpkgs";

  outputs = {nixpkgs, ...}: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
  in {
    packages.${system} = rec {
      default = pkgs.callPackage ./. {};
    };

    devShells.${system}.default = with pkgs;
      mkShell {
        nativeBuildInputs = [
          pkg-config
          dbus
        ];
        buildInputs = [
          libsForQt5.kdialog
        ];
        LD_LIBRARY_PATH = lib.makeLibraryPath [dbus];
      };
  };
}
