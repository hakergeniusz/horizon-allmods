// The mouse's look, while the elytra carries the player: the sideways turn
// feeds the roll instead of the yaw (the rudder steers), the vertical turn
// keeps being the pitch. The zoom scale still applies, and the vanilla body
// runs untouched when the player is not gliding or the mod is off.
// Copyright (c) 2026 hakergeniusz. MIT.
fn barrel_look(
    game: &mut crate::game::Game<'_>,
    x: f64,
    y: f64,
) -> std::ops::ControlFlow<()> {
    use crate::game::barrel_state as state;
    if !game.native_on(crate::game::barrel_roll::ID) {
        return std::ops::ControlFlow::Continue(());
    }
    let gliding = game.screen == crate::game::Screen::Playing
        && game
            .player
            .as_ref()
            .is_some_and(|p| p.fall_flying);
    if !gliding {
        return std::ops::ControlFlow::Continue(());
    }
    let zoom = game.zoom_turn_scale();
    state::add_mouse_roll(x * zoom, true);
    game.turn_player(0.0, y * zoom);
    std::ops::ControlFlow::Break(())
}
