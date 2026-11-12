// Every tick, on the window's Game only: while the flight toggle is on and
// the mode does not already fly, the local player's flight grant is
// re-asserted — the game's spawn sync (`spawn_player`) and the server's
// ability packets both reset `may_fly`/`flying` from the game mode, and a
// granted flight dies quietly without this. Toggled off, the mod touches
// nothing: the fields are already cleared at the toggle.
// Copyright (c) 2026 hakergeniusz. MIT.
fn fly_tick(game: &mut crate::game::Game<'_>) {
    use crate::game::fly as fly;
    if !game.native_on(fly::ID) {
        return;
    }
    if !game.native_window() {
        return;
    }
    let enabled = fly::setting_on(|k| game.native_setting(fly::ID, k));
    let on = fly::flight_on() && enabled;
    let creative = game.creative;
    let spectator = game.mode == horizon_client::command::args::GameMode::Spectator;
    if let Some(p) = game.player.as_mut() {
        if let Some((may_fly, flying)) = fly::reassert(on, creative, spectator, p.may_fly, p.flying) {
            p.may_fly = may_fly;
            p.flying = flying;
        }
    }
}
