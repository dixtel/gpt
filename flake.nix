{
  description = "gpt";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/release-24.11";
  };

  outputs = { self, nixpkgs }:
    let
      supportedSystems = [ "x86_64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      pkgs = forAllSystems (system: nixpkgs.legacyPackages.${system});
    in
    {
      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/gpt";
        };
      });

      packages = forAllSystems (system: {
        default = with pkgs.${system}; rustPlatform.buildRustPackage {
          pname = "gpt";
          version = "0.2";
          cargoLock.lockFile = ./Cargo.lock;
          src = nixpkgs.lib.cleanSource ./.;

          nativeBuildInputs = [
            pkg-config
          ];

          buildInputs = [ openssl ];
        };
      });

      devShells = forAllSystems (system: {
        default = pkgs.${system}.mkShell {
          packages = with pkgs.${system}; [
            pkg-config
            openssl
           ];
        };
      });
    };
}
