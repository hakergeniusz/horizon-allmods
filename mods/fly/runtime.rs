// Fly's brain: pure standard library, no game types, so it compiles and
// tests wherever — as this crate's library and, injected, as a module of the
// game. The policy is one toggle: while it is on, the local player is
// re-granted flight every tick (the game's own spawn and mode sync resets
// `may_fly` from the game mode, and the server's ability packets say the
// same, so a granted flight dies quietly without the re-assert); while it
// is off, the mod touches nothing. The toggle itself is a keybind (the
// freecam pattern): pressing it flips the state and writes both fields at
// once — the engine's own remote tick notices the `flying` flip and tells
// the server on its own, so the mod sends no packets of its own.
//
// Clean-room: written for Horizon from the game's own public behavior; it
// copies nothing from any mod.
//
// Copyright (c) 2026 hakergeniusz. MIT.

use std::sync::atomic::{AtomicBool, Ordering};

/// The mod's registry id: the settings folder and the Options > Mods row.
pub const ID: &str = "fly";
/// The version the row shows.
pub const VERSION: &str = "0.1.0";
/// The mod's description for its row.
pub const DESCRIPTION: &str = "Survival flight: press your fly key (R) to toggle creative-style \
flight anywhere, and again to land. The flight stays on through mode changes \
until you toggle it off.";

/// The toggle's setting file (one file per key under the mod's data dir),
/// which every gate re-reads: a change lands within the tick that follows it.
pub const SETTING: &str = "enabled";
/// The keybind's setting name, as freecam's toggle registers its own.
pub const TOGGLE: &str = "toggle";
/// The keybind out of the box: R is unbound in the game and in the other
/// native mods (F4 freecam, G fullbright, C zoom).
pub const DEFAULT_TOGGLE_KEY: &str = "key.keyboard.r";

/// Whether the mod is on. A missing file means ON — the mod exists to fly,
/// so installing it is asking for it; anything but `false` in the file is
/// on, trimmed like the game reads its settings.
pub fn setting_on(get: impl FnOnce(&str) -> Option<String>) -> bool {
    !matches!(get(SETTING).as_deref(), Some(v) if v.trim() == "false")
}

/// Whether this key event is the fly toggle's: the player is playing (not a
/// menu), the mod is on, they are not in a mode that already flies (creative
/// and spectator have the game's own flight), and the exact toggle key went
/// down. Releases and other keys are nothing.
pub fn is_toggle(
    playing: bool,
    creative: bool,
    spectator: bool,
    enabled: bool,
    pressed: bool,
    key: &str,
    toggle_key: &str,
) -> bool {
    playing && enabled && pressed && !creative && !spectator && key == toggle_key
}

/// The flight fields to write for the toggle's new value: granting flight
/// sets `may_fly` and lifts off; landing clears both, so no stale grant
/// survives the toggle.
pub fn apply(on: bool) -> (bool, bool) {
    if on { (true, true) } else { (false, false) }
}

/// Re-asserted every tick while the toggle is on: the game resets `may_fly`
/// from the game mode on every spawn (`spawn_player`) and the server's
/// ability packets say `may_fly = false` in survival, so a granted flight
/// must be re-granted or it silently dies. The flight stays on until the
/// player toggles it off — landing by touch does not end it, exactly like a
/// fly mod should. Creative and spectator already fly; nothing to enforce.
/// `None` leaves the fields alone.
pub fn reassert(
    on: bool,
    creative: bool,
    spectator: bool,
    may_fly: bool,
    flying: bool,
) -> Option<(bool, bool)> {
    if on && !creative && !spectator && (!may_fly || !flying) {
        Some((true, true))
    } else {
        None
    }
}

/// The settings page's single row.
pub fn label(on: bool) -> String {
    format!("Enabled: {}", if on { "ON" } else { "OFF" })
}

/// A click on the settings page's row `i`: the first row toggles the mod,
/// the rest are nothing (the page has the one row). Returns the new value
/// when the click was the toggle's, so the caller stores only real clicks.
pub fn click(i: usize, on: bool) -> Option<bool> {
    (i == 0).then_some(!on)
}

/// What to store for a value: the one setting key and its file text.
pub fn stored(on: bool) -> (&'static str, String) {
    (SETTING, if on { "true" } else { "false" }.to_owned())
}

// The toggle's own state, shared by the key and tick helpers (both are
// injected next to each other; the runtime module they read is
// `pub(crate)`-visible crate-wide). Default: not flying.
static FLIGHT_ON: AtomicBool = AtomicBool::new(false);

/// Whether the player's flight is toggled on.
pub fn flight_on() -> bool {
    FLIGHT_ON.load(Ordering::Acquire)
}

