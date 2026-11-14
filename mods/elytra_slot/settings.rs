// The mod's settings page: the Enabled toggle, then a read-only row saying
// what the bay holds, read fresh from the live state so the page always
// shows what the gates act on — plus, after any failed save, why the bay
// could not be written (an item was never lost to a silent failure again).
// Copyright (c) 2026 hakergeniusz. MIT.
fn elytra_slot_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use crate::game::elytra_slot as elytra;
    if mod_id != elytra::ID {
        return;
    }
    let game = &*game;
    let on = elytra::setting_on(|key| game.native_setting(elytra::ID, key));
    buttons.extend(elytra::page_labels(on, elytra::held().is_some()));
    if let Some(error) = elytra::save_error() {
        buttons.push(format!("Bay save failed: {error}"));
    }
}
