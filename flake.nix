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
          in
          {
            options.services.yknotify-rs = {
              enable = lib.mkEnableOption "Enable yknotify-rs launchd service";

              requestSound = lib.mkOption {
                description = ''
                  Name of the macOS system sound to play when a new touch request is detected.
                  
                  Available sounds can be found in `/System/Library/Sounds`, `/Library/Sounds` or
                  `~/Library/Sounds`. The sound name must be a filename without an extension, e.g.
                  `Purr`.
                '';
                type = lib.types.nullOr lib.types.str;
                default = null;
              };

              dismissedSound = lib.mkOption {
                description = ''
                  Name of the macOS system sound to play when a new touch request is detected.
                  
                  Available sounds can be found in `/System/Library/Sounds`, `/Library/Sounds` or
                  `~/Library/Sounds`. The sound name must be a filename without an extension, e.g.
                  `Pop`.
                '';
                type = lib.types.nullOr lib.types.str;
                default = null;
              };
            };

            config =
              let
                cfg = config.services.yknotify-rs;
              in
              lib.mkIf cfg.enable {
                launchd.user.agents.yknotify-rs = {
                  serviceConfig = {
                    Label = "xyz.reo101.yknotify-rs";
                    Program = lib.getExe yknotify-rs;
                    RunAtLoad = true;
                    KeepAlive = true;
                    # StandardOutPath = "/var/log/${Label}/stdout.log";
                    # StandardErrorPath = "/var/log/${Label}/stderr.log";
                  };
                  environment = {
                    YKNOTIFY_REQUEST_SOUND = lib.mkIf (cfg.requestSound != null)
                      cfg.requestSound;
                    YKNOTIFY_DISMISSED_SOUND = lib.mkIf (cfg.dismissedSound != null)
                      cfg.dismissedSound;
                  };
                };
              };
          };

        default = inputs.self.outputs.darwinModules.yknotify-rs;
      };
    };
  };
}
