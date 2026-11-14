// The wiki screen, drawn whole once a frame over the world: the dim over
// the world, one flat dark panel with a large title row over a hairline,
// and whatever the view holds — the search field with its caret, the
// results a search found, the wait on a worker, or the page's wrapped text
// with its scrollbar and the browser's three buttons. The look is the
// engine's own flat fills only: one dark surface, one accent, hairline
// separators, no gradients, no borders but the hairlines. The results' rows
// and the buttons keep the geometry the click plan answers. Any worker's
// answer that landed since the last frame applies first.
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
    let mode = wiki::with_viewer(|viewer| viewer.mode.clone());
    // The title row: the page's own title on the page, the wiki's name
    // elsewhere — left at the padding, over the hairline that spans the
    // padded width. A failed fetch names itself in red, where the accent
    // would sit.
    let (title, title_rgb) = match &mode {
        wiki::Mode::Page { title, failed, .. } => (
            title.clone(),
            if *failed {
                wiki::FAILURE
            } else {
                wiki::ACCENT
            },
        ),
        _ => ("Minecraft Wiki".to_string(), wiki::TEXT),
    };
    game.native_text(batch, &title, [px + wiki::PANEL_PAD, py + 10.0], title_rgb, false);
    game.native_fill(
        batch,
        [
            px + wiki::PANEL_PAD,
            py + 24.0,
            pw - wiki::PANEL_PAD * 2.0,
            1.0,
        ],
        wiki::HAIRLINE,
    );
    match mode {
        // The search field: a flat inset with the accent's own hairline for
        // its bottom edge (the focused cue), the query with its caret (an
        // underscore dropped into the text where the caret sits) or the
        // placeholder.
        wiki::Mode::Search { query } => {
            let (fx, fy, fw, fh) = (
                px + wiki::PANEL_PAD,
                py + 32.0,
                pw - wiki::PANEL_PAD * 2.0,
                26.0,
            );
            game.native_fill(batch, [fx, fy, fw, fh], wiki::SURFACE_SUNK);
            game.native_fill(batch, [fx, fy + fh - 1.0, fw, 1.0], wiki::ACCENT);
            if query.value.is_empty() {
                game.native_text(
                    batch,
                    "What are you looking for?",
                    [fx + 8.0, fy + 9.0],
                    wiki::TEXT_DIM,
                    false,
                );
            } else {
                // The caret sits where the box's cursor does: the text
                // split around an underscore, character-counted like the
                // cursor is.
                let head: String = query.value.chars().take(query.cursor).collect();
                let tail: String = query.value.chars().skip(query.cursor).collect();
                let shown = head + "_" + &tail;
                game.native_text(batch, &shown, [fx + 8.0, fy + 9.0], wiki::TEXT, true);
            }
            game.native_text(
                batch,
                "Enter: search the wiki   Esc: close",
                [px + wiki::PANEL_PAD, py + ph - 18.0],
                wiki::TEXT_DIM,
                false,
            );
        }
        // The wait: what the worker is after, at the content's top.
        wiki::Mode::Loading { note } => {
            game.native_text(
                batch,
                &note,
                [px + wiki::PANEL_PAD, py + 36.0],
                wiki::TEXT,
                true,
            );
            game.native_text(
                batch,
                "Esc: close",
                [px + wiki::PANEL_PAD, py + ph - 18.0],
                wiki::TEXT_DIM,
                false,
            );
        }
        // The results: the ranked titles the wiki found, the selection
        // washed in the accent, kept three rows from the window's top edge.
        wiki::Mode::Results { titles, selected } => {
            let shown = ((ph - 64.0) / 12.0).floor().max(1.0) as usize;
            let start = selected
                .saturating_sub(3)
                .min(titles.len().saturating_sub(shown.min(titles.len())));
            for (row, index) in (start..titles.len().min(start + shown)).enumerate() {
                let y = py + 34.0 + row as f32 * 12.0;
                if index == selected {
                    game.native_fill(
                        batch,
                        [px + 8.0, y - 2.0, pw - 16.0, 12.0],
                        wiki::ACCENT_WASH,
                    );
                }
                game.native_text(
                    batch,
                    &titles[index],
                    [px + wiki::PANEL_PAD, y],
                    if index == selected {
                        wiki::ACCENT
                    } else {
                        wiki::TEXT_DIM
                    },
                    index == selected,
                );
            }
            game.native_text(
                batch,
                &format!(
                    "{} results — Up/Down: pick   Enter: open   Esc: back",
                    titles.len()
                ),
                [px + wiki::PANEL_PAD, py + ph - 18.0],
                wiki::TEXT_DIM,
                false,
            );
        }
        // The page: light text on the dark surface — the section headers in
        // the accent with a hairline rule under them, the subheaders a step
        // in, the body plain — with the scrollbar at the body's right and
        // the browser's three buttons below, with the rects the click
        // helper answers. (No footer hint here: the button row owns the
        // bottom edge.)
        wiki::Mode::Page {
            lines,
            scroll,
            ..
        } => {
            let body = [
                px + wiki::PANEL_PAD,
                py + 32.0,
                pw - wiki::PANEL_PAD * 2.0,
                ph - 64.0,
            ];
            let [bx, by, bw, bh] = body;
            let show = ((bh - 8.0) / wiki::LINE_HEIGHT).floor().max(1.0) as usize;
            let view = show.max(wiki::BODY_LINES);
            for (row, line) in lines.iter().skip(scroll).take(view).enumerate() {
                let y = by + 4.0 + row as f32 * wiki::LINE_HEIGHT;
                match line.kind {
                    wiki::LineKind::Heading => {
                        game.native_text(batch, &line.text, [bx + 8.0, y], wiki::ACCENT, true);
                        game.native_fill(
                            batch,
                            [bx + 8.0, y + 9.0, bw - 16.0, 1.0],
                            wiki::HAIRLINE,
                        );
                    }
                    wiki::LineKind::Subheading => {
                        game.native_text(batch, &line.text, [bx + 16.0, y], wiki::TEXT, true);
                    }
                    wiki::LineKind::Body => {
                        game.native_text(batch, &line.text, [bx + 8.0, y], wiki::TEXT, false);
                    }
                }
            }
            // The scrollbar: a thin track down the body's right, its thumb
            // in the accent where the scroll is, when the text runs past
            // the view.
            let max = wiki::max_scroll(lines.len(), wiki::BODY_LINES);
            if max > 0 {
                let bar_h = bh - 8.0;
                let thumb = (bar_h / max as f32).max(8.0).min(bar_h);
                let at = (bar_h - thumb) * scroll as f32 / max as f32;
                game.native_fill(
                    batch,
                    [bx + bw - 5.0, by + 4.0, 3.0, bar_h],
                    wiki::SURFACE_SUNK,
                );
                game.native_fill(
                    batch,
                    [bx + bw - 5.0, by + 4.0 + at, 3.0, thumb],
                    wiki::ACCENT,
                );
            }
            // The buttons: a flat inset each, a hairline around it (inside
            // the rect, so the click plan's bounds stay exact), the label
            // at the padding.
            for (plan, rect) in wiki::button_rects(panel) {
                let [bx, by, bw, bh] = rect;
                game.native_fill(batch, rect, wiki::SURFACE_SUNK);
                game.native_fill(batch, [bx, by, bw, 1.0], wiki::HAIRLINE);
                game.native_fill(batch, [bx, by + bh - 1.0, bw, 1.0], wiki::HAIRLINE);
                game.native_fill(batch, [bx, by, 1.0, bh], wiki::HAIRLINE);
                game.native_fill(batch, [bx + bw - 1.0, by, 1.0, bh], wiki::HAIRLINE);
                game.native_text(
                    batch,
                    wiki::button_label(plan),
                    [bx + 8.0, by + 6.0],
                    wiki::TEXT,
                    false,
                );
            }
        }
    }
}
