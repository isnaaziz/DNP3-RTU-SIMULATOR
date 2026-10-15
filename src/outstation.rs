use std::net::SocketAddr;
use std::time::Duration;

use dnp3::app::control::*;
use dnp3::app::measurement::*;
use dnp3::app::{Listener, MaybeAsync, Timestamp};
use dnp3::link::{EndpointAddress, LinkErrorMode};
use dnp3::outstation::database::*;
use dnp3::outstation::*;
use dnp3::tcp::{AddressFilter, Server};

use crate::model::{Kind, Point};
use crate::state::{Runtime, Shared, now_ms};

pub async fn start(shared: &Shared) -> Result<(), String> {
    let (settings, points) = {
        let mut app = shared.lock();
        app.runtime = None;
        app.connected = false;
        (app.config.settings.clone(), app.config.points.clone())
    };

    let address: SocketAddr = format!("{}:{}", settings.bind_ip, settings.port)
        .parse()
        .map_err(|e| {
            format!(
                "alamat {}:{} tidak valid: {e}",
                settings.bind_ip, settings.port
            )
        })?;
    let outstation_address = EndpointAddress::try_new(settings.outstation_address)
        .map_err(|e| format!("outstation address: {e:?}"))?;
    let master_address = EndpointAddress::try_new(settings.master_address)
        .map_err(|e| format!("master address: {e:?}"))?;

    let mut config = OutstationConfig::new(
        outstation_address,
        master_address,
        EventBufferConfig::all_types(settings.event_buffer),
    );
    config.features.unsolicited = if settings.unsolicited {
        Feature::Enabled
    } else {
        Feature::Disabled
    };

    let mut server = Server::new_tcp_server(LinkErrorMode::Close, address);
    let outstation = server
        .add_outstation(
            config,
            Box::new(Application {
                shared: shared.clone(),
            }),
            Box::new(Information),
            Box::new(Controls {
                shared: shared.clone(),
            }),
            Box::new(Connection {
                shared: shared.clone(),
            }),
            AddressFilter::Any,
        )
        .map_err(|e| format!("gagal membuat outstation: {e:?}"))?;

    outstation.transaction(|db| {
        for point in &points {
            add_point(db, point);
            write_point(db, point);
        }
    });

    let handle = server
        .bind()
        .await
        .map_err(|e| format!("gagal bind {address}: {e}"))?;
    let mut app = shared.lock();
    app.runtime = Some(Runtime {
        _server: handle,
        outstation,
    });
    app.error = None;
    app.log(
        true,
        format!(
            "Outstation aktif di {address} (outstation {}, master {})",
            settings.outstation_address, settings.master_address
        ),
    );
    Ok(())
}

pub async fn restart(shared: &Shared) -> Result<(), String> {
    shared.lock().runtime = None;
    tokio::time::sleep(Duration::from_millis(300)).await;
    start(shared).await
}

fn class(value: u8) -> Option<EventClass> {
    match value {
        1 => Some(EventClass::Class1),
        2 => Some(EventClass::Class2),
        3 => Some(EventClass::Class3),
        _ => None,
    }
}

pub fn add_point(db: &mut Database, point: &Point) {
    let (index, class) = (point.data.index, class(point.data.class));
    match point.data.kind {
        Kind::BinaryInput => {
            db.add(
                index,
                class,
                BinaryInputConfig::new(
                    StaticBinaryInputVariation::Group1Var2,
                    EventBinaryInputVariation::Group2Var2,
                ),
            );
        }
        Kind::DoubleBit => {
            db.add(
                index,
                class,
                DoubleBitBinaryInputConfig::new(
                    StaticDoubleBitBinaryInputVariation::Group3Var2,
                    EventDoubleBitBinaryInputVariation::Group4Var2,
                ),
            );
        }
        Kind::BinaryOutput => {
            db.add(
                index,
                class,
                BinaryOutputStatusConfig::new(
                    StaticBinaryOutputStatusVariation::Group10Var2,
                    EventBinaryOutputStatusVariation::Group11Var2,
                ),
            );
        }
        Kind::Counter => {
            db.add(
                index,
                class,
                CounterConfig::new(
                    StaticCounterVariation::Group20Var1,
                    EventCounterVariation::Group22Var5,
                    0,
                ),
            );
        }
        Kind::AnalogInput => {
            db.add(
                index,
                class,
                AnalogInputConfig::new(
                    StaticAnalogInputVariation::Group30Var5,
                    EventAnalogInputVariation::Group32Var7,
                    0.0,
                ),
            );
        }
        Kind::AnalogOutput => {
            db.add(
                index,
                class,
                AnalogOutputStatusConfig::new(
                    StaticAnalogOutputStatusVariation::Group40Var3,
                    EventAnalogOutputStatusVariation::Group42Var7,
                    0.0,
                ),
            );
        }
    }
}

