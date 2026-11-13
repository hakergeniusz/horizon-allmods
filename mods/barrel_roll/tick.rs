// Every game tick, on the window's game only: while the elytra carries the
// player, the strafe keys are the rudder (a yaw turn through the game's own
// `turn_player`, so ropes, the boat clamp and every other look side effect
// stay vanilla) and the roll banks into the resulting yaw rate; off the
// elytra, everything settles level. The mod toggled off resets the state.
// Copyright (c) 2026 hakergeniusz. MIT.
fn barrel_tick(game: &mut crate::game::Game<'_>) {
    use crate::game::barrel_state as state;
    if !game.native_on(crate::game::barrel_roll::ID) {
        state::reset();
        return;
    }
    if !game.native_window() {
        return;
    }
    let gliding = game.screen == crate::game::Screen::Playing
        && game
            .player
            .as_ref()
            .is_some_and(|p| p.fall_flying);
    if !gliding {
        state::integrate(game.yaw, false);
        return;
    }
    // The rudder: the strafe keys turn the yaw through the game's own turn.
    let turn = crate::game::barrel_roll::strafe_turn(
        game.mapped_key_down("key.left"),
        game.mapped_key_down("key.right"),
    );
    if turn != 0.0 {
        game.turn_player(turn, 0.0);
    }
    state::integrate(game.yaw, true);
}