/// Remember the toggle's new value.
pub fn set_flight(on: bool) {
    FLIGHT_ON.store(on, Ordering::Release);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setting(value: &str) -> impl Fn(&str) -> Option<String> + '_ {
        move |key| (key == SETTING).then(|| value.to_owned())
    }

    /// Installed and never touched: the setting file does not exist yet, and
    /// the mod is on — installing it is asking for the flight. Only an
    /// explicit `false` turns it off, and what the click writes reads back
    /// as that state.
    #[test]
    fn installed_means_on_until_declined() {
        assert!(setting_on(|_| None), "the default must be on");
        assert!(setting_on(setting(" true \n")));
        assert!(setting_on(setting("on")), "anything but false is on");
        assert!(!setting_on(setting("false")));
        for on in [true, false] {
            let (key, value) = stored(on);
            assert_eq!(key, SETTING);
            assert_eq!(setting_on(setting(&value)), on, "{value} reads back {on}");
        }
    }

    /// The toggle key answers only the full gate: playing, the mod on, the
    /// key going down, and a mode that does not already fly. Each failed
    /// condition refuses the very same press that would otherwise toggle.
    #[test]
    fn the_toggle_answers_only_the_full_gate() {
        let good = (
            true,
            false,
            false,
            true,
            true,
            "key.keyboard.r",
            "key.keyboard.r",
        );
        assert!(is_toggle(
            good.0, good.1, good.2, good.3, good.4, good.5, good.6
        ));
        assert!(
            !is_toggle(
                false,
                false,
                false,
                true,
                true,
                "key.keyboard.r",
                "key.keyboard.r"
            ),
            "a menu is not play"
        );
        assert!(
            !is_toggle(
                true,
                true,
                false,
                true,
                true,
                "key.keyboard.r",
                "key.keyboard.r"
            ),
            "creative already flies"
        );
        assert!(
            !is_toggle(
                true,
                false,
                true,
                true,
                true,
                "key.keyboard.r",
                "key.keyboard.r"
            ),
            "spectator already flies"
        );
        assert!(
            !is_toggle(
                true,
                false,
                false,
                false,
                true,
                "key.keyboard.r",
                "key.keyboard.r"
            ),
            "the mod is off"
        );
        assert!(
            !is_toggle(
                true,
                false,
                false,
                true,
                false,
                "key.keyboard.r",
                "key.keyboard.r"
            ),
            "a release is not a press"
        );
        assert!(
            !is_toggle(
                true,
                false,
                false,
                true,
                true,
                "key.keyboard.g",
                "key.keyboard.r"
            ),
            "another key is nothing"
        );
    }

    /// Granting lifts off, landing clears both — no stale grant survives.
    #[test]
    fn apply_maps_the_flight_fields() {
        assert_eq!(apply(true), (true, true));
        assert_eq!(apply(false), (false, false));
    }

    /// While toggled on, a reset `may_fly`/`flying` is re-granted — whether
    /// the reset came from a spawn's mode sync or the server's ability
    /// packet. Creative and spectator already fly; the mod off; and an
    /// already-flying player all leave the fields alone.
    #[test]
    fn reassert_restores_only_a_lost_grant() {
        assert_eq!(
            reassert(true, false, false, false, false),
            Some((true, true))
        );
        assert_eq!(
            reassert(true, false, false, true, false),
            Some((true, true)),
            "may_fly survived, the lift did not"
        );
        assert_eq!(
            reassert(true, false, false, false, true),
            Some((true, true)),
            "the lift survived, the grant did not"
        );
        assert_eq!(
            reassert(true, false, false, true, true),
            None,
            "already flying"
        );
        assert_eq!(
            reassert(true, true, false, false, false),
            None,
            "creative flies on its own"
        );
        assert_eq!(
            reassert(true, false, true, false, false),
            None,
            "spectator flies on its own"
        );
        assert_eq!(
            reassert(false, false, false, false, false),
            None,
            "the mod is off"
        );
    }

    /// The settings page shows the one row in the state the files hold, and
    /// a click on it flips the state; clicks elsewhere change nothing.
    #[test]
    fn the_first_click_toggles_and_the_rest_are_nothing() {
        let (on, shown) = (false, label(false));
        assert_eq!(shown, "Enabled: OFF");
        assert_eq!(label(true), "Enabled: ON");
        assert_eq!(click(0, on), Some(true));
        assert_eq!(click(1, on), None, "the page has the one row");
        assert_eq!(click(7, true), None);
    }

    /// The toggle's state round-trips through the gate.
    #[test]
    fn the_flight_state_round_trips() {
        assert!(!flight_on(), "not flying out of the box");
        set_flight(true);
        assert!(flight_on());
        set_flight(false);
        assert!(!flight_on());
    }
}
