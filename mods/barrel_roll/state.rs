// The roll's shared state: the tick and look injections integrate it, the
// bob injection reads the frame's interpolation. One mutex-guarded snapshot,
// so a reader never blocks a writer beyond a copy.
// Copyright (c) 2026 hakergeniusz. MIT.
use std::sync::Mutex;

/// One flight's roll: the last two integrated angles (the render lerps
/// between them), the yaw the last tick saw (the banking's rate), and
/// whether the elytra carried the player as of that tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flight {
    pub prev_roll: f64,
    pub roll: f64,
    pub prev_yaw: f32,
    pub gliding: bool,
}

impl Default for Flight {
    fn default() -> Self {
        Self {
            prev_roll: 0.0,
            roll: 0.0,
            prev_yaw: 0.0,
            gliding: false,
        }
    }
}

static FLIGHT: Mutex<Flight> = Mutex::new(Flight {
    prev_roll: 0.0,
    roll: 0.0,
    prev_yaw: 0.0,
    gliding: false,
});

/// The flight as of the last tick.
pub fn held() -> Flight {
    FLIGHT.lock().map(|f| *f).unwrap_or_default()
}

/// The mouse's sideways turn (a frame amount in turn units) rolls at once.
pub fn add_mouse_roll(turn: f64, gliding: bool) {
    if let Ok(mut f) = FLIGHT.lock() {
        let was = f.roll;
        f.roll = crate::game::barrel_roll::step(f.roll, turn, 0.0, gliding);
        f.prev_roll = was;
        f.gliding = gliding;
    }
}

/// One tick's integration: the banking chase (and the level settle off the
/// elytra) from `yaw`, the player's current yaw in degrees.
pub fn integrate(yaw: f32, gliding: bool) {
    if let Ok(mut f) = FLIGHT.lock() {
        let rate = f64::from(yaw - f.prev_yaw);
        let bank = crate::game::barrel_roll::banking(rate);
        let was = f.roll;
        f.roll = crate::game::barrel_roll::step(f.roll, 0.0, bank, gliding);
        f.prev_roll = was;
        f.prev_yaw = yaw;
        f.gliding = gliding;
    }
}

/// Everything level: the mod toggled off (or the world went away).
pub fn reset() {
    if let Ok(mut f) = FLIGHT.lock() {
        *f = Flight::default();
    }
}

/// The frame's rendered roll, in radians — None when the last tick was not
/// a glide (the vanilla bob runs untouched).
pub fn rendered(partial: f32) -> Option<f64> {
    let f = held();
    f.gliding.then(|| {
        crate::game::barrel_roll::rendered(f.prev_roll, f.roll, partial)
    })
}
