use color_eyre::Report;
use eyre::Result;
use log::error;
use notify_rust::{set_application, Notification};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration, Instant};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LogEntry {
    process_image_path: String,
    sender_image_path: Option<String>,
    subsystem: Option<String>,
    event_message: String,
}

struct TouchState {
    fido2_needed: bool,
    openpgp_needed: bool,
    last_notify: Instant,
}

impl Default for TouchState {
    fn default() -> Self {
        Self {
            fido2_needed: false,
            openpgp_needed: false,
            last_notify: Instant::now(),
        }
    }
}

#[derive(Serialize)]
struct TouchEvent {
    ts: String,
    #[serde(rename = "type")]
    event_type: String,
}

impl TouchState {
    async fn check_and_notify(&mut self) -> Result<()> {
        let now = Instant::now();
        if now.duration_since(self.last_notify) < Duration::from_secs(1) {
            return Ok(());
        }

        let timestamp = chrono::Utc::now().to_rfc3339();
        if self.fido2_needed {
            let event = TouchEvent {
                event_type: "FIDO2".to_string(),
                ts: timestamp.clone(),
            };
            println!("{}", serde_json::to_string(&event)?);

            Notification::new()
                .summary("YubiKey Touch Needed")
                .body("FIDO2 authentication is required.")
                .show()?;
        }
        if self.openpgp_needed {
            let event = TouchEvent {
                event_type: "OpenPGP".to_string(),
                ts: timestamp,
            };
            println!("{}", serde_json::to_string(&event)?);

            Notification::new()
                .summary("YubiKey Touch Needed")
                .body("OpenPGP authentication is required.")
                .show()?;
        }
        self.last_notify = now;

        Ok(())
    }
}

async fn stream_logs() -> Result<()> {
    let mut cmd = Command::new("log");
    cmd.args(["stream", "--level", "debug", "--style", "ndjson"])
        .stdout(Stdio::piped());

    let mut child = cmd.spawn()?;
    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout).lines();

    let state = Arc::new(Mutex::new(TouchState::default()));
    let state_clone = Arc::clone(&state);

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(1)).await;
            let mut state = state_clone.lock().await;
            if let Err(err) = state.check_and_notify().await {
                error!("check_and_notify failed: {:?}", err);
            }
        }
    });

    tokio::pin!(reader);
    while let Some(line) = reader.next_line().await? {
        if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
            let mut state = state.lock().await;
            match entry.process_image_path.as_str() {
                "/kernel"
                    if entry
                        .sender_image_path
                        .as_deref()
                        .map_or(false, |s| s.ends_with("IOHIDFamily")) =>
                {
                    state.fido2_needed = entry.event_message.contains("IOHIDLibUserClient:0x")
                        && entry.event_message.ends_with("startQueue");
                }
                _ if entry.process_image_path.ends_with("usbsmartcardreaderd")
                    && entry
                        .subsystem
                        .as_deref()
                        .map_or(false, |s| s.ends_with("CryptoTokenKit")) =>
                {
                    state.openpgp_needed = entry.event_message == "Time extension received";
                }
                _ => {}
            }
            state.check_and_notify().await?;
        }
    }

    child.wait().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    set_application("com.apple.keychainaccess")?;

    stream_logs().await.map_err(Report::from)?;

    Ok(())
}
