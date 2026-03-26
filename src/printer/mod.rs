mod status;
mod tls;

use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS, TlsConfiguration, Transport};
use rustls::ClientConfig;
use serde_json::{Value, json};
pub use status::PrinterStatus;
use std::sync::{
  Arc,
  atomic::{AtomicU64, Ordering},
};
use tokio::sync::{RwLock, mpsc};

pub struct PrinterConfig {
  pub ip: String,
  pub serial: String,
  pub access_code: String,
}

pub enum PrinterEvent {
  StatusUpdate(PrinterStatus),
  Disconnected,
}

#[repr(u8)]
pub enum PrinterSpeed {
  Silent = 1,    //    50%
  Normal = 2,    //   100%
  Sport = 3,     //    125%
  Ludicrous = 4, // 160%
}

#[derive(Clone)]
pub struct PrinterClient {
  state: Arc<RwLock<Option<PrinterStatus>>>,
  mqtt: AsyncClient,
  serial: String,
  seq: Arc<AtomicU64>,
}

impl PrinterClient {
  pub async fn connect(
    cfg: PrinterConfig,
    event_tx: mpsc::Sender<PrinterEvent>,
  ) -> anyhow::Result<Self> {
    let tls = ClientConfig::builder()
      .dangerous()
      .with_custom_certificate_verifier(Arc::new(tls::AcceptAnyCert))
      .with_no_client_auth();

    let mut opts = MqttOptions::new("rhizome-rs", &cfg.ip, 8883);
    opts
      .set_credentials("bblp", &cfg.access_code)
      .set_keep_alive(std::time::Duration::from_secs(30))
      .set_transport(Transport::tls_with_config(TlsConfiguration::Rustls(
        Arc::new(tls),
      )));

    let (mqtt, mut eventloop) = AsyncClient::new(opts, 64);

    mqtt
      .subscribe(format!("device/{}/report", cfg.serial), QoS::AtMostOnce)
      .await?;

    mqtt
      .publish(
        format!("device/{}/request", cfg.serial),
        QoS::AtMostOnce,
        false,
        json!({
            "pushing": {
                "sequence_id": "0",
                "command": "pushall",
                "version": 1,
                "push_target": 1
            }
        })
        .to_string(),
      )
      .await?;

    let state: Arc<RwLock<Option<PrinterStatus>>> = Arc::new(RwLock::new(None));
    let bg_state = Arc::clone(&state);

    tokio::spawn(async move {
      loop {
        match eventloop.poll().await {
          Ok(Event::Incoming(Packet::Publish(msg))) => {
            let Ok(json) = serde_json::from_slice::<Value>(&msg.payload) else {
              continue;
            };
            if let Some(print) = json.get("print") {
              let is_status = print.get("command").and_then(Value::as_str) == Some("push_status");

              if is_status {
                let mut lock = bg_state.write().await;
                let status = lock.get_or_insert_with(PrinterStatus::default);
                status.merge_update(print);
                let snapshot = status.clone();
                drop(lock);

                if event_tx
                  .send(PrinterEvent::StatusUpdate(snapshot))
                  .await
                  .is_err()
                {
                  break;
                }
              }
            }
          }
          Ok(_) => {} // pings, acks, subacks — nothing to do
          Err(e) => {
            eprintln!("[mqtt] error: {e}");
            *bg_state.write().await = None;
            let _ = event_tx.send(PrinterEvent::Disconnected).await;
            break;
          }
        }
      }
    });

    Ok(Self {
      state,
      mqtt,
      serial: cfg.serial,
      seq: Arc::new(AtomicU64::new(1)),
    })
  }

  pub async fn status(&self) -> Option<PrinterStatus> {
    self.state.read().await.clone()
  }

  pub async fn is_connected(&self) -> bool {
    self.state.read().await.is_some()
  }

  pub async fn send_gcode(&self, gcode: &str) -> anyhow::Result<()> {
    let seq = self.seq.fetch_add(1, Ordering::Relaxed);
    self
      .publish(json!({
          "print": {
              "sequence_id": seq.to_string(),
              "command": "gcode_line",
              "param": gcode
          }
      }))
      .await
  }

  pub async fn set_speed(&self, level: PrinterSpeed) -> anyhow::Result<()> {
    let seq = self.seq.fetch_add(1, Ordering::Relaxed);
    self
      .publish(json!({
          "print": {
              "sequence_id": seq.to_string(),
              "command": "print_speed",
              "param": (level as u8).to_string()
          }
      }))
      .await
  }

  pub async fn send_command(&self, payload: serde_json::Value) -> anyhow::Result<()> {
    self.publish(payload).await
  }

  pub async fn start_print(&self, path: &str) -> anyhow::Result<()> {
    let seq = self.seq.fetch_add(1, Ordering::Relaxed);
    self
      .publish(json!({
          "print": {
            "sequence_id": seq.to_string(),
            "command": "project_file",
            "param": "Metadata/plate_1.gcode",
            "url": format!("file://{}", path),
            "project_id": 0,
            "profile_id": "0",
            "task_id": "0",
            "subtask_id": "0",
            "subtask_name": "",
            "file": "",
            "md5": "",
            "timelapse": false,
            "bed_type": "auto",
            "bed_levelling": true,
            "flow_cali": true,
            "vibration_cali": true,
            "layer_inspect": false,
            "use_ams": false
          }
      }))
      .await
  }

  pub async fn pause_print(&self) -> anyhow::Result<()> {
    let seq = self.seq.fetch_add(1, Ordering::Relaxed);
    self
      .publish(json!({
          "print": {
            "sequence_id": seq.to_string(),
            "command": "pause",
            "param": ""
          }
      }))
      .await
  }

  pub async fn resume_print(&self) -> anyhow::Result<()> {
    let seq = self.seq.fetch_add(1, Ordering::Relaxed);
    self
      .publish(json!({
          "print": {
            "sequence_id": seq.to_string(),
            "command": "resume",
            "param": ""
          }
      }))
      .await
  }

  pub async fn stop_print(&self) -> anyhow::Result<()> {
    let seq = self.seq.fetch_add(1, Ordering::Relaxed);
    self
      .publish(json!({
          "print": {
            "sequence_id": seq.to_string(),
            "command": "stop",
            "param": ""
          }
      }))
      .await
  }

  async fn publish(&self, payload: serde_json::Value) -> anyhow::Result<()> {
    self
      .mqtt
      .publish(
        format!("device/{}/request", self.serial),
        QoS::AtMostOnce,
        false,
        payload.to_string(),
      )
      .await?;
    Ok(())
  }
}
