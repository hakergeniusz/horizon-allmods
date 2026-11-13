// The open bundle's panel, drawn over the plain survival inventory once a
// frame (`native_screen_render_post`, after the screen and its tooltips):
// the panel box, one real slot frame per entry with the game's own item
// rendering — counts, durability, tints all come free — the hover highlight
// and the hovered entry's name, and the weight bar under the grid, blue the
// way the bundle's own bar is until the bundle is full, then red. Read-only:
// the clicks answer in their own helper.
// Copyright (c) 2026 hakergeniusz. MIT.
fn bundle_inventory_draw(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
) {
    use crate::game::bundle_inventory as bi;
    use crate::game::bundle_inventory_hooks as hooks;
    use horizon_client::bundle::Contents;
    if screen.id != "inventory" || screen.menu.is_some() || !hooks::gates(game) {
        return;
    }
    if game.book_covers_menu(batch.width) {
        return;
    }
    let Some(open) = bi::open_slot() else {
        return;
    };
    let Some(view) = hooks::snapshot(game, open) else {
        return;
    };
    // The entries again, as the engine's own stacks: the panel draws them
    // with the game's own item renderer, resource packs included.
    let items = game.res.items();
    let Some(stack) = hooks::slot_stack(game, open) else {
        return;
    };
    let Some(contents) = Contents::of(items, stack) else {
        return;
    };
    let (ox, oy) = game.inventory_origin(batch);
    let panel = bi::panel_rect([ox, oy], batch.width);
    let atlas = &game.gui.atlas;
    let (shown, hidden) = bi::shown_count(view.entries.len());
    // The panel box: a one-pixel border around the vanilla inventory gray,
    // so it reads as part of the screen it stands beside.
    batch.fill(atlas, panel[0], panel[1], panel[2], panel[3], 0xFF37_3737);
    batch.fill(
        atlas,
        panel[0] + bi::BORDER,
        panel[1] + bi::BORDER,
        panel[2] - bi::BORDER * 2.0,
        panel[3] - bi::BORDER * 2.0,
        0xFFC6_C6C6,
    );
    for index in 0..shown {
        let [x, y, ..] = bi::entry_rect(panel, index);
        batch.sprite(atlas, "gui/sprites/container/slot", x, y);
        game.draw_item(batch, &contents.items()[index], x, y);
    }
    let at = [game.cursor.0, game.cursor.1];
    if let Some(index) = bi::entry_at(at, panel, view.entries.len()) {
        let [x, y, ..] = bi::entry_rect(panel, index);
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
        let names = |item: &str| game.pack.names.display_item(item);
        let mut lines = vec![(
            horizon_client::stations::hover_name(&names, &contents.items()[index]),
            u32::MAX,
        )];
        if hidden > 0 {
            lines.push((format!("and {hidden} more…"), 0xFFA0_A0A0));
        }
        game.draw_tooltip_lines(batch, &lines, game.cursor.0, game.cursor.1);
    }
    // The weight bar, under the grid: the bundle's own colors, the width the
    // weight's fraction of the grid.
    let bar = bi::bar_rect(panel, shown);
    batch.fill(atlas, bar[0], bar[1], bar[2], bar[3], 0x6800_0000);
    let fraction = contents.weight_fraction().clamp(0.0, 1.0);
    if fraction > 0.0 {
        let color = if contents.is_full() {
            0xFFFF_5454
        } else {
            0xFF70_87FF
        };
        batch.fill(
            atlas,
            bar[0],
            bar[1],
            (bar[2] * fraction).max(1.0),
            bar[3],
            color,
        );
    }
}
