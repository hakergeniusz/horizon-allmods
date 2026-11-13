// The mod's settings page: the Enabled toggle, then two read-only rows
// saying what the slot holds and what chestpiece the mod keeps, read fresh
// from the setting files so the page always shows what the gates act on.
// Copyright (c) 2026 hakergeniusz. MIT.
fn elytra_slot_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use crate::game::elytra_slot as elytra;
    if mod_id != elytra::ID {
        return;
    }
    let game = &*game;
    let on = elytra::setting_on(|key| game.native_setting(elytra::ID, key));
    let worn = elytra::equipped_of(|key| game.native_setting(elytra::ID, key))
        && elytra::is_elytra(game.armor[1].as_ref().map(|s| &*s.item));
    let kept = elytra::stored_of(|key| game.native_setting(elytra::ID, key)).and_then(|slot| {
        game.inventory
            .get(slot)
            .and_then(|s| s.as_ref())
            .map(|s| game.pack.names.display_item(&s.item))
    });
    buttons.extend(elytra::labels(on, worn, kept.as_deref()));
}
