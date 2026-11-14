// The elytra slot, drawn over the plain survival inventory once a frame
// (`native_screen_render_post`, after the screen, its tooltips and the
// carried stack): the slot frame right of the player model, the bay's elytra
// in it — or the dimmed ghost when the bay is empty, the way the armor
// ghosts draw — the vanilla hover highlight, and the tooltip naming the
// elytra and the bay's rules. Read-only: the click answers in its own helper.
// Copyright (c) 2026 hakergeniusz. MIT.
fn elytra_slot_draw(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
) {
    use crate::game::elytra_slot as elytra;
    use crate::game::elytra_slot_bay as bay;
    use horizon_client::items::ItemStack;
    if screen.id != "inventory" || screen.menu.is_some() {
        return;
    }
    if !game.native_on(elytra::ID) || !game.native_window() {
        return;
    }
    if !elytra::setting_on(|k| game.native_setting(elytra::ID, k)) {
        return;
    }
    if !elytra::plain_inventory(
        game.container.is_none(),
        game.station.is_none(),
        game.craft_size == 2,
        game.mount_active(),
        game.creative_tab_open(),
    ) {
        return;
    }
    if game.book_covers_menu(batch.width) {
        return;
    }
    let origin = game.inventory_origin(batch);
    let (x, y) = (origin.0 + elytra::SLOT_X, origin.1 + elytra::SLOT_Y);
    let hovered = horizon_client::ui::hovered(game.cursor, x, y, elytra::SLOT_SIZE, elytra::SLOT_SIZE);
    let atlas = &game.gui.atlas;
    batch.sprite(atlas, "gui/sprites/container/slot", x, y);
    // While the cursor carries a stack over the slot, the carried stack is
    // already drawn there: the slot's own content stays hidden under it.
    let show_content = !(hovered && game.carried.is_some());
    let held = elytra::held();
    if show_content {
        if let Some(stack) = held.as_ref().map(bay::from_text) {
            game.draw_item(batch, &stack, x, y);
        } else {
            let ghost = ItemStack::new(elytra::ELYTRA, 1);
            game.draw_item(batch, &ghost, x, y);
            batch.fill(atlas, x + 1.0, y + 1.0, 16.0, 16.0, 0x9000_0000);
        }
    }
    if hovered {
        horizon_client::ui::blit_sprite(
            batch,
            atlas,
            "gui/sprites/container/slot_highlight_front",
            x - 4.0,
            y - 4.0,
            24.0,
            24.0,
            u32::MAX,
        );
        if game.carried.is_none() {
            let mut lines: Vec<(String, u32)> = Vec::new();
            match held.as_ref().map(bay::from_text) {
                Some(stack) => {
                    let names = |item: &str| game.pack.names.display_item(item);
                    lines.push((horizon_client::stations::hover_name(&names, &stack), u32::MAX));
                }
                None => lines.push(("Elytra Slot".to_owned(), u32::MAX)),
            }
            if game.remote.is_some() {
                lines.push(("Glides in single player and LAN".to_owned(), 0xFFA0_A0A0));
            } else {
                lines.push(("Your chestplate stays worn beside it".to_owned(), 0xFFA0_A0A0));
            }
            game.draw_tooltip_lines(batch, &lines, game.cursor.0, game.cursor.1);
        }
    }
}
