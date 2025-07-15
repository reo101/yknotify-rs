use chrono::Utc;
use eyre::Result;
use notify_rust::{set_application, Notification};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LogEntry {
    process_image_path: String,
    sender_image_path: Option<String>,
    subsystem: Option<String>,
    event_message: String,
}

#[derive(Serialize)]
struct TouchEvent {
    ts: String,
    #[serde(rename = "type")]
    event_type: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    set_application("com.apple.keychainaccess")?;

    let mut cmd = Command::new("log");
    cmd.args(["stream", "--level", "debug", "--style", "ndjson"])
        .stdout(Stdio::piped());

    let mut child = cmd.spawn()?;
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout).lines();

    let mut openpgp_notifying = false;

    while let Some(line) = reader.next_line().await? {
        let Ok(entry) = serde_json::from_str::<LogEntry>(&line) else {
            continue;
        };

        if entry.process_image_path.as_str() == "/kernel"
            && entry
                .sender_image_path
                .as_deref()
                .is_some_and(|s| s.ends_with("IOHIDFamily"))
            && entry.event_message.contains("IOHIDLibUserClient:0x")
        {
            println!("{}", entry.event_message);

            if entry.event_message.ends_with("startQueue") {
                let event = TouchEvent {
                    event_type: "FIDO2".to_string(),
                    ts: Utc::now().to_string(),
                };
                println!("{}", serde_json::to_string(&event)?);

                Notification::new()
                    .summary("YubiKey Touch Needed")
                    .body("FIDO2 authentication is required.")
                    .sound_name("Purr")
                    .show()?;
            } else if entry.event_message.ends_with("stopQueue") {
                let event = TouchEvent {
                    event_type: "FIDO2".to_string(),
                    ts: Utc::now().to_string(),
                };
                println!("{}", serde_json::to_string(&event)?);

                Notification::new()
                    .summary("YubiKey Touch Confirmed")
                    .body("YubiKey touch was detected.")
                    .sound_name("Pop")
                    .show()?;
            }
        } else if entry.process_image_path.ends_with("usbsmartcardreaderd")
            && entry
                .subsystem
                .as_deref()
                .is_some_and(|s| s.ends_with("CryptoTokenKit"))
        {
            // This is an OpenPGP message, but we don't know if a notification is
            // needed yet because it might be a repeat.
            let openpgp_needed = entry.event_message == "Time extension received";

            if openpgp_needed && !openpgp_notifying {
                openpgp_notifying = true;

                let event = TouchEvent {
                    event_type: "OpenPGP".to_string(),
                    ts: Utc::now().to_string(),
                };
                println!("{}", serde_json::to_string(&event)?);

                Notification::new()
                    .summary("YubiKey Touch Needed")
                    .body("OpenPGP authentication is required.")
                    .sound_name("Purr")
                    .show()?;
            } else if !openpgp_needed && openpgp_notifying {
                openpgp_notifying = false;

                let event = TouchEvent {
                    event_type: "OpenPGP".to_string(),
                    ts: Utc::now().to_string(),
                };
                println!("{}", serde_json::to_string(&event)?);

                Notification::new()
                    .summary("YubiKey Touch Confirmed")
                    .body("YubiKey touch was detected.")
                    .sound_name("Pop")
                    .show()?;
            }
        }
    }

    child.wait().await?;
    Ok(())
}
