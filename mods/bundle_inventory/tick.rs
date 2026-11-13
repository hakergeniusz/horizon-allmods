// Every tick: keep the panel honest, and run the plan the last click or key
// left. The panel closes itself the moment the world disagrees with it — the
// screen is no longer the plain survival inventory, or the bundle left its
// slot (a death, a vanilla click, the bundle emptied some other way). A plan
// runs step by step as ordinary menu clicks, each step's cursor expectation
// checked against the live game first, so a greedy transfer chain stops the
// moment the cursor empties and nothing is ever clicked onto a stack that
// would pick it up.
// Copyright (c) 2026 hakergeniusz. MIT.
fn bundle_inventory_tick(game: &mut crate::game::Game<'_>) {
    use crate::game::bundle_inventory as bi;
    use crate::game::bundle_inventory_hooks as hooks;
    if !game.native_on(bi::ID) {
        return;
    }
    if let Some(open) = bi::open_slot() {
        let plain = game.screen == crate::game::Screen::Inventory && hooks::gates(game);
        let is_bundle = plain
            && hooks::slot_stack(game, open)
                .is_some_and(|stack| game.res.items().get(&stack.item).is_some_and(|i| i.bundle));
        if !is_bundle {
            bi::set_open(None);
        }
    }
    let Some(steps) = bi::take_deferred() else {
        return;
    };
    let Some(open) = bi::open_slot() else {
        return;
    };
    for step in steps {
        match step {
            bi::Step::Select(index) => {
                if index >= 0 {
                    hooks::select(game, open, index);
                }
            }
            bi::Step::Take { bundle } => {
                // The menu's bundle click override takes the selected entry
                // out on a right-click — with an empty cursor only.
                if game.carried.is_some() || !hooks::menu_click(game, Some(bundle), true) {
                    break;
                }
            }
            bi::Step::Give { slot, allow_empty } => {
                if game.carried.is_none() {
                    if allow_empty {
                        // The put-back onto a slot the plan emptied itself:
                        // with nothing on the cursor the click would do
                        // nothing, so it does not go out at all.
                        continue;
                    }
                    break;
                }
                if !hooks::menu_click(game, slot, false) {
                    break;
                }
            }
            bi::Step::Carry { slot } => {
                if game.carried.is_some() || !hooks::menu_click(game, Some(slot), false) {
                    break;
                }
            }
        }
    }
}
