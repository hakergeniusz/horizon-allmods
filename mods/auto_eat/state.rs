// The meal's progress, game-side: which step the machine is on and whose
// hotbar slot was the player's own. Pure bookkeeping — the decision is the
// runtime's, the press and the release are the tick helper's.
// Copyright (c) 2026 hakergeniusz. MIT.
use core::cell::RefCell;

/// One meal, from the food slot's selection to the button's release.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mode {
    Idle,
    /// The food slot is chosen; the carried-slot packet goes out this tick,
    /// so the next tick's press is read with the right slot held.
    Selecting { slot: usize, saved: usize },
    /// The button is held; the bite runs its ticks.
    Holding { slot: usize, saved: usize, ticks: u32, had: i64 },
    /// Handing the hotbar back to the player.
    Done { saved: usize },
}

thread_local! {
    static MODE: RefCell<Mode> = const { RefCell::new(Mode::Idle) };
}

/// The machine's step now.
pub fn get() -> Mode {
    MODE.with(|m| *m.borrow())
}

/// Move the machine to `mode`.
pub fn set(mode: Mode) {
    MODE.with(|m| *m.borrow_mut() = mode);
}

/// Forget the meal: the next low tick starts over.
pub fn reset() {
    set(Mode::Idle);
}

/// Let the button go and hand the hotbar back. A release never fights the
/// player's own hand: the mapping waits while another device holds it.
pub fn release(game: &mut crate::game::Game<'_>, saved: usize) {
    let _ = game.mouse_mapping(
        winit::event::MouseButton::Right,
        false,
        horizon_client::held::Source::Agent,
    );
    set(Mode::Done { saved });
}
