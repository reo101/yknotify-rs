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

              fido2RequestSound = lib.mkOption {
                description = ''
                  Name of the macOS system sound to play when a new FIDO2 touch request is detected.

                  Overrides the `requestSound` option, which sets the request sound for all types of
                  touch request.
                '';
                type = lib.types.nullOr lib.types.str;
                default = null;
              };

              openPGPRequestSound = lib.mkOption {
                description = ''
                  Name of the macOS system sound to play when a new OpenPGP touch request is
                  detected.

                  Overrides the `requestSound` option, which sets the request sound for all types of
                  touch request.
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

              fido2DismissedSound = lib.mkOption {
                description = ''
                  Name of the macOS system sound to play when a FIDO2 touch request is dismissed.

                  Overrides the `dismissedSound` option, which sets the dismissed sound for all
                  types of touch request.
                '';
                type = lib.types.nullOr lib.types.str;
                default = null;
              };

              openPGPDismissedSound = lib.mkOption {
                description = ''
                  Name of the macOS system sound to play when an OpenPGP touch request is dismissed.

                  Overrides the `dismissedSound` option, which sets the dismissed sound for all
                  types of touch request.
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
                    YKNOTIFY_REQUEST_SOUND = lib.mkIf
                      (cfg.requestSound != null)
                      cfg.requestSound;

                    YKNOTIFY_FIDO2_REQUEST_SOUND = lib.mkIf
                      (cfg.fido2RequestSound != null)
                      cfg.fido2RequestSound;

                    YKNOTIFY_OPENPGP_REQUEST_SOUND = lib.mkIf
                      (cfg.openPGPRequestSound != null)
                      cfg.openPGPRequestSound;

                    YKNOTIFY_DISMISSED_SOUND = lib.mkIf
                      (cfg.dismissedSound != null)
                      cfg.dismissedSound;

                    YKNOTIFY_FIDO2_DISMISSED_SOUND = lib.mkIf
                      (cfg.fido2DismissedSound != null)
                      cfg.fido2DismissedSound;

                    YKNOTIFY_OPENPGP_DISMISSED_SOUND = lib.mkIf
                      (cfg.openPGPDismissedSound != null)
                      cfg.openPGPDismissedSound;
                  };
                };

                assertions = [
                  {
                    assertion = cfg.fido2RequestSound != null -> cfg.requestSound == null;
                    message = ''
                      fido2RequestSound cannot be set at the same time as requestSound. Either set
                      the request sound individually for each type of request, or use requestSound
                      to set a single sound for all requests.
                    '';
                  }
                  {
                    assertion = cfg.openPGPRequestSound != null -> cfg.requestSound == null;
                    message = ''
                      openPGPRequestSound cannot be set at the same time as requestSound. Either set
                      the request sound individually for each type of request, or use requestSound
                      to set a single sound for all requests.
                    '';
                  }
                  {
                    assertion = cfg.fido2DismissedSound != null -> cfg.dismissedSound == null;
                    message = ''
                      fido2DismissedSound cannot be set at the same time as dismissedSound. Either
                      set the dismissed sound individually for each type of request, or use
                      dismissedSound to set a single sound for all requests.
                    '';
                  }
                  {
                    assertion = cfg.openPGPDismissedSound != null -> cfg.requestSound == null;
                    message = ''
                      openPGPDismissedSound cannot be set at the same time as dismissedSound. Either
                      set the dismissed sound individually for each type of request, or use
                      dismissedSound to set a single sound for all requests.
                    '';
                  }
                ];
              };
          };

        default = inputs.self.outputs.darwinModules.yknotify-rs;
      };
    };
  };
}
