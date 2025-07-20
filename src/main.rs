use clap::Parser;
use eyre::Result;
use notify_rust::{set_application, Notification};
use serde::Deserialize;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};
use tracing::{debug, info, level_filters::LevelFilter};
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _, EnvFilter};

use std::process::Stdio;

#[derive(Parser, Debug)]
struct Args {
    /// Name of the macOS system sound to play when a new touch request is detected.
    ///
    /// Available sounds can be found in `/System/Library/Sounds`, `/Library/Sounds` or
    /// `~/Library/Sounds`. The sound name must be a filename without an extension, e.g. `Purr`.
    #[arg(long, env = "YKNOTIFY_REQUEST_SOUND")]
    request_sound: Option<String>,

    /// Name of the macOS system sound to play when a new FIDO2 touch request is detected.
    ///
    /// Overrides the `--request-sound` option, which sets the request sound for all types of touch
    /// request.
    #[arg(
        long,
        conflicts_with = "request_sound",
        env = "YKNOTIFY_FIDO2_REQUEST_SOUND"
    )]
    fido2_request_sound: Option<String>,

    /// Name of the macOS system sound to play when a new OpenPGP touch request is detected.
    ///
    /// Overrides the `--request-sound` option, which sets the request sound for all types of touch
    /// request.
    #[arg(
        long,
        conflicts_with = "request_sound",
        env = "YKNOTIFY_OPENPGP_REQUEST_SOUND"
    )]
    openpgp_request_sound: Option<String>,

    /// Name of the macOS system sound to play when a touch request is dismissed (for example, when
    /// the YubiKey is touched).
    ///
    /// Available sounds can be found in `/System/Library/Sounds`, `/Library/Sounds` or
    /// `~/Library/Sounds`. The sound name must be a filename without an extension, e.g. `Pop`.
    #[arg(long, env = "YKNOTIFY_DISMISSED_SOUND")]
    dismissed_sound: Option<String>,

    /// Name of the macOS system sound to play when a FIDO2 touch request is dismissed.
    ///
    /// Overrides the `--dismissed-sound` option, which sets the dismissed sound for all types of
    /// touch request.
    #[arg(
        long,
        conflicts_with = "request_sound",
        env = "YKNOTIFY_FIDO2_DISMISSED_SOUND"
    )]
    fido2_dismissed_sound: Option<String>,

    /// Name of the macOS system sound to play when an OpenPGP touch request is dismissed.
    ///
    /// Overrides the `--dismissed-sound` option, which sets the dismissed sound for all types of
    /// touch request.
    #[arg(
        long,
        conflicts_with = "request_sound",
        env = "YKNOTIFY_OPENPGP_DISMISSED_SOUND"
    )]
    openpgp_dismissed_sound: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LogEntry {
    process_image_path: String,
    sender_image_path: Option<String>,
    subsystem: Option<String>,
    event_message: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    set_application("com.apple.keychainaccess")?;

    let mut cmd = Command::new("log");
    cmd.args(["stream", "--level", "debug", "--style", "ndjson"])
        .stdout(Stdio::piped());

    let mut child = cmd.spawn()?;
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout).lines();

    info!("listening for events");

    let mut openpgp_notifying = false;

    while let Some(line) = reader.next_line().await? {
        let Ok(entry) = serde_json::from_str::<LogEntry>(&line) else {
            debug!(event = ?line, "failed to parse event");
            continue;
        };

        if entry.process_image_path.as_str() == "/kernel"
            && entry
                .sender_image_path
                .as_deref()
                .is_some_and(|s| s.ends_with("IOHIDFamily"))
            && entry.event_message.contains("IOHIDLibUserClient:0x")
        {
            debug!(kind = %"fido2", event_message = %entry.event_message, "received event");

            if entry.event_message.ends_with("startQueue") {
                info!(kind = %"FIDO2", event = %"start", "dispatching notification for touch event");

                let mut notification = Notification::new();

                notification
                    .summary("YubiKey Touch Needed")
                    .body("FIDO2 authentication is required.");

                if let Some(sound) = args
                    .fido2_request_sound
                    .as_ref()
                    .or(args.request_sound.as_ref())
                {
                    notification.sound_name(sound);
                }

                notification.show()?;
            } else if entry.event_message.ends_with("stopQueue") {
                info!(kind = %"FIDO2", event = %"stop", "dispatching notification for touch event");

                let mut notification = Notification::new();

                notification
                    .summary("YubiKey Touch Confirmed")
                    .body("YubiKey touch was detected.");

                if let Some(sound) = args
                    .fido2_dismissed_sound
                    .as_ref()
                    .or(args.dismissed_sound.as_ref())
                {
                    notification.sound_name(sound);
                }

                notification.show()?;
            }
        } else if entry.process_image_path.ends_with("usbsmartcardreaderd")
            && entry
                .subsystem
                .as_deref()
                .is_some_and(|s| s.ends_with("CryptoTokenKit"))
        {
            debug!(kind = %"OpenPGP", event_message = %entry.event_message, "received event");

            // This is an OpenPGP message, but we don't know if a notification is
            // needed yet because it might be a repeat.
            let openpgp_needed = entry.event_message == "Time extension received";

            if openpgp_needed && !openpgp_notifying {
                // We received an event that indicates that an OpenPGP touch is needed, plus the
                // most recent one we saw was not *also* of the same type (we're not in a
                // "notifying" state already).
                openpgp_notifying = true;

                info!(kind = %"OpenPGP", event = %"start", "dispatching notification for touch event");

                let mut notification = Notification::new();

                notification
                    .summary("YubiKey Touch Needed")
                    .body("OpenPGP authentication is required.");

                if let Some(sound) = args
                    .openpgp_request_sound
                    .as_ref()
                    .or(args.request_sound.as_ref())
                {
                    notification.sound_name(sound);
                }

                notification.show()?;
            } else if !openpgp_needed && openpgp_notifying {
                // We received a closing event (one that indicates an OpenPGP touch is no longer
                // needed), and we *are* in a "notifying" state.
                openpgp_notifying = false;

                info!(kind = %"OpenPGP", event = %"stop", "dispatching notification for touch event");

                let mut notification = Notification::new();

                notification
                    .summary("YubiKey Touch Confirmed")
                    .body("YubiKey touch was detected.");

                if let Some(sound) = args
                    .openpgp_dismissed_sound
                    .as_ref()
                    .or(args.dismissed_sound.as_ref())
                {
                    notification.sound_name(sound);
                }

                notification.show()?;
            }
        }
    }

    child.wait().await?;
    Ok(())
}
