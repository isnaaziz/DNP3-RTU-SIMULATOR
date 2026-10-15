use std::f64::consts::PI;
use std::time::{Duration, Instant};

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::model::{Kind, SimMode, Simulation, normalize};
use crate::state::Shared;

pub async fn run(shared: Shared) {
    let started = Instant::now();
    let mut rng = StdRng::from_entropy();
    let mut ticker = tokio::time::interval(Duration::from_secs(1));
    loop {
        ticker.tick().await;
        let t = started.elapsed().as_secs_f64();
        let mut app = shared.lock();
        let mut changed = Vec::new();
        for point in app.config.points.iter_mut() {
            let Some(sim) = &point.data.sim else { continue };
            if let Some(value) = next(point.data.kind, point.data.value, sim, t, &mut rng)
                && value != point.data.value
            {
                point.data.value = value;
                changed.push(point.clone());
            }
        }
        app.write(&changed);
    }
}

fn next(kind: Kind, current: f64, sim: &Simulation, t: f64, rng: &mut StdRng) -> Option<f64> {
    let span = sim.max - sim.min;
    let phase = t / sim.period_s;
    let second_half = (phase as u64) % 2 == 1;
    let value = match (kind, sim.mode) {
        (Kind::Counter, _) => current + rng.gen_range(sim.min..=sim.max).round(),
        (Kind::DoubleBit, SimMode::Toggle) => {
            if second_half {
                2.0
            } else {
                1.0
            }
        }
        (_, SimMode::Toggle) => {
            if second_half {
                sim.max
            } else {
                sim.min
            }
        }
        (_, SimMode::Random) => {
            (current + rng.gen_range(-1.0..=1.0) * span * 0.05).clamp(sim.min, sim.max)
        }
        (_, SimMode::Sine) => sim.min + span * (0.5 + 0.5 * (2.0 * PI * phase).sin()),
        (_, SimMode::Ramp) => sim.min + span * phase.fract(),
    };
    normalize(kind, (value * 1000.0).round() / 1000.0).ok()
}
