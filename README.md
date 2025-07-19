# yknotify-rs

`yknotify-rs` is a Rust rewrite of [`yknotify`](https://github.com/noperator/yknotify). It watches macOS logs (via `log stream`) for heuristically determined events that indicate a YubiKey is waiting for touch.

I primarily use the FIDO2 and OpenPGP features and haven't tested other applications listed in `ykman info` (e.g., Yubico OTP, FIDO U2F, OATH, PIV, YubiHSM Auth).

## Detection Strategy

When waiting for FIDO2 touch, the following log message appears once (with a sample hex value):

```
kernel: (IOHIDFamily) IOHIDLibUserClient:0x123456789 startQueue
```

When waiting for OpenPGP touch, this message appears repeatedly:

```
usbsmartcardreaderd: [com.apple.CryptoTokenKit:ccid] Time extension received
```


As soon as the YubiKey is touched, a new/different log message appears in the same category. The strategy here is to check whether either of the above messages are the last logged event in their respective categories, and if so, notify the user to touch the YubiKey.

## Installation

### Using `nix`

#### Run without installation:

```sh
nix run github:reo101/yknotify-rs
```

Install it permanently:

```sh
nix profile install github:reo101/yknotify-rs
```

### Using `nix-darwin`:

A nix-darwin module is available for managing yknotify-rs as a macOS LaunchAgent. Add this to your darwin-configuration.nix:

```nix
{
  description = "Example nix-darwin configuration using yknotify-rs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nix-darwin.url = "github:LnL7/nix-darwin";
    flake-parts.url = "github:hercules-ci/flake-parts";
    yknotify-rs.url = "github:reo101/yknotify-rs";
  };

  outputs = inputs: inputs.flake-parts.lib.mkFlake { inherit inputs; } {
    systems = [
      "aarch64-darwin"
      "x86_64-darwin"
    ];

    flake = {
      darwinConfigurations."my-mac" = nix-darwin.lib.darwinSystem {
        system = "aarch64-darwin";
        modules = [
          yknotify-rs.darwinModules.default
          {
            services.yknotify-rs = {
              enable = true;

              # You can set notification sounds (find available sounds in `/System/Library/Sounds`):
              requestSound = "Purr";
              dismissedSound = "Pop";
            };
          }
        ];
      };
    };
  };
}
```

Then apply the configuration:

```sh
darwin-rebuild switch
```

### Manual Build

If you're not using Nix, you can install yknotify-rs with Cargo:

```sh
cargo install --git https://github.com/reo101/yknotify-rs
```

Or manually clone and build:

```sh
git clone https://github.com/reo101/yknotify-rs
cd yknotify-rs
cargo build --release
```

Then move the binary to a directory in your PATH:

```sh
mv target/release/yknotify-rs /usr/local/bin/
```

## Usage

```
Usage: yknotify-rs [OPTIONS]

Options:
      --request-sound <REQUEST_SOUND>
          Name of the macOS system sound to play when a new touch request is detected.

          Available sounds can be found in `/System/Library/Sounds`, `/Library/Sounds` or
          `~/Library/Sounds`. The sound name must be a filename without an extension, e.g. `Purr`.

          [env: YKNOTIFY_REQUEST_SOUND=]

      --fido2-request-sound <FIDO2_REQUEST_SOUND>
          Name of the macOS system sound to play when a new FIDO2 touch request is detected.

          Overrides the `--request-sound` option, which sets the request sound for all types of
          touch request.

          [env: YKNOTIFY_FIDO2_REQUEST_SOUND=]

      --openpgp-request-sound <OPENPGP_REQUEST_SOUND>
          Name of the macOS system sound to play when a new OpenPGP touch request is detected.

          Overrides the `--request-sound` option, which sets the request sound for all types of
          touch request.

          [env: YKNOTIFY_OPENPGP_REQUEST_SOUND=]

      --dismissed-sound <DISMISSED_SOUND>
          Name of the macOS system sound to play when a touch request is dismissed (for example,
          when the YubiKey is touched).

          Available sounds can be found in `/System/Library/Sounds`, `/Library/Sounds` or
          `~/Library/Sounds`. The sound name must be a filename without an extension, e.g. `Pop`.

          [env: YKNOTIFY_DISMISSED_SOUND=]

      --fido2-dismissed-sound <FIDO2_DISMISSED_SOUND>
          Name of the macOS system sound to play when a FIDO2 touch request is dismissed.

          Overrides the `--dismissed-sound` option, which sets the dismissed sound for all types of
          touch request.

          [env: YKNOTIFY_FIDO2_DISMISSED_SOUND=]

      --openpgp-dismissed-sound <OPENPGP_DISMISSED_SOUND>
          Name of the macOS system sound to play when an OpenPGP touch request is dismissed.

          Overrides the `--dismissed-sound` option, which sets the dismissed sound for all types of
          touch request.

          [env: YKNOTIFY_OPENPGP_DISMISSED_SOUND=]

  -h, --help
          Print help (see a summary with '-h')
```

Example output:

```
2025-07-19T16:57:42.475000Z  INFO yknotify_rs: listening for events
2025-07-19T16:57:53.343860Z  INFO yknotify_rs: dispatching notification for touch event kind=FIDO2 event=start
2025-07-19T16:57:54.949215Z  INFO yknotify_rs: dispatching notification for touch event kind=FIDO2 event=stop
```

## Credits

This project is a Rust rewrite of [yknotify](https://github.com/noperator/yknotify) by [noperator](https://github.com/noperator), which originally implemented this detection strategy in Go.
