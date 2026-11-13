// The frame's camera pose, while the roll carries an angle: the vanilla bob
// (hurt tilt, view bobbing, the shake) rebuilt exactly as the body builds it
// — the same public camera helpers, the same fields — then the roll read on
// top as a Z rotation. Without a roll the vanilla body runs untouched, so
// the ground game never sees this mod.
// Copyright (c) 2026 hakergeniusz. MIT.
fn barrel_bob(
    game: &crate::game::Game<'_>,
    partial: f32,
) -> std::ops::ControlFlow<glam::Mat4> {
    use crate::game::barrel_state as state;
    let Some(roll) = state::rendered(partial) else {
        return std::ops::ControlFlow::Continue(());
    };
    let Some(p) = game.player.as_ref() else {
        return std::ops::ControlFlow::Continue(());
    };
    let hurt = &p.health.hurt;
    let bob_state = horizon_client::camera::BobState {
        living: Some(horizon_client::camera::HurtBob {
            hurt_time: p.health.hurt_animation_time as f32 - partial,
            hurt_duration: hurt.hurt_duration,
            hurt_dir: p.health.hurt_dir,
            death_time: game.death_time().map(|t| t as f32 + partial),
        }),
        player: Some((
            game.avatar
                .backwards_interpolated_walk_distance(partial),
            game.avatar.interpolated_bob(partial),
        )),
    };
    let tilt = game.settings.file.double("damageTiltStrength");
    let mut pose = glam::Mat4::from_cols_array(
        &horizon_client::camera::bob_matrix(&bob_state, tilt, game.settings.view_bobbing).m,
    ) * game.native_shake_matrix(partial);
    // glam has no in-place rotate_z: the roll post-multiplies — the same
    // composition as the bob's own hurt tilt, applied last, in radians.
    pose = pose
        * glam::Mat4::from_rotation_z(
            crate::game::barrel_roll::camera_degrees(roll).to_radians(),
        );
    std::ops::ControlFlow::Break(pose)
}
