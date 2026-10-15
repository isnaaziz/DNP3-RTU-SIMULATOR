use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use dnp3::outstation::OutstationHandle;
use dnp3::tcp::ServerHandle;
use serde::Serialize;
use tokio::sync::watch;

use crate::model::{Config, Kind, Point};
use crate::outstation::write_point;

const LOG_LIMIT: usize = 300;

#[derive(Clone, Serialize)]
pub struct LogEntry {
    pub time_ms: u64,
    pub ok: bool,
    pub text: String,
}

pub struct Runtime {
    pub _server: ServerHandle,
    pub outstation: OutstationHandle,
}

pub struct App {
    pub config: Config,
    pub path: PathBuf,
    pub runtime: Option<Runtime>,
    pub connected: bool,
    pub log: VecDeque<LogEntry>,
    pub error: Option<String>,
    pub locked_port: Option<u16>,
    pub changes: watch::Sender<u64>,
}

#[derive(Clone)]
pub struct Shared(Arc<Mutex<App>>);

impl Shared {
    pub fn new(app: App) -> Self {
        Self(Arc::new(Mutex::new(app)))
    }

    pub fn lock(&self) -> MutexGuard<'_, App> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

impl App {
    pub fn touch(&self) {
        self.changes.send_modify(|version| *version += 1);
    }

    pub fn log(&mut self, ok: bool, text: impl Into<String>) {
        let text = text.into();
        tracing::info!("{text}");
        self.log.push_front(LogEntry {
            time_ms: now_ms(),
            ok,
            text,
        });
        self.log.truncate(LOG_LIMIT);
        self.touch();
    }

    pub fn persist(&mut self) {
        if let Err(err) = self.config.save(&self.path) {
            self.error = Some(format!("gagal menyimpan konfigurasi: {err}"));
        }
        self.touch();
    }

    pub fn write(&self, points: &[Point]) {
        if points.is_empty() {
            return;
        }
        self.touch();
        if let Some(rt) = &self.runtime {
            rt.outstation.transaction(|db| {
                for point in points {
                    write_point(db, point);
                }
            });
        }
    }

    pub fn set_value(&mut self, kind: Kind, index: u16, value: f64) -> Option<Point> {
        let point = self.config.find_mut(kind, index)?;
        point.data.value = value;
        let point = point.clone();
        self.write(std::slice::from_ref(&point));
        Some(point)
    }
}
