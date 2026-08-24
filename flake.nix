{
  description = "Sleepy Linux versioned document contract SDK";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      packageFor = system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        pkgs.rustPlatform.buildRustPackage {
          pname = "sleepy-sdk";
          version = "0.1.0";
          src = self;
          cargoLock.lockFile = ./Cargo.lock;
          meta.license = pkgs.lib.licenses.gpl3Only;
        };
    in
    {
      packages = forAllSystems (system:
        let package = packageFor system;
        in {
          default = package;
          sleepy-contract = package;
        });

      checks = forAllSystems (system: {
        contracts = packageFor system;
      });
    };
}
