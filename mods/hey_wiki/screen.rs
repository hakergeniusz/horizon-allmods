// The wiki screen, drawn whole once a frame over the world: the backdrop,
// a centered panel, and whatever the view holds — the search box with its
// caret, the results a search found, the wait on a worker, or the page's
// wrapped text with its scrollbar and the browser's three buttons. Any
// worker's answer that landed since the last frame applies first.
// Copyright (c) 2026 hakergeniusz. MIT.
fn hey_wiki_screen(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
) {
    use crate::game::hey_wiki as wiki;
    if screen.id != "mod" || game.native_mod_screen() != Some(wiki::ID) {
        return;
    }
    if !game.native_on(wiki::ID) || !game.native_window() {
        return;
    }
    // The workers' answers land here, between frames.
    if let Some(outcome) = wiki::take_outcome() {
        wiki::with_viewer(|viewer| viewer.apply(outcome));
    }
    let (w, h) = (screen.rect[2], screen.rect[3]);
    let panel = [
        (w - wiki::PANEL_W) / 2.0,
        (h - wiki::PANEL_H) / 2.0,
        wiki::PANEL_W,
        wiki::PANEL_H,
    ];
    let [px, py, pw, ph] = panel;
    // The backdrop dims the world the way a screen does; the panel is the
    // standard box over it, its border four thin fills.
    game.native_fill(batch, [0.0, 0.0, w, h], 0x90000000);
    game.native_fill(batch, panel, 0xF0101010);
    let border = 0xFF3C3C3C;
    let t = 1.0;
    game.native_fill(batch, [px, py, pw, t], border);
    game.native_fill(batch, [px, py + ph - t, pw, t], border);
    game.native_fill(batch, [px, py, t, ph], border);
    game.native_fill(batch, [px + pw - t, py, t, ph], border);
    let mode = wiki::with_viewer(|viewer| viewer.mode.clone());
    match mode {
        // The search box: the wiki's name, the query with its caret (an
        // underscore dropped into the text where the caret sits), the hint.
        wiki::Mode::Search { query } => {
            game.native_text(
                batch,
                "Search the Minecraft Wiki",
                [px + 10.0, py + 9.0],
                0xFFFFFF,
                false,
            );
            let (fx, fy, fw, fh) = (px + 8.0, py + 28.0, pw - 16.0, 22.0);
            game.native_fill(batch, [fx, fy, fw, fh], 0xF0000000);
            game.native_fill(batch, [fx + 3.0, fy + 1.0, fw - 6.0, fh - 2.0], 0xFF2A2A2A);
            if query.value.is_empty() {
                game.native_text(
                    batch,
                    "What are you looking for?",
                    [fx + 5.0, fy + 7.0],
                    0x808080,
                    false,
                );
            } else {
                // The caret sits where the box's cursor does: the text
                // split around an underscore, character-counted like the
                // cursor is.
                let head: String = query.value.chars().take(query.cursor).collect();
                let tail: String = query.value.chars().skip(query.cursor).collect();
                let shown = head + "_" + &tail;
                game.native_text(batch, &shown, [fx + 5.0, fy + 7.0], 0xFFFFFF, true);
            }
            game.native_text(
                batch,
                "Enter: search the wiki   Esc: close",
                [px + 10.0, py + ph - 18.0],
                0xA0A0A0,
                false,
            );
        }
        // The wait: what the worker is after, over the box it came from.
        wiki::Mode::Loading { note } => {
            game.native_text(
                batch,
                "The Minecraft Wiki",
                [px + 10.0, py + 9.0],
                0xFFFFFF,
                false,
            );
            game.native_text(batch, &note, [px + 10.0, py + 40.0], 0xFFFFFF, true);
            game.native_text(
                batch,
                "Esc: close",
                [px + 10.0, py + ph - 18.0],
                0xA0A0A0,
                false,
            );
        }
        // The results: the ranked titles the wiki found, the selection
        // highlighted, kept three rows from the window's top edge.
        wiki::Mode::Results { titles, selected } => {
            game.native_text(
                batch,
                &format!("{} results — Enter opens, Esc goes back", titles.len()),
                [px + 10.0, py + 9.0],
                0xFFFFFF,
                false,
            );
            let shown = ((ph - 64.0) / 12.0).floor().max(1.0) as usize;
            let start = selected
                .saturating_sub(3)
                .min(titles.len().saturating_sub(shown.min(titles.len())));
            for (row, index) in (start..titles.len().min(start + shown)).enumerate() {
                let y = py + 34.0 + row as f32 * 12.0;
                if index == selected {
                    game.native_fill(batch, [px + 6.0, y - 2.0, pw - 12.0, 12.0], 0x40FFFFFF);
                }
                game.native_text(
                    batch,
                    &titles[index],
                    [px + 10.0, y],
                    if index == selected { 0xFFFFFF } else { 0xC0C0C0 },
                    index == selected,
                );
            }
            game.native_text(
                batch,
                "Up/Down: pick   Enter: open   Esc: back",
                [px + 10.0, py + ph - 18.0],
                0xA0A0A0,
                false,
            );
        }
        // The page: the title, the wrapped text one line a piece (the
        // scroll's window), a scrollbar at the right, and the browser's
        // three buttons, with the rects the click helper answers.
        wiki::Mode::Page {
            title,
            lines,
            scroll,
            failed,
            ..
        } => {
            game.native_text(
                batch,
                &title,
                [px + 10.0, py + 9.0],
                if failed { 0xFF5555 } else { 0xFFFF55 },
                false,
            );
            for (row, line) in lines.iter().skip(scroll).take(wiki::BODY_LINES).enumerate() {
                if line.is_empty() {
                    continue;
                }
                game.native_text(
                    batch,
                    line,
                    [px + 10.0, py + 30.0 + row as f32 * wiki::LINE_HEIGHT],
                    0xFFFFFF,
                    true,
                );
            }
            // The scrollbar: a thin bar down the panel's right, its thumb
            // where the scroll is, when the text runs past the view.
            let max = wiki::max_scroll(lines.len(), wiki::BODY_LINES);
            if max > 0 {
                let bar_h = ph - 96.0;
                let thumb = (bar_h / max as f32).max(8.0).min(bar_h);
                let at = (bar_h - thumb) * scroll as f32 / max as f32;
                game.native_fill(batch, [px + pw - 5.0, py + 34.0, 3.0, bar_h], 0x30FFFFFF);
                game.native_fill(
                    batch,
                    [px + pw - 5.0, py + 34.0 + at, 3.0, thumb],
                    0xA0FFFFFF,
                );
            }
            for (plan, rect) in wiki::button_rects(panel) {
                game.native_fill(batch, rect, 0xFF333333);
                game.native_text(
                    batch,
                    wiki::button_label(plan),
                    [rect[0] + 6.0, rect[1] + 5.0],
                    0xFFFFFF,
                    false,
                );
            }
            game.native_text(
                batch,
                "Wheel/Up/Down: scroll   Esc: back",
                [px + 10.0, py + ph - 18.0],
                0xA0A0A0,
                false,
            );
        }
    }
}
