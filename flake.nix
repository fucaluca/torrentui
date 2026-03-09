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
    , crane
    , fenix
    , ...
    }:
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      rust = fenix.packages.x86_64-linux.fromToolchainFile {
        file = ./rust-toolchain.toml;
        sha256 = "sha256-SBKjxhC6zHTu0SyJwxLlQHItzMzYZ71VCWQC2hOzpRY=";
      };
      craneLib = (crane.mkLib pkgs).overrideToolchain rust;
      cargoArtifacts = craneLib.buildDepsOnly {
        src = craneLib.cleanCargoSource ./.;
      };
      package = craneLib.buildPackage {
        src = craneLib.cleanCargoSource ./.;
        nativeBuildInputs = [ pkgs.pkg-config ];
        buildInputs = [ pkgs.cacert ];
      };
      cov-report = pkgs.writeShellScriptBin "cov-report" ''
        ${rust}/bin/cargo llvm-cov nextest --lcov --output-path target/coverage.lcov
      '';
      nt = pkgs.writeShellScriptBin "nt" ''
        ${rust}/bin/cargo nextest run
      '';
      cov-html = pkgs.writeShellScriptBin "cov-html" ''
        export LLVM_PROFILE_FILE="target/grcov/torrentui.profraw"
        export RUSTFLAGS="-Cinstrument-coverage"
        ${rust}/bin/cargo test
        grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/grcov/html
        xdg-open target/grcov/html/index.html
        unset LLVM_PROFILE_FILE
        unset RUSTFLAGS
      '';

    in
    {
      packages.x86_64-linux.default = package;
      apps.x86_64-linux.default = {
        type = "app";
        program = "${self.packages.x86_64-linux.default}/bin/torrentui";
      };

      devShells.x86_64-linux.default = pkgs.mkShell {
        inputsFrom = [ self.packages.x86_64-linux.default ];
        buildInputs = with pkgs; [
          cacert
          pkg-config
        ];
        nativeBuildInputs = with pkgs; [
          cargo-machete
          cargo-watch
          cargo-edit
          cargo-nextest
          cargo-tarpaulin
          cargo-llvm-cov
          cov-report
          cov-html
          nt
          grcov
          tea

          # Pre-commit dependencies
          pre-commit
          nixpkgs-fmt
          statix
          deadnix
        ];
        # RUSTFLAGS = "-Cinstrument-coverage";
        # LLVM_PROFILE_FILE = "target/coverage/torrentui.profraw";
        shellHook = ''
          pre-commit install
        '';
      };

      checks.x86_64-linux = {
        torrentui-test = craneLib.cargoNextest {
          src = craneLib.cleanCargoSource ./.;
          cargoArtifacts = cargoArtifacts;
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.cacert ];
          env = {
            # SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
            # NIX_SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
          };
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
            enable = lib.mkEnableOption "A lightweight TUI client for rqbit written in Rust";
            /* settings = lib.mkOption {
              type =
                with lib.types;
                submodule {
                  options = { };
                };
              default = { };
            }; */
            settings = lib.mkOption
              {
                type = lib.types.submodule {
                  options = {
                    notification_timeout_millis = lib.mkOption {
                      type = lib.types.int;
                      example = 100;
                      default = 100;
                    };
                    player_cmd = lib.mkOption {
                      type = lib.types.str;
                      example = "setsid mpv";
                      default = "setsid mpv";
                    };
                    auto_show_help = lib.mkOption {
                      type = lib.types.bool;
                      example = true;
                      default = true;
                    };
                    auto_insert_magnet = lib.mkOption {
                      type = lib.types.bool;
                      example = true;
                      default = true;
                    };
                    keybindings = lib.mkOption {
                      type = lib.types.submodule {
                        options = {
                          torrent-list = lib.mkOption {
                            type = lib.types.attrsOf (lib.types.oneOf [
                              lib.types.str
                              (lib.types.submodule {
                                options = {
                                  description = lib.mkOption {
                                    type = lib.types.str;
                                    description = "Description";
                                    example = "Add magnet";
                                  };
                                  action = lib.mkOption {
                                    type = lib.types.nullOr lib.types.str;
                                    default = null;
                                    description = "Action";
                                  };
                                };
                              })
                            ]);
                            default = { };
                            example = {
                              "<q>" = "Quit";
                              "<space><a>" = { action = "AddMagnet"; description = "Add magnet"; };
                              "<ctrl-d>" = { description = "Delete"; };
                              "<ctrl-d><d>" = { action = "Delete"; description = "Forget about the torrent, remove the files"; };
                            };
                            description = "Keybindings for the torrent list screen.";
                          };
                          add-magnet = lib.mkOption {
                            type = lib.types.submodule {
                              options = {
                                input = lib.mkOption {
                                  type = lib.types.attrsOf lib.types.str;
                                  default = { };
                                  example = { "<k>" = "Up"; };
                                  description = "Keybindings for the add magnet input screen.";
                                };
                                connectors = lib.mkOption {
                                  type = lib.types.attrsOf lib.types.str;
                                  default = { };
                                  example = { "<j>" = "Down"; };
                                  description = "Keybindings for the add magnet connectors screen.";
                                };
                              };
                            };
                            default = { };
                          };
                        };
                      };
                      default = { };
                      description = "Keybindings per screen.";
                    };
                    connectors = lib.mkOption {
                      type = lib.types.attrsOf (lib.types.submodule {
                        options = {
                          kind = lib.mkOption {
                            type = lib.types.enum [ "rqbit" ];
                            description = "Type of connector (currently only rqbit is supported) ";
                            default = "rqbit";
                            example = "rqbit";
                          };
                          url = lib.mkOption {
                            type = lib.types.str;
                            description = "Base URL of the rqbit API";
                            default = "http://localhost:3030";
                            example = "http://localhost:3030";
                          };
                          api_version = lib.mkOption {
                            type = lib.types.enum [ "v8" ];
                            description = "API version (currently only v8 is supported)";
                            default = "v8";
                            # description = "API version (v8 or v9)";
                            example = "v8";
                          };
                          update_interval_secs = lib.mkOption {
                            type = lib.types.int;
                            default = 1;
                            example = 1;
                            description = "Update interval in seconds";
                          };
                          selected_by_default = lib.mkOption {
                            type = lib.types.bool;
                            default = true;
                            example = true;
                            description = "Whether this connector is selected by default";
                          };
                        };
                      });
                      default = { };
                      description = "Connectors to rqbit instances";
                    };
                    styles = lib.mkOption {
                      type = lib.types.submodule {
                        options = {
                          active = lib.mkOption {
                            type = lib.types.attrsOf lib.types.str;
                            default = { };
                            example = {
                              upload = "black on yellow";
                              download = "black on blue";
                            };
                            description = "Styles for the torrent list screen.";
                          };
                          paused = lib.mkOption {
                            type = lib.types.attrsOf lib.types.str;
                            default = { };
                            example = { default = "gray on rgb:0,0,0"; };
                            description = "Styles for the torrent list screen.";
                          };
                          notification = lib.mkOption {
                            type = lib.types.attrsOf lib.types.str;
                            default = { };
                            example = { info = "blue on rgb:0,0,0"; };
                            description = "Styles for the torrent list screen.";
                          };
                          which-key = lib.mkOption {
                            type = lib.types.attrsOf lib.types.str;
                            default = { };
                            example = { key = "yellow on rgb:30,30,30"; };
                            description = "Styles for the torrent list screen.";
                          };
                          default = lib.mkOption {
                            type = lib.types.attrsOf lib.types.str;
                            default = { };
                            example = { highlight = "yellow on black"; };
                            description = "Default styles.";
                          };
                          add-magnet = lib.mkOption {
                            type = lib.types.submodule {
                              options = {
                                input = lib.mkOption {
                                  type = lib.types.attrsOf lib.types.str;
                                  default = { };
                                  example = { border = "blue on rgb:30,30,30"; };
                                  description = "Styles for the add magnet input screen";
                                };
                                connectors = lib.mkOption {
                                  type = lib.types.attrsOf lib.types.str;
                                  default = { };
                                  example = { input = "dark gray on rgb:30,30,30"; };
                                  description = "Styles for the add magnet input screen";
                                };
                              };
                            };
                            default = { };
                          };
                        };
                      };
                      default = { };
                    };
                  };
                };
                default = { };
                description = "TorrenTUI configuration.";
              };
          };

          config = lib.mkIf config.programs.torrentui.enable {
            home.packages = [ self.packages.${pkgs.system}.default ];
            xdg.configFile."torrentui/config.toml" =
              let
                tomlFormat = pkgs.formats.toml { };
                cleanNulls = attrs:
                  builtins.mapAttrs
                    (name: value:
                      if builtins.isAttrs value && ! builtins.isList value
                      then cleanNulls value   # рекурсивно чистим вложенные атрибуты
                      else value
                    )
                    (lib.filterAttrs (name: value: value != null) attrs);
              in
              {
                source = tomlFormat.generate "torrentui-config" (cleanNulls config.programs.torrentui.settings);
              };
          };
        };
    };
}
