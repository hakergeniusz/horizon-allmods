// Every tick: the bay refills from its config file once it is empty — the
// file is the durable copy (the bay survives restarts and deaths, because
// the elytra it holds is not part of the world's inventory). A missing or
// junk file answers None and clears nothing: the live bay is never taken
// away by a file that cannot be read.
// Copyright (c) 2026 hakergeniusz. MIT.
fn elytra_slot_tick(game: &mut crate::game::Game<'_>) {
    use crate::game::elytra_slot as elytra;
    if !game.native_on(elytra::ID) {
        return;
    }
    if !elytra::setting_on(|k| game.native_setting(elytra::ID, k)) {
        return;
    }
    if elytra::held().is_none()
        && let Ok(dir) = crate::game::Game::native_config_dir()
        && let Some(stack) = elytra::load_bay(&dir)
    {
        elytra::set(Some(stack));
    }
}
