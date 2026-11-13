// Barrel Roll's brain: pure standard library, no game types, so it compiles
// and tests wherever — as this crate's library and, injected, as a module of
// the game. The roll model: while the elytra carries the player, the mouse's
// sideways turn feeds the camera's roll instead of the yaw (the strafe keys
// steer the yaw), the roll banks into the turn the yaw rate makes, and
// everything settles back to level on the ground. The camera and the world
// read a partial-tick interpolation of the roll.
//
// Clean-room: written for Horizon from the game's own public behavior; it
// copies nothing from any mod. The idea is enjarai's Do a Barrel Roll.
//
// Copyright (c) 2026 hakergeniusz. MIT.

/// The mod's registry id: the Options > Mods row and the injection gates.
pub const ID: &str = "barrel_roll";

/// Radians of roll per mouse turn unit: a full across-the-screen swipe
/// (~120 turn units) rolls about a quarter circle.
pub const ROLL_PER_TURN: f64 = 0.012;
/// Yaw turn units per tick while a strafe key is held (A or D): the elytra's
/// rudder. Sixty turn units a tick turns ~9 degrees per second.
pub const STRAFE_TURN: f64 = 60.0;
/// Radians of banking per degree per tick of yaw rate: banking rolls into
/// the turn the rudder makes.
pub const BANK_PER_DEGREE: f64 = 0.02;
/// The roll chases the banking target this far each gliding tick.
pub const BANK_LERP: f64 = 0.12;
/// Off the elytra, the roll settles level this far each tick.
pub const SETTLE_LERP: f64 = 0.25;
/// The roll never winds past a full circle either way.
pub const MAX_ROLL: f64 = std::f64::consts::PI;
/// The camera's roll sign: positive roll (a swipe right) rolls the wings
/// clockwise seen from behind, which the bob matrix reads as a negative
/// Z rotation.
pub const CAMERA_SIGN: f32 = -1.0;

/// The banking target for this tick's yaw rate (degrees per tick, positive
/// turning right): banking rolls into the turn — right turn, right wing down.
pub fn banking(yaw_rate_degrees: f64) -> f64 {
    (-yaw_rate_degrees * BANK_PER_DEGREE).clamp(-0.9, 0.9)
}

/// The strafe's rudder turn for this tick: A turns left (+), D right (-).
pub fn strafe_turn(left: bool, right: bool) -> f64 {
    match (left, right) {
        (true, false) => STRAFE_TURN,
        (false, true) => -STRAFE_TURN,
        _ => 0.0,
    }
}

/// One tick of the roll: the mouse's sideways turn (a frame amount, added
/// straight in), then the chase toward the banking target while gliding —
/// or toward level when the elytra is off. Wraps past a half circle.
pub fn step(roll: f64, mouse_x: f64, bank: f64, gliding: bool) -> f64 {
    let mut roll = roll + mouse_x * ROLL_PER_TURN;
    let target = if gliding { bank } else { 0.0 };
    let lerp = if gliding { BANK_LERP } else { SETTLE_LERP };
    roll += (target - roll) * lerp;
    roll.clamp(-MAX_ROLL, MAX_ROLL)
}

/// The frame's rendered roll between the last two ticks.
pub fn rendered(prev: f64, roll: f64, partial: f32) -> f64 {
    prev + (roll - prev) * f64::from(partial)
}

/// The rotation the bob matrix reads for a roll: `CAMERA_SIGN` times the
/// roll, in degrees for `Mat4::rotate_z`.
pub fn camera_degrees(roll: f64) -> f32 {
    (roll as f32).to_degrees() * CAMERA_SIGN
}

/// The version and description the Options > Mods row shows.
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str =
    "Barrel rolls for the elytra: the mouse rolls, the strafe keys steer, the camera banks.";

/// The settings row's name.
pub const ENABLED: &str = "enabled";

/// The gate reads the toggle straight from the file each time.
pub fn setting_on(read: impl Fn(&str) -> Option<String>) -> bool {
    read(ENABLED).is_none_or(|value| value == "true")
}

/// The row's label for the current state.
pub fn label(on: bool) -> String {
    if on {
        "Enabled: on".into()
    } else {
        "Enabled: off".into()
    }
}

/// A click's new state: row 0 is the Enabled toggle; None for other rows.
pub fn click(button: usize, on: bool) -> Option<bool> {
    (button == 0).then_some(!on)
}

/// What a new state stores: the key and the value the file takes.
pub fn stored(on: bool) -> (&'static str, &'static str) {
    (ENABLED, if on { "true" } else { "false" })
}

#[cfg(test)]
mod tests {
    use super::{
        MAX_ROLL, banking, camera_degrees, click, label, rendered, setting_on, step, stored,
        strafe_turn,
    };

    #[test]
    fn the_rudder_turns_one_way_per_key() {
        assert!(strafe_turn(true, false) > 0.0, "A turns left");
        assert!(strafe_turn(false, true) < 0.0, "D turns right");
        assert_eq!(strafe_turn(true, true), 0.0);
        assert_eq!(strafe_turn(false, false), 0.0);
    }

    #[test]
    fn banking_rolls_into_the_turn() {
        // A right turn (positive yaw rate) banks the right wing down, which
        // this model stores as a negative roll.
        assert!(banking(1.2) < 0.0);
        assert!(banking(-1.2) > 0.0);
        assert_eq!(banking(0.0), 0.0);
        let hard = banking(1000.0);
        assert!(hard.abs() <= 0.9, "the bank clamps at a wing-over");
    }

    #[test]
    fn the_roll_chases_the_bank_while_gliding() {
        let mut roll = 0.0;
        let bank = banking(10.0);
        for _ in 0..60 {
            roll = step(roll, 0.0, bank, true);
        }
        assert!(
            (roll - bank).abs() < 0.01,
            "a held turn banks fully: {} vs {}",
            roll,
            bank
        );
    }

    #[test]
    fn the_roll_settles_level_off_the_elytra() {
        let mut roll = 2.0;
        for _ in 0..120 {
            roll = step(roll, 0.0, 0.5, false);
        }
        assert!(roll.abs() < 0.01, "grounded roll settles: {}", roll);
    }

    #[test]
    fn the_mouse_swipes_the_roll_and_it_stays_bounded() {
        let mut roll = 0.0;
        for _ in 0..100_000 {
            roll = step(roll, 120.0, 0.0, true);
        }
        assert!(roll.abs() <= MAX_ROLL, "the roll clamps: {}", roll);
        assert!(roll > 0.0, "a rightward swipe rolls right: {}", roll);
    }

    #[test]
    fn the_enabled_toggle_flips_and_stores() {
        assert!(setting_on(|_| None), "a missing setting defaults on");
        assert!(setting_on(|k| (k == "enabled").then(|| "true".into())));
        assert!(!setting_on(|k| (k == "enabled").then(|| "false".into())));
        assert_eq!(label(true), "Enabled: on");
        assert_eq!(click(0, true), Some(false));
        assert_eq!(click(1, true), None, "other rows are not ours");
        let (key, value) = stored(false);
        assert_eq!((key, value), ("enabled", "false"));
    }

    #[test]
    fn the_render_interpolates_between_ticks() {
        assert_eq!(rendered(0.0, 1.0, 0.0), 0.0);
        assert_eq!(rendered(0.0, 1.0, 1.0), 1.0);
        assert!((rendered(0.0, 1.0, 0.5) - 0.5).abs() < 1.0e-9);
        let degrees = camera_degrees(1.0);
        assert!(
            degrees < 0.0,
            "positive roll reads as a negative Z rotation"
        );
    }
}
