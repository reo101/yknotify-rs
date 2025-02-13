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

    perSystem = { lib, pkgs, system, ... }: {
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

        meta = {
          description = "Notify when YubiKey needs touch on macOS";
          homepage = "https://github.com/reo101/yknotify-rs";
          license = lib.licenses.mit;
          mainProgram = "yknotify-rs";
          platforms = lib.platforms.darwin;
        };
      };
    };

    flake = {
      darwinModules = {
        yknotify-rs = { config, pkgs, lib, ... }:
          let
            yknotify-rs = inputs.self.packages.${pkgs.hostPlatform.system}.yknotify-rs;
          in {
            options.services.yknotify-rs = {
              enable = lib.mkEnableOption "Enable yknotify-rs launchd service";
            };

            config = lib.mkIf config.services.yknotify-rs.enable {
              launchd.user.agents.yknotify-rs = {
                script = lib.getExe yknotify-rs;
                serviceConfig = rec {
                  Label = "xyz.reo101.yknotify-rs";
                  ProgramArguments = [ ];
                  RunAtLoad = true;
                  KeepAlive = true;
                  StandardOutPath = "/var/log/${Label}/stdout.log";
                  StandardErrorPath = "/var/log/${Label}/stderr.log";
                };
              };
            };
          };

        default = inputs.self.outputs.darwinModules.yknotify-rs;
      };
    };
  };
}
