{
  description = "Notify when YubiKey needs touch on macOS";

  inputs = {
    nixpkgs = {
      url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    };
    # nix-darwin = {
    #   url = "github:LnL7/nix-darwin";
    # };
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
    };
  };

  outputs = inputs: inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "aarch64-darwin"
        "x86_64-darwin"
      ];

      perSystem = { pkgs, system, ... }: {
        packages.yknotify-rs = pkgs.rustPlatform.buildRustPackage {
          pname = "yknotify-rs";
          version = "0.1.0";

          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [
            pkgs.pkg-config
          ];
          buildInputs = [
            pkgs.openssl
          ];
        };
      };

    flake = {
      darwinModules = {
        yknotify-rs = { config, pkgs, lib, ... }:
          let
            yknotify-rs = pkgs.writeShellScriptBin "yknotify-rs" ''
              ${lib.getExe pkgs.yknotify-rs} >> $HOME/Library/Logs/yknotify-rs.log 2>&1
            '';
          in {
            options.services.yknotify-rs = {
              enable = lib.mkEnableOption "Enable yknotify-rs launchd service";
            };

            config = lib.mkIf config.services.yknotify-rs.enable {
              services.launchd.agents.yknotify-rs = {
                enable = true;
                config = {
                  Label = "xyz.reo101.yknotify-rs";
                  ProgramArguments = [
                    (lib.getExe yknotify-rs)
                  ];
                  RunAtLoad = true;
                  KeepAlive = true;
                  StandardErrorPath = "$HOME/Library/Logs/yknotify-rs.log";
                  StandardOutPath = "$HOME/Library/Logs/yknotify-rs.log";
                };
              };
            };
          };

        default = inputs.self.outputs.darwinModules.yknotify-rs;
      };
    };
  };
}