pub fn remove_point(db: &mut Database, kind: Kind, index: u16) {
    match kind {
        Kind::BinaryInput => Remove::<BinaryInput>::remove(db, index),
        Kind::DoubleBit => Remove::<DoubleBitBinaryInput>::remove(db, index),
        Kind::BinaryOutput => Remove::<BinaryOutputStatus>::remove(db, index),
        Kind::Counter => Remove::<Counter>::remove(db, index),
        Kind::AnalogInput => Remove::<AnalogInput>::remove(db, index),
        Kind::AnalogOutput => Remove::<AnalogOutputStatus>::remove(db, index),
    };
}

pub fn write_point(db: &mut Database, point: &Point) {
    let (index, value) = (point.data.index, point.data.value);
    let flags = Flags::new(point.data.flags);
    let time = Time::synchronized(now_ms());
    let options = UpdateOptions::detect_event();
    match point.data.kind {
        Kind::BinaryInput => {
            db.update(index, &BinaryInput::new(value != 0.0, flags, time), options)
        }
        Kind::DoubleBit => db.update(
            index,
            &DoubleBitBinaryInput::new(double_bit(value), flags, time),
            options,
        ),
        Kind::BinaryOutput => db.update(
            index,
            &BinaryOutputStatus::new(value != 0.0, flags, time),
            options,
        ),
        Kind::Counter => db.update(index, &Counter::new(value as u32, flags, time), options),
        Kind::AnalogInput => db.update(index, &AnalogInput::new(value, flags, time), options),
        Kind::AnalogOutput => {
            db.update(index, &AnalogOutputStatus::new(value, flags, time), options)
        }
    };
}

fn double_bit(value: f64) -> DoubleBit {
    match value as u8 {
        0 => DoubleBit::Intermediate,
        1 => DoubleBit::DeterminedOff,
        2 => DoubleBit::DeterminedOn,
        _ => DoubleBit::Indeterminate,
    }
}

struct Application {
    shared: Shared,
}

impl OutstationApplication for Application {
    fn write_absolute_time(&mut self, time: Timestamp) -> Result<(), RequestError> {
        self.shared.lock().log(
            true,
            format!("Time sync dari master: {} ms", time.raw_value()),
        );
        Ok(())
    }
}

struct Information;

impl OutstationInformation for Information {}

struct Connection {
    shared: Shared,
}

impl Listener<ConnectionState> for Connection {
    fn update(&mut self, value: ConnectionState) -> MaybeAsync<()> {
        let connected = value == ConnectionState::Connected;
        let mut app = self.shared.lock();
        app.connected = connected;
        app.log(
            connected,
            if connected {
                "Master terhubung"
            } else {
                "Master terputus"
            },
        );
        MaybeAsync::ready(())
    }
}

struct Controls {
    shared: Shared,
}

impl ControlHandler for Controls {}

impl Controls {
    fn target(&self, kind: Kind, index: u16) -> Result<(u16, Option<bool>), CommandStatus> {
        let app = self.shared.lock();
        let (point, role) = match kind {
            Kind::BinaryOutput => app.config.control_target(index),
            _ => app.config.find(kind, index).map(|p| (p, None)),
        }
        .ok_or(CommandStatus::NotSupported)?;
        if app.config.is_local(point) {
            return Err(CommandStatus::Local);
        }
        if point.data.flags & crate::model::FLAG_ONLINE == 0 {
            return Err(CommandStatus::HardwareError);
        }
        Ok((point.data.index, role))
    }

    fn check(&self, kind: Kind, index: u16) -> CommandStatus {
        self.target(kind, index)
            .map(|_| CommandStatus::Success)
            .unwrap_or_else(|status| status)
    }

    fn report(&self, action: &str, kind: Kind, index: u16, detail: &str, status: CommandStatus) {
        self.shared.lock().log(
            status == CommandStatus::Success,
            format!("{action} {}[{index}] {detail} → {status:?}", kind.label()),
        );
    }

