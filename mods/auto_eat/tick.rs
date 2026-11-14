// Every game tick, on the window's game only: run the meal's state machine —
// choose the food slot, select it a tick ahead (the carried-slot packet
// must lead the use on the wire: a server reads the bite with the slot it
// knows), press use through the game's own input mapping under the agent
// source, hold it until the stack's count drops, and hand the hotbar back.
//
// The item-use shape (`GuestUse::Item`) keeps a block under the crosshair
// from taking the click: the press eats, whatever the player is looking at.
// Copyright (c) 2026 hakergeniusz. MIT.

// Every game tick: run the meal's state machine.
fn auto_eat_tick(game: &mut crate::game::Game<'_>) {
    use crate::game::auto_eat as eat;
    use crate::game::auto_eat_state as state;

    if !game.native_on(eat::ID) || !game.native_window() {
        return;
    }
    // Any screen over the world stops the machine; the meal in flight lets
    // its button go first — the screen's own release already bled the press.
    if game.screen != crate::game::Screen::Playing || game.dialog_open() {
        if let state::Mode::Holding { saved, .. } = state::get() {
            let _ = game.mouse_mapping(
                winit::event::MouseButton::Right,
                false,
                horizon_client::held::Source::Agent,
            );
            game.guest_use = None;
            game.selected = saved;
        }
        state::reset();
        return;
    }
    let Some(p) = game.player.as_ref() else {
        state::reset();
        return;
    };
    let food = p.health.food.food as f32;

    match state::get() {
        state::Mode::Idle => {
            if !eat::should_eat(food) || game.right || game.spectator() {
                return;
            }
            // The hotbar only, the mod's list, best first.
            let slot = (0..=8).find(|&i| {
                game.inventory[i]
                    .as_ref()
                    .is_some_and(|s| eat::is_food(s.item.as_ref()))
            });
            let Some(slot) = slot else {
                return;
            };
            state::set(state::Mode::Selecting { slot, saved: game.selected });
        }
        state::Mode::Selecting { slot, saved } => {
            // The carried-slot packet goes out with this tick's input
            // handling; the press waits for the next.
            game.selected = slot;
            state::set(state::Mode::Holding { slot, saved, ticks: 0, had: -1 });
        }
        state::Mode::Holding { slot, saved, ticks, had } => {
            if ticks == 0 {
                let Some(stack) = game.inventory[slot].as_ref() else {
                    state::set(state::Mode::Done { saved });
                    return;
                };
                let had = i64::from(stack.count);
                // The item-use shape makes the press the food's own use,
                // then the button holds through the input handling.
                game.guest_use =
                    Some(crate::game::GuestUse::Item(horizon_client::first_person::Hand::Main));
                let _ = game.mouse_mapping(
                    winit::event::MouseButton::Right,
                    true,
                    horizon_client::held::Source::Agent,
                );
                state::set(state::Mode::Holding { slot, saved, ticks: 1, had });
            } else {
                let count = game.inventory[slot]
                    .as_ref()
                    .map(|s| i64::from(s.count))
                    .unwrap_or(had - 1);
                if count < had {
                    // The bite landed: let go before the next one starts.
                    state::release(game, saved);
                } else if ticks >= eat::EAT_TICKS {
                    // The bite never landed (the use was taken, the world
                    // refused it): let go and try the next low tick.
                    state::release(game, saved);
                } else {
                    state::set(state::Mode::Holding { slot, saved, ticks: ticks + 1, had });
                }
            }
        }
        state::Mode::Done { saved } => {
            game.guest_use = None;
            game.selected = saved;
            state::reset();
        }
    }
}

