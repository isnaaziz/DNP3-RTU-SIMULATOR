use std::path::Path;

use serde::{Deserialize, Serialize};

pub const FLAG_ONLINE: u8 = 0x01;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    BinaryInput,
    DoubleBit,
    BinaryOutput,
    Counter,
    AnalogInput,
    AnalogOutput,
}

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Kind::BinaryInput => "BI",
            Kind::DoubleBit => "DBI",
            Kind::BinaryOutput => "BO",
            Kind::Counter => "Counter",
            Kind::AnalogInput => "AI",
            Kind::AnalogOutput => "AO",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimMode {
    Random,
    Sine,
    Ramp,
    Toggle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Simulation {
    pub mode: SimMode,
    pub min: f64,
    pub max: f64,
    pub period_s: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Feedback {
    pub kind: Kind,
    pub index: u16,
    pub travel_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalRemote {
    pub kind: Kind,
    pub index: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PointInput {
    pub kind: Kind,
    pub index: u16,
    pub name: String,
    #[serde(default)]
    pub value: f64,
    #[serde(default = "default_flags")]
    pub flags: u8,
    #[serde(default = "default_class")]
    pub class: u8,
    #[serde(default)]
    pub sim: Option<Simulation>,
    #[serde(default)]
    pub feedback: Option<Feedback>,
    #[serde(default)]
    pub local: bool,
    #[serde(default)]
    pub lr: Option<LocalRemote>,
    #[serde(default)]
    pub close_index: Option<u16>,
    #[serde(default)]
    pub unit: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Point {
    pub id: u64,
    #[serde(flatten)]
    pub data: PointInput,
}

impl Point {
    pub fn key(&self) -> (Kind, u16) {
        (self.data.kind, self.data.index)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub bind_ip: String,
    pub port: u16,
    pub outstation_address: u16,
    pub master_address: u16,
    pub event_buffer: u16,
    pub unsolicited: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub settings: Settings,
    pub points: Vec<Point>,
    pub next_id: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Export {
    pub settings: Settings,
    pub points: Vec<PointInput>,
}

impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if self.bind_ip.trim().is_empty() {
            return Err("IP bind wajib diisi".into());
        }
        if self.port == 0 {
            return Err("port harus > 0".into());
        }
        if self.outstation_address == self.master_address {
            return Err("alamat outstation dan master tidak boleh sama".into());
        }
        if self.event_buffer == 0 {
            return Err("event buffer harus > 0".into());
        }
        Ok(())
    }
}

fn default_flags() -> u8 {
    FLAG_ONLINE
}

fn default_class() -> u8 {
    1
}

impl PointInput {
    pub fn validate(&mut self) -> Result<(), String> {
        self.name = self.name.trim().to_string();
        self.unit = self.unit.trim().to_string();
        if self.class > 3 {
            return Err("class harus 0-3".into());
        }
        self.value = normalize(self.kind, self.value)?;
        if let Some(fb) = &self.feedback {
            let allowed = match self.kind {
                Kind::BinaryOutput => matches!(fb.kind, Kind::BinaryInput | Kind::DoubleBit),
                Kind::AnalogOutput => matches!(fb.kind, Kind::AnalogInput),
                _ => false,
            };
            if !allowed {
                return Err(format!(
                    "feedback {} tidak valid untuk {}",
                    fb.kind.label(),
                    self.kind.label()
                ));
            }
        }
        if let Some(close) = self.close_index {
            if self.kind != Kind::BinaryOutput {
                return Err("index kontrol Close hanya untuk BO".into());
            }
            if close == self.index {
                return Err("index kontrol Close harus berbeda dari index Open".into());
            }
        }
        if let Some(lr) = &self.lr {
            if !matches!(self.kind, Kind::BinaryOutput | Kind::AnalogOutput) {
                return Err("status L/R hanya untuk BO atau AO".into());
            }
            if !matches!(lr.kind, Kind::BinaryInput | Kind::DoubleBit) {
                return Err("status L/R harus BI atau DBI".into());
            }
        }
        if let Some(sim) = &self.sim {
            if sim.max < sim.min {
                return Err("simulasi: max harus >= min".into());
            }
            if sim.period_s <= 0.0 {
                return Err("simulasi: periode harus > 0".into());
            }
        }
        Ok(())
    }
}

fn claimed(input: &PointInput) -> Vec<u16> {
    input.close_index.into_iter().chain([input.index]).collect()
}

pub fn conflict(points: &[Point], input: &PointInput, except: Option<u64>) -> Option<String> {
    let mine = claimed(input);
    points
        .iter()
        .filter(|p| Some(p.id) != except && p.data.kind == input.kind)
        .find_map(|p| {
            claimed(&p.data)
                .into_iter()
                .find(|i| mine.contains(i))
                .map(|i| {
                    format!(
                        "{} index {i} sudah dipakai oleh \"{}\"",
                        input.kind.label(),
                        p.data.name
                    )
                })
        })
}

pub fn normalize(kind: Kind, value: f64) -> Result<f64, String> {
    if !value.is_finite() {
        return Err("nilai tidak valid".into());
    }
    match kind {
        Kind::BinaryInput | Kind::BinaryOutput => Ok(if value != 0.0 { 1.0 } else { 0.0 }),
        Kind::DoubleBit if (0.0..=3.0).contains(&value) && value.fract() == 0.0 => Ok(value),
        Kind::DoubleBit => {
            Err("double bit harus 0 (intermediate), 1 (off), 2 (on), atau 3 (bad)".into())
        }
        Kind::Counter if value >= 0.0 && value <= u32::MAX as f64 => Ok(value.floor()),
        Kind::Counter => Err("counter harus 0..4294967295".into()),
        Kind::AnalogInput | Kind::AnalogOutput => Ok(value),
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::sample());
        }
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let text = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, text).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, path).map_err(|e| e.to_string())
    }

    pub fn export(&self) -> Export {
        let mut points: Vec<PointInput> = self.points.iter().map(|p| p.data.clone()).collect();
        points.sort_by_key(|p| (p.kind as u8, p.index));
        Export {
            settings: self.settings.clone(),
            points,
        }
    }

    pub fn import(export: Export) -> Result<Self, String> {
        export.settings.validate()?;
        let mut points: Vec<Point> = Vec::with_capacity(export.points.len());
        for (position, mut data) in export.points.into_iter().enumerate() {
            let label = format!(
                "point #{} ({} {})",
                position + 1,
                data.kind.label(),
                data.index
            );
            data.validate().map_err(|e| format!("{label}: {e}"))?;
            if let Some(conflict) = conflict(&points, &data, None) {
                return Err(format!("{label}: {conflict}"));
            }
            points.push(Point {
                id: position as u64 + 1,
                data,
            });
        }
        Ok(Self {
            settings: export.settings,
            next_id: points.len() as u64 + 1,
            points,
        })
    }

    pub fn find(&self, kind: Kind, index: u16) -> Option<&Point> {
        self.points.iter().find(|p| p.key() == (kind, index))
    }

    pub fn conflict(&self, input: &PointInput, except: Option<u64>) -> Option<String> {
        conflict(&self.points, input, except)
    }

    pub fn control_target(&self, index: u16) -> Option<(&Point, Option<bool>)> {
        self.points.iter().find_map(|p| {
            if p.data.kind != Kind::BinaryOutput {
                return None;
            }
            match p.data.close_index {
                Some(close) if close == index => Some((p, Some(true))),
                Some(_) if p.data.index == index => Some((p, Some(false))),
                None if p.data.index == index => Some((p, None)),
                _ => None,
            }
        })
    }

    pub fn is_local(&self, point: &Point) -> bool {
        match point
            .data
            .lr
            .as_ref()
            .and_then(|lr| self.find(lr.kind, lr.index))
        {
            Some(status) if status.data.kind == Kind::DoubleBit => status.data.value == 2.0,
            Some(status) => status.data.value != 0.0,
            None => point.data.local,
        }
    }

    pub fn find_mut(&mut self, kind: Kind, index: u16) -> Option<&mut Point> {
        self.points.iter_mut().find(|p| p.key() == (kind, index))
    }

    fn sample() -> Self {
        let point = |kind, index, name: &str, value| PointInput {
            kind,
            index,
            name: name.into(),
            value,
            flags: FLAG_ONLINE,
            class: 1,
            sim: None,
            feedback: None,
            local: false,
            lr: None,
            close_index: None,
            unit: String::new(),
        };
        let sim = |mode, min, max, period_s| {
            Some(Simulation {
                mode,
                min,
                max,
                period_s,
            })
        };
        let feedback = |kind, index, travel_ms| {
            Some(Feedback {
                kind,
                index,
                travel_ms,
            })
        };

        let unit = |unit: &str, data: PointInput| PointInput {
            unit: unit.into(),
            ..data
        };

        let inputs = vec![
            point(Kind::DoubleBit, 0, "CB Incoming", 2.0),
            PointInput {
                feedback: feedback(Kind::DoubleBit, 0, 2000),
                lr: Some(LocalRemote {
                    kind: Kind::BinaryInput,
                    index: 12,
                }),
                ..point(Kind::BinaryOutput, 0, "CB Incoming", 1.0)
            },
            point(Kind::DoubleBit, 1, "LBS Feeder 1", 1.0),
            PointInput {
                feedback: feedback(Kind::DoubleBit, 1, 3000),
                ..point(Kind::BinaryOutput, 1, "LBS Feeder 1", 0.0)
            },
            point(Kind::BinaryInput, 0, "LBS Feeder 2", 0.0),
            PointInput {
                feedback: feedback(Kind::BinaryInput, 0, 1000),
                ..point(Kind::BinaryOutput, 2, "LBS Feeder 2", 0.0)
            },
            point(Kind::BinaryInput, 10, "OCR", 0.0),
            point(Kind::BinaryInput, 11, "GFR", 0.0),
            point(Kind::BinaryInput, 12, "CB Incoming L/R", 0.0),
            unit(
                "kV",
                PointInput {
                    sim: sim(SimMode::Sine, 19.5, 20.5, 60.0),
                    ..point(Kind::AnalogInput, 0, "Tegangan", 20.0)
                },
            ),
            unit(
                "A",
                PointInput {
                    sim: sim(SimMode::Random, 50.0, 150.0, 1.0),
                    ..point(Kind::AnalogInput, 1, "Arus Phase R", 100.0)
                },
            ),
            unit(
                "Hz",
                PointInput {
                    sim: sim(SimMode::Random, 49.8, 50.2, 1.0),
                    ..point(Kind::AnalogInput, 2, "Frekuensi", 50.0)
                },
            ),
            unit("MW", point(Kind::AnalogInput, 3, "Setpoint Feedback", 0.0)),
            unit(
                "kWh",
                PointInput {
                    sim: sim(SimMode::Random, 1.0, 5.0, 1.0),
                    ..point(Kind::Counter, 0, "kWh Export", 1000.0)
                },
            ),
            unit(
                "MW",
                PointInput {
                    feedback: feedback(Kind::AnalogInput, 3, 500),
                    ..point(Kind::AnalogOutput, 0, "Setpoint Daya", 0.0)
                },
            ),
        ];

        let points: Vec<Point> = inputs
            .into_iter()
            .enumerate()
            .map(|(i, data)| Point {
                id: i as u64 + 1,
                data,
            })
            .collect();

        Self {
            settings: Settings {
                bind_ip: "0.0.0.0".into(),
                port: 20000,
                outstation_address: 10,
                master_address: 1,
                event_buffer: 200,
                unsolicited: true,
            },
            next_id: points.len() as u64 + 1,
            points,
        }
    }
}