    fn apply_output(&self, kind: Kind, index: u16, value: f64, revert_after: Option<Duration>) {
        let feedback = {
            let mut app = self.shared.lock();
            let previous = app.config.find(kind, index).map(|p| p.data.value);
            app.set_value(kind, index, value);
            app.persist();
            let feedback = app
                .config
                .find(kind, index)
                .and_then(|p| p.data.feedback.clone());
            if let (Some(delay), Some(previous)) = (revert_after, previous) {
                let shared = self.shared.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(delay).await;
                    let mut app = shared.lock();
                    app.set_value(kind, index, previous);
                    app.persist();
                });
            }
            feedback
        };

        let Some(feedback) = feedback else { return };
        let shared = self.shared.clone();
        let target = match feedback.kind {
            Kind::DoubleBit => {
                if value != 0.0 {
                    2.0
                } else {
                    1.0
                }
            }
            Kind::BinaryInput => {
                if value != 0.0 {
                    1.0
                } else {
                    0.0
                }
            }
            _ => value,
        };
        if feedback.kind == Kind::DoubleBit && feedback.travel_ms > 0 {
            shared
                .lock()
                .set_value(Kind::DoubleBit, feedback.index, 0.0);
        }
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(feedback.travel_ms)).await;
            let mut app = shared.lock();
            if app
                .set_value(feedback.kind, feedback.index, target)
                .is_some()
            {
                app.persist();
                app.log(
                    true,
                    format!(
                        "Feedback {}[{}] = {target}",
                        feedback.kind.label(),
                        feedback.index
                    ),
                );
            } else {
                app.log(
                    false,
                    format!(
                        "Feedback {}[{}] tidak ditemukan",
                        feedback.kind.label(),
                        feedback.index
                    ),
                );
            }
        });
    }
}

fn crob_state(code: ControlCode) -> Option<bool> {
    match (code.tcc, code.op_type) {
        (TripCloseCode::Close, _) => Some(true),
        (TripCloseCode::Trip, _) => Some(false),
        (TripCloseCode::Nul, OpType::LatchOn | OpType::PulseOn) => Some(true),
        (TripCloseCode::Nul, OpType::LatchOff | OpType::PulseOff) => Some(false),
        _ => None,
    }
}

fn describe(control: &Group12Var1) -> String {
    format!(
        "{:?}/{:?} count={} on={}ms off={}ms",
        control.code.tcc, control.code.op_type, control.count, control.on_time, control.off_time
    )
}

impl ControlSupport<Group12Var1> for Controls {
    fn select(
        &mut self,
        control: Group12Var1,
        index: u16,
        _db: &mut DatabaseHandle,
    ) -> CommandStatus {
        let status = match crob_state(control.code) {
            None => CommandStatus::NotSupported,
            Some(_) => self.check(Kind::BinaryOutput, index),
        };
        self.report(
            "SELECT",
            Kind::BinaryOutput,
            index,
            &describe(&control),
            status,
        );
        status
    }

    fn operate(
        &mut self,
        control: Group12Var1,
        index: u16,
        op_type: OperateType,
        _db: &mut DatabaseHandle,
    ) -> CommandStatus {
        let result = match crob_state(control.code) {
            None => Err(CommandStatus::NotSupported),
            Some(code_state) => self
                .target(Kind::BinaryOutput, index)
                .map(|(main, role)| (main, role.unwrap_or(code_state))),
        };
        let status = result.map(|_| CommandStatus::Success).unwrap_or_else(|s| s);
        let action = match result {
            Ok((_, true)) => " → CLOSE",
            Ok((_, false)) => " → OPEN",
            Err(_) => "",
        };
        self.report(
            &format!("OPERATE ({op_type:?})"),
            Kind::BinaryOutput,
            index,
            &format!("{}{action}", describe(&control)),
            status,
        );
        if let Ok((main, state)) = result {
            let pulse = matches!(control.code.op_type, OpType::PulseOn | OpType::PulseOff);
            let revert = pulse.then(|| Duration::from_millis(control.on_time.max(500) as u64));
            self.apply_output(
                Kind::BinaryOutput,
                main,
                if state { 1.0 } else { 0.0 },
                revert,
            );
        }
        status
    }
}

macro_rules! analog_control {
    ($($variation:ty),*) => {$(
        impl ControlSupport<$variation> for Controls {
            fn select(&mut self, control: $variation, index: u16, _db: &mut DatabaseHandle) -> CommandStatus {
                let status = self.check(Kind::AnalogOutput, index);
                self.report("SELECT", Kind::AnalogOutput, index, &format!("value={}", control.value), status);
                status
            }

            fn operate(&mut self, control: $variation, index: u16, op_type: OperateType, _db: &mut DatabaseHandle) -> CommandStatus {
                let status = self.check(Kind::AnalogOutput, index);
                self.report(&format!("OPERATE ({op_type:?})"), Kind::AnalogOutput, index, &format!("value={}", control.value), status);
                if status == CommandStatus::Success {
                    self.apply_output(Kind::AnalogOutput, index, control.value as f64, None);
                }
                status
            }
        }
    )*};
}

analog_control!(Group41Var1, Group41Var2, Group41Var3, Group41Var4);
