{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
    nixos-unstable.url = "github:nixos/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    fenix.url = "github:nix-community/fenix";
  };

  outputs =
    { self
    , nixpkgs
    , nixos-unstable
    , crane
    , fenix
    , ...
    }:
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      unstable = nixos-unstable.legacyPackages.x86_64-linux;
      rust = fenix.packages.x86_64-linux.fromToolchainFile {
        file = ./rust-toolchain.toml;
        sha256 = "sha256-sqSWJDUxc+zaz1nBWMAJKTAGBuGWP25GCftIOlCEAtA=";
      };
      craneLib = (crane.mkLib pkgs).overrideToolchain rust;
      cargoArtifacts = craneLib.buildDepsOnly {
        src = craneLib.cleanCargoSource ./.;
      };
      package = craneLib.buildPackage {
        src = craneLib.cleanCargoSource ./.;
      };
    in
    {
      packages.x86_64-linux.default = package;
      apps.x86_64-linux.default = {
        type = "app";
        program = "${self.packages.x86_64-linux.default}/bin/torrentui";
      };

      devShells.x86_64-linux.default = pkgs.mkShell {
        inputsFrom = [ self.packages.x86_64-linux.default ];
        nativeBuildInputs = with pkgs; [
          pkg-config
          unstable.mermaid-cli
          cargo-watch
          cargo-edit
          cargo-nextest
          gh

          # Pre-commit dependencies
          pre-commit
          nixpkgs-fmt
          statix
          deadnix
        ];
        shellHook = ''
          pre-commit install
        '';
      };

      checks.x86_64-linux = {
        torrentui-test = craneLib.cargoNextest {
          src = craneLib.cleanCargoSource ./.;
          cargoArtifacts = cargoArtifacts;
        };

        torrentui-clippy = craneLib.cargoClippy {
          src = craneLib.cleanCargoSource ./.;
          cargoArtifacts = cargoArtifacts;
          cargoClippyExtraArgs = "-- -D warnings";
        };

        torrentui-build = package;
      };

      nixosModules.torrentui =
        { config
        , lib
        , pkgs
        , ...
        }:
        {
          options.programs.torrentui = {
            enable = lib.mkEnableOption "TorrenTUI is TUI for Rqbit and others";
            settings = lib.mkOption {
              type =
                with lib.types;
                submodule {
                  options = { };
                };
              default = { };
            };
          };

          config = lib.mkIf config.programs.torrentui.enable {
            home.packages = [ self.packages.${pkgs.system}.default ];
            xdg.configFile."torrentui/config.toml" =
              let
                tomlFormat = pkgs.formats.toml { };
              in
              {
                source = tomlFormat.generate "torrentui-config" config.programs.torrentui.settings;
              };
          };
        };
    };
}
