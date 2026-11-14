// The glide check (`Game::glider_worn`), before the game's own answer: the
// mod's bay counts as worn elytra — the chestplate stays worn beside it.
// On a remote server the check stays vanilla (the slot is display-only
// there; the server's own equipment check would end the glide), and so it
// is when the mod or its setting is off.
// Copyright (c) 2026 hakergeniusz. MIT.
fn elytra_slot_glider(game: &crate::game::Game<'_>) -> core::ops::ControlFlow<bool> {
    use crate::game::elytra_slot as elytra;
    if !game.native_on(elytra::ID) || game.remote.is_some() {
        return core::ops::ControlFlow::Continue(());
    }
    if !elytra::setting_on(|k| game.native_setting(elytra::ID, k)) {
        return core::ops::ControlFlow::Continue(());
    }
    if elytra::held().is_some_and(|stack| stack.is_elytra()) {
        core::ops::ControlFlow::Break(true)
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
