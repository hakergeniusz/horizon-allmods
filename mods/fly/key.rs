// The fly toggle: wherever a key goes down or up. The press of the mod's
// toggle key (default R, configurable like freecam's) flips the flight
// state and writes both flight fields at once — the engine's remote tick
// notices the `flying` flip and tells the server by itself, so this helper
// sends no packets of its own. The key is not broken: the game keeps it
// (an unbound key like R does nothing vanilla-side anyway).
// Copyright (c) 2026 hakergeniusz. MIT.
fn fly_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    use crate::game::fly as fly;
    if !game.native_on(fly::ID) {
        return;
    }
    let playing = game.native_playing();
    if !playing {
        return;
    }
    let toggle_key = game.native_mod_key(fly::ID, fly::TOGGLE, fly::DEFAULT_TOGGLE_KEY);
    let enabled = fly::setting_on(|k| game.native_setting(fly::ID, k));
    let creative = game.creative;
    let spectator = game.mode == horizon_client::command::args::GameMode::Spectator;
    if !fly::is_toggle(playing, creative, spectator, enabled, pressed, key, &toggle_key) {
        return;
    }
    let on = !fly::flight_on();
    fly::set_flight(on);
    if let Some(p) = game.player.as_mut() {
        let (may_fly, flying) = fly::apply(on);
        p.may_fly = may_fly;
        p.flying = flying;
    }
}
