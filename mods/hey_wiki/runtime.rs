// Hey Wiki's brain: pure standard library, no game types, so it compiles and
// tests wherever — as this crate's library and, injected, as a module of the
// game. Every decision the helpers act on lives here, and every one of those
// decisions is tested below: the wiki page a name becomes, the API URLs a
// page and a search fetch, the JSON answers' plain-text parsing, the body's
// word wrapping, which of the crosshair's block and mob the H press opens,
// the search box's editing, the page view's keys, buttons and scroll, and
// the settings the gates re-read.
//
// The wiki itself is fetched natively (the engine's own `ureq` on a worker
// thread, published through a mutex): the page's text renders in the game,
// and the browser stays as the fallback the page view offers.
//
// Clean-room: written for Horizon from the game's own public behavior; it
// copies nothing from the original mod. The concept it re-implements is
// Hey Wiki (github.com/mc-wiki/minecraft-mod-heywiki, LGPL-3.0); this port
// is MIT, like the barrel_roll and elytra_slot ports before it.
//
// Copyright (c) 2026 hakergeniusz. MIT.

/// The mod's registry id: the settings folder, the Options > Mods row and
/// the owner of the mod's own screen (the wiki's viewer).
pub const ID: &str = "hey_wiki";
/// The version the row shows.
pub const VERSION: &str = "0.2.0";
/// The mod's description for its row.
pub const DESCRIPTION: &str = "The Minecraft Wiki, a keypress away: press H to open the \
page of the block, mob or item your crosshair is on (the hovered slot's item \
while a screen is open), Alt+H for the item in your hand, and B for an \
in-game wiki search. Pages render in the game's own viewer — a dark panel, \
the wiki's sections and headers, the full text to scroll; a browser is the \
fallback.";

/// The wiki the mod opens: the official English Minecraft Wiki, a MediaWiki
/// site with the TextExtracts extension the page fetch reads.
pub const BASE_URL: &str = "https://minecraft.wiki/";
/// The wiki's MediaWiki API, the page and search fetches' endpoint.
pub const API_URL: &str = "https://minecraft.wiki/api.php";

/// The mod's setting files (one file per key under the mod's data dir),
/// which every gate re-reads: a change lands within the tick that follows it.
pub const SETTING_ENABLED: &str = "enabled";
pub const SETTING_NATIVE: &str = "native-view";
pub const SETTING_CONFIRM: &str = "confirm-open";
pub const SETTING_REACH: &str = "reach";

/// The keybind's setting names (the Options > Mods row's keybind slots) and
/// the keys out of the box. B is unbound in the game; H carries only the
/// debug advanced-tooltips toggle, which the debug screen owns.
pub const OPEN_KEYBIND: &str = "open";
pub const SEARCH_KEYBIND: &str = "search";
pub const DEFAULT_OPEN_KEY: &str = "key.keyboard.h";
pub const DEFAULT_SEARCH_KEY: &str = "key.keyboard.b";

/// How far the crosshair looks (blocks): the H target must be within this.
/// The game's own reach highlights a little under 5 blocks in survival; the
/// original's default lets H answer a little past it.
pub const DEFAULT_REACH: f32 = 5.2;
/// What clicking the Reach row cycles through, nearest first.
pub const REACH_STEPS: [f32; 4] = [3.0, 5.2, 8.0, 12.0];

/// The player inventory's slots: 0..9 hotbar, 9..36 main; the off hand is
/// the game's own slot 40 (`shield::OFFHAND_SLOT`), and the inventory menu's
/// armor slots are the game's 500..503 (`["head","chest","legs","feet"]`).
pub const MAX_SLOT: usize = 36;
pub const OFFHAND_SLOT: usize = 40;
pub const ARMOR_BASE: usize = 500;
pub const ARMOR_SLOTS: usize = 4;

/// The page view's box, in GUI pixels: the panel's size, the text's wrap
/// width in characters (the font averages 6 px a glyph), the lines' height
/// and how many lines the body's window holds.
pub const PANEL_W: f32 = 400.0;
pub const PANEL_H: f32 = 250.0;
pub const WRAP_WIDTH: usize = 62;
pub const LINE_HEIGHT: f32 = 10.0;
pub const BODY_LINES: usize = 15;
/// The page view's buttons, bottom right of the panel: a piece's size.
pub const BUTTON_W: f32 = 110.0;
pub const BUTTON_H: f32 = 18.0;
/// The panel's padding: how far the frame's content — the title row, the
/// search field, the page's body, the footer hints — sits from the panel's
/// edges. The results' rows and the buttons keep their own, pinned geometry
/// (the click plan answers those exact rects).
pub const PANEL_PAD: f32 = 14.0;

/// The viewer's palette, the Ore-UI look: one dark surface, one accent,
/// hairline separators, two text tones. Flat fills only — no gradients,
/// no borders but the hairlines. The fills read `argb` (alpha first); the
/// text helper takes the same shape and reads only its lower three bytes.
pub const SURFACE: u32 = 0xF00E1116;
/// The world behind the panel, dimmed the way a screen does.
pub const SURFACE_DIM: u32 = 0x90000000;
/// A step lighter than the surface: the search field, the buttons, the
/// scrollbar's track.
pub const SURFACE_SUNK: u32 = 0xFF161C22;
/// The 1 px separators: under the title row, under the section headers,
/// around the buttons.
pub const HAIRLINE: u32 = 0xFF27303A;
/// The one accent: the page's title and section headers, the field's
/// underline, the results' selection, the scrollbar's thumb.
pub const ACCENT: u32 = 0xFF35A7CC;
/// The accent, washed flat over a row for the results' selection.
pub const ACCENT_WASH: u32 = 0x2E35A7CC;
/// The text the user reads: body, labels, the search query.
pub const TEXT: u32 = 0xFFE7EBEE;
/// The text that waits: hints, the placeholder, unselected results.
pub const TEXT_DIM: u32 = 0xFF8E979F;
/// A failed fetch's title, where the accent would sit.
pub const FAILURE: u32 = 0xFFE0655A;

/// Whether the mod is on. A missing file means ON — the mod exists for the
/// keys, so installing it is asking for it; anything but `false` in the file
/// is on, trimmed like the game reads its settings.
pub fn setting_on(get: impl FnOnce(&str) -> Option<String>) -> bool {
    !matches!(get(SETTING_ENABLED).as_deref(), Some(v) if v.trim() == "false")
}

/// Whether pages show in the game's own screen. A missing file means ON —
/// the native view is the point of the port; `false` falls back to the
/// original's browser flow (the game's own link confirmation, then the
/// browser).
pub fn native_view(get: impl FnOnce(&str) -> Option<String>) -> bool {
    !matches!(get(SETTING_NATIVE).as_deref(), Some(v) if v.trim() == "false")
}

/// Whether the browser opens only after the confirmation (the game's own
/// open-or-copy link dialog). A missing file means ON, the original's
/// default: an untrusted page should not open unasked.
pub fn confirm_open(get: impl FnOnce(&str) -> Option<String>) -> bool {
    !matches!(get(SETTING_CONFIRM).as_deref(), Some(v) if v.trim() == "false")
}

/// The crosshair's reach from the `reach` file: a decimal of blocks. A
/// missing or unreadable file is the default; anything outside 1..=32 is
/// clamped, so a stray edit cannot make the ray run to the horizon.
pub fn reach_of(get: impl FnOnce(&str) -> Option<String>) -> f32 {
    get(SETTING_REACH)
        .and_then(|v| v.trim().parse::<f32>().ok())
        .map_or(DEFAULT_REACH, |r| r.clamp(1.0, 32.0))
}

/// What to write for a reach: one decimal place, the way the page shows it.
pub fn reach_text(reach: f32) -> String {
    format!("{:.1}", reach)
}

/// What one click on the Reach row changes it to: the next preset up,
/// wrapping to the smallest once past the largest. A reach the file set by
/// hand lands on the first preset above it.
pub fn reach_cycle(reach: f32) -> f32 {
    REACH_STEPS
        .iter()
        .copied()
        .find(|&step| step > reach + f32::EPSILON)
        .unwrap_or(REACH_STEPS[0])
}

// ===== Names and URLs =====

/// The wiki page a registry id names, as the wiki's search best takes it:
/// the namespace goes, the snake_case becomes Title Words ("minecraft:
/// wither_skeleton" searches "Wither Skeleton"). Nothing to name — an empty
/// or namespace-only id — is no page.
pub fn wiki_title(id: &str) -> Option<String> {
    let tail = id.split_once(':').map_or(id, |(_, tail)| tail);
    if tail.is_empty() {
        return None;
    }
    let title = tail
        .split('_')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    (!title.is_empty()).then_some(title)
}

/// Percent-encoding for a URL's query value: spaces become `+`, everything
/// beyond letters, digits and the safe `-` and `.` is percent-encoded UTF-8.
pub fn encoded(query: &str) -> String {
    let mut url = String::new();
    for ch in query.chars() {
        match ch {
            ' ' => url.push('+'),
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '.' => url.push(ch),
            _ => {
                let mut buf = [0u8; 4];
                for byte in ch.encode_utf8(&mut buf).as_bytes() {
                    url.push('%');
                    url.push_str(&format!("{:02X}", byte));
                }
            }
        }
    }
    url
}

/// The URL a wiki search opens in the browser (`?search=`). Nothing to
/// search is no URL.
pub fn search_url(query: &str) -> Option<String> {
    let query = query.trim();
    (!query.is_empty()).then(|| format!("{BASE_URL}?search={}", encoded(query)))
}

/// The URL a wiki page opens in the browser (`/wiki/Name`): the page's
/// canonical title (what the API answered, redirects resolved), spaces back
/// to underscores. No title is no URL.
pub fn page_url(title: &str) -> Option<String> {
    let title = title.trim();
    (!title.is_empty()).then(|| format!("{BASE_URL}wiki/{}", encoded(title)))
}

/// The URL the H press' browser fallback opens for a registry id: its
/// title, searched.
pub fn page_search_url(id: &str) -> Option<String> {
    wiki_title(id).and_then(|title| search_url(&title))
}

/// The MediaWiki API URL a page's text fetches: the plain-text extract of
/// the page, redirects resolved to the canonical title
/// (`action=query&prop=extracts&explaintext=1&redirects=1`).
pub fn api_extract_url(title: &str) -> Option<String> {
    let title = title.trim();
    (!title.is_empty()).then(|| {
        format!(
            "{API_URL}?action=query&format=json&prop=extracts&explaintext=1&redirects=1&titles={}",
            encoded(title)
        )
    })
}

/// The MediaWiki API URL a search fetches: the titles the wiki finds
/// (`action=query&list=search`).
pub fn api_search_url(query: &str) -> Option<String> {
    let query = query.trim();
    (!query.is_empty()).then(|| {
        format!(
            "{API_URL}?action=query&format=json&list=search&srsearch={}",
            encoded(query)
        )
    })
}

// ===== The API answers, as plain text =====

/// A JSON string literal's decoded content, read from just after its
/// opening quote up to the closing one: the standard escapes, `\/`, and
/// `\uXXXX` (surrogate pairs folded). The answers are MediaWiki's, whose
/// strings hold titles and page text.
fn json_string(body: &str) -> String {
    let mut out = String::new();
    let mut chars = body.chars();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => break,
            '\\' => match chars.next() {
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some('/') => out.push('/'),
                Some('b') => out.push('\u{8}'),
                Some('f') => out.push('\u{c}'),
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('u') => {
                    let hex: String = chars.by_ref().take(4).collect();
                    let Ok(unit) = u16::from_str_radix(&hex, 16) else {
                        continue;
                    };
                    // A high surrogate waits for its low half; a lone one
                    // stands alone, as JSON allows.
                    if !(0xD800..0xDC00).contains(&unit) {
                        if let Some(ch) = char::from_u32(u32::from(unit)) {
                            out.push(ch);
                        }
                        continue;
                    }
                    let rest: String = chars.clone().take(6).collect();
                    if let Some(low) = rest
                        .strip_prefix("\\u")
                        .and_then(|hex| u16::from_str_radix(hex, 16).ok())
                        .filter(|low| (0xDC00..0xE000).contains(low))
                    {
                        for _ in 0..6 {
                            chars.next();
                        }
                        let combined = 0x10000
                            + (u32::from(unit) - 0xD800) * 0x400
                            + (u32::from(low) - 0xDC00);
                        if let Some(ch) = char::from_u32(combined) {
                            out.push(ch);
                        }
                    }
                }
                Some(other) => out.push(other),
                None => break,
            },
            other => out.push(other),
        }
    }
    out
}

/// The string a `"key":"value"` holds, the first such pair in the body. The
/// value's opening quote is part of the pattern, so text that merely names
/// the key cannot answer in its place.
fn json_value_of(body: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{key}\":\"");
    let at = body.find(&pattern)? + pattern.len();
    Some(json_string(&body[at..]))
}

/// The titles a search answer holds (`query.search[].title`), in the order
/// the wiki ranked them. The array's end (`"searchinfo"`) stops the scan.
pub fn parse_search_titles(body: &str) -> Vec<String> {
    let Some(rest) = body.split("\"search\":[").nth(1) else {
        return Vec::new();
    };
    let array = rest.split("\"searchinfo\"").next().unwrap_or(rest);
    array
        .split("\"title\":\"")
        .skip(1)
        .map(json_string)
        .collect()
}

/// What a page fetch answers: the page's canonical title (redirects
/// resolved) and its plain text, or nothing when the wiki has no page by
/// that name (the answer carries `"missing"` instead of an extract).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Extract {
    pub title: String,
    pub body: String,
}

pub fn parse_extract(body: &str) -> Option<Extract> {
    let pages = body.split("\"pages\":").nth(1)?;
    if pages.contains("\"missing\"") {
        return None;
    }
    let title = json_value_of(pages, "title")?;
    let body = json_value_of(pages, "extract")?;
    (!body.is_empty()).then_some(Extract { title, body })
}

/// What a page's text becomes for the view: word-wrapped at `width`
/// characters (a word longer than the width hard-splits), the page's own
/// blank lines kept as the paragraphs' spacing.
pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        if paragraph.trim().is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            let mut rest = word;
            while !rest.is_empty() {
                let len = line.chars().count();
                if len == 0 {
                    if rest.chars().count() <= width {
                        line.push_str(rest);
                        break;
                    }
                    // The word alone overflows the line: ship what fits.
                    let head: String = rest.chars().take(width).collect();
                    rest = &rest[head.len()..];
                    lines.push(head);
                } else if rest.chars().count() < width - len {
                    line.push(' ');
                    line.push_str(rest);
                    break;
                } else {
                    lines.push(std::mem::take(&mut line));
                }
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
    }
    lines
}

// ===== The page as the wiki lays it out =====

/// One drawn line of a page, with the wiki look it draws as: the extract's
/// own `== Section ==` markers become the wiki's headers, the rest is body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageLine {
    pub text: String,
    pub kind: LineKind,
}

/// How a page's line reads: a `== Section ==` header (the wiki draws a rule
/// under those), a deeper `=== Sub ===` header, or body text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineKind {
    Heading,
    Subheading,
    Body,
}

/// The wiki header a line names: `== Section ==` is level 2 (the page title
/// is 1 and never appears in an extract's body), `=== Sub ===` is 3 and
/// deeper, with the same number of `=` closing as opening. Not a header: none.
pub fn heading_of(line: &str) -> Option<(usize, &str)> {
    let level = line.chars().take_while(|&c| c == '=').count();
    if level < 2 {
        return None;
    }
    let closing = line.chars().rev().take_while(|&c| c == '=').count();
    if closing != level || line.len() < level * 2 {
        return None;
    }
    let inner = line[level..line.len() - level].trim();
    (!inner.is_empty()).then_some((level, inner))
}

/// What a page's text becomes for the view: the wiki's layout — each
/// `== Section ==` (and deeper) its own header lines, everything else body —
/// word-wrapped at `width` characters. Blank lines carry no drawing of their
/// own; the kinds space the page.
pub fn wrap_page(text: &str, width: usize) -> Vec<PageLine> {
    let mut out = Vec::new();
    for paragraph in text.split('\n') {
        let trimmed = paragraph.trim();
        if trimmed.is_empty() {
            continue;
        }
        match heading_of(trimmed) {
            Some((level, title)) => {
                let kind = if level >= 3 {
                    LineKind::Subheading
                } else {
                    LineKind::Heading
                };
                for line in wrap_text(title, width) {
                    out.push(PageLine { text: line, kind });
                }
            }
            None => {
                for line in wrap_text(trimmed, width) {
                    out.push(PageLine {
                        text: line,
                        kind: LineKind::Body,
                    });
                }
            }
        }
    }
    out
}

// ===== The crosshair target =====

/// Which crosshair target an H press opens: the mob only when its hit is
/// strictly nearer than the block's (`GameRenderer.pick`: the block the ray
/// meets bounds the entity ray, so a wall between hides the mob), else the
/// block, else nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Which {
    Block,
    Mob(usize),
}

pub fn resolve_target(block: Option<f64>, mob: Option<(usize, f64)>) -> Option<Which> {
    match (block, mob) {
        (None, None) => None,
        (None, Some((index, _))) => Some(Which::Mob(index)),
        (Some(_), None) => Some(Which::Block),
        (Some(block_t), Some((index, mob_t))) if mob_t < block_t => Some(Which::Mob(index)),
        (Some(_), Some(_)) => Some(Which::Block),
    }
}

/// Where an H press looks: the item in hand under Alt, the hovered slot's
/// item while a screen is open (the plain survival inventory only — a
/// container's slots number a chest's grid, not the player's), else the
/// crosshair over the world.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Crosshair,
    Hovered,
    Held,
    Nothing,
}

pub fn lookup_focus(screen_open: bool, plain_inventory: bool, alt: bool) -> Focus {
    if alt {
        Focus::Held
    } else if screen_open {
        if plain_inventory {
            Focus::Hovered
        } else {
            Focus::Nothing
        }
    } else {
        Focus::Crosshair
    }
}

/// The slot the plain inventory's hover names, as the game's own slot ids
/// number it: the player's 36, the off hand at 40, the armor at 500..503.
/// Anything else — a container's grid, the creative tabs' display — is no
/// player item to look up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoverSlot {
    Inventory(usize),
    Offhand,
    Armor(usize),
}

pub fn hover_slot(slot: usize) -> Option<HoverSlot> {
    if slot < MAX_SLOT {
        Some(HoverSlot::Inventory(slot))
    } else if slot == OFFHAND_SLOT {
        Some(HoverSlot::Offhand)
    } else if (ARMOR_BASE..ARMOR_BASE + ARMOR_SLOTS).contains(&slot) {
        Some(HoverSlot::Armor(slot - ARMOR_BASE))
    } else {
        None
    }
}

// ===== The keys =====

/// What a key press means to the mod. `None` leaves it to the game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Press {
    None,
    /// H: the wiki page of what H looks up (see `lookup_focus`).
    Lookup,
    /// Alt+H: the item in the main hand.
    Held,
    /// B: the wiki screen.
    Search,
}

/// Whether a key event is the mod's: the key went down (releases are
/// nothing), the mod is on, the player is playing (no menu owns the
/// keyboard), the mod's own screen is not (its box owns the keyboard) and
/// the key is one of the mod's two, as the settings hold them.
#[allow(clippy::too_many_arguments)]
pub fn route(
    pressed: bool,
    enabled: bool,
    playing: bool,
    own_screen: bool,
    key: &str,
    open_key: &str,
    search_key: &str,
    alt: bool,
) -> Press {
    if !pressed || !enabled || !playing || own_screen {
        return Press::None;
    }
    if key == open_key {
        if alt { Press::Held } else { Press::Lookup }
    } else if key == search_key {
        Press::Search
    } else {
        Press::None
    }
}

/// The screen's keys, as the engine names them (`keybinds::key_name`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchKey {
    Enter,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Other,
}

pub fn search_key_of(name: &str) -> SearchKey {
    match name {
        "key.keyboard.enter" | "key.keyboard.keypad.enter" => SearchKey::Enter,
        "key.keyboard.backspace" => SearchKey::Backspace,
        "key.keyboard.delete" => SearchKey::Delete,
        "key.keyboard.left" => SearchKey::Left,
        "key.keyboard.right" => SearchKey::Right,
        "key.keyboard.up" => SearchKey::Up,
        "key.keyboard.down" => SearchKey::Down,
        "key.keyboard.home" => SearchKey::Home,
        "key.keyboard.end" => SearchKey::End,
        _ => SearchKey::Other,
    }
}

// ===== The wiki screen =====

/// What the native view's worker thread fetches: a page's text or a search's
/// titles, as the URL it was built for says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FetchKind {
    Page,
    Search,
}

/// What a fetch came home with: a page's canonical title and text, the
/// search's ranked titles, or the failure (the name and URL stay, so the
/// view's browser fallback and copy still work).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Page(Extract),
    Results {
        query: String,
        titles: Vec<String>,
    },
    Failed {
        kind: FetchKind,
        name: String,
        url: String,
    },
}

/// What an HTTP worker answered with: the body, or nothing (any network or
/// parse failure is the same nothing to the fetch's caller).
pub type HttpResult = Option<String>;

/// The wiki screen's modes: the search box, the results a search found, the
/// page's text (with its browser URL, for the buttons), or the wait between.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    Search {
        query: Search,
    },
    Results {
        titles: Vec<String>,
        selected: usize,
    },
    Loading {
        /// What the wait is for, shown over the view it came from.
        note: String,
    },
    Page {
        title: String,
        /// The browser URL the buttons open and copy.
        url: String,
        /// The wrapped body, one screen line a piece, each with the wiki
        /// look it draws as ([`LineKind`]).
        lines: Vec<PageLine>,
        scroll: usize,
        /// Why the text is not what the page said (a fetch failure), if so.
        failed: bool,
    },
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Search {
            query: Search::new(),
        }
    }
}

/// The wiki screen: the mode and, while a worker runs, what it is after.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Viewer {
    pub mode: Mode,
    /// The fetch in flight: its kind and the name it went out for, so the
    /// answer knows where it belongs.
    pub pending: Option<(FetchKind, String)>,
}

/// What a key on the wiki screen asks for, as the mode answers it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Keyed {
    /// The screen keeps the key (the game's own handling is canceled).
    Kept,
    /// The box's query is submitted (Enter on the search).
    Submit(String),
    /// The results' selected title is opened (Enter on the results).
    Open(String),
    /// The key is not the screen's (Escape and the letters go on, so the
    /// engine's own Escape still closes the screen).
    Nothing,
}

impl Viewer {
    /// The screen opens: the search box, empty.
    pub fn new() -> Self {
        Self::default()
    }

    /// A fetch is on its way: the view waits, saying for what.
    pub fn start(&mut self, kind: FetchKind, name: &str) {
        self.pending = Some((kind, name.to_string()));
        self.mode = Mode::Loading {
            note: match kind {
                FetchKind::Page => format!("Loading {}…", name),
                FetchKind::Search => format!("Searching for {}…", name),
            },
        };
    }

    /// A fetch's answer, applied: a page wraps into the view, a search's
    /// titles become the results, a failure goes back to where it started
    /// with its note (the browser fallback and the copy stay available).
    pub fn apply(&mut self, outcome: Outcome) {
        let Some((kind, name)) = self.pending.take() else {
            return;
        };
        match outcome {
            Outcome::Page(extract) => {
                self.mode = Mode::Page {
                    url: page_url(&extract.title).unwrap_or_default(),
                    title: extract.title,
                    lines: wrap_page(&extract.body, WRAP_WIDTH),
                    scroll: 0,
                    failed: false,
                };
            }
            Outcome::Results { titles, .. } => {
                self.mode = Mode::Results {
                    titles,
                    selected: 0,
                };
            }
            Outcome::Failed { .. } => match kind {
                FetchKind::Page => {
                    self.mode = Mode::Page {
                        url: page_url(&name).unwrap_or_default(),
                        title: name,
                        lines: wrap_page(
                            "The wiki has no page to show here, or it could not be \
                             fetched. The buttons below still open it or copy its \
                             address.",
                            WRAP_WIDTH,
                        ),
                        scroll: 0,
                        failed: true,
                    };
                }
                FetchKind::Search => {
                    // The failed query is back in the box, to edit and retry.
                    let mut query = Search::new();
                    for ch in name.chars() {
                        query.insert(ch);
                    }
                    self.mode = Mode::Search { query };
                }
            },
        }
    }

    /// A key's meaning on the wiki screen, as the mode answers it: the box
    /// edits (kept), Enter submits or opens, the results' arrows pick and
    /// the page's scroll (kept). Escape and the letters are never the
    /// screen's: the engine's own Escape closes it.
    pub fn key(&mut self, key: SearchKey) -> Keyed {
        match &mut self.mode {
            Mode::Search { query } => match key {
                SearchKey::Enter => Keyed::Submit(query.take()),
                other if search_edit(query, other) => Keyed::Kept,
                _ => Keyed::Nothing,
            },
            Mode::Results { titles, selected } => match key {
                SearchKey::Up => {
                    *selected = selected.saturating_sub(1);
                    Keyed::Kept
                }
                SearchKey::Down => {
                    if *selected + 1 < titles.len() {
                        *selected += 1;
                    }
                    Keyed::Kept
                }
                SearchKey::Enter => titles
                    .get(*selected)
                    .cloned()
                    .map_or(Keyed::Nothing, Keyed::Open),
                _ => Keyed::Nothing,
            },
            Mode::Loading { .. } => Keyed::Nothing,
            Mode::Page { lines, scroll, .. } => match key {
                SearchKey::Up => {
                    *scroll = scroll.saturating_sub(3);
                    Keyed::Kept
                }
                SearchKey::Down => {
                    *scroll = (*scroll + 3).min(max_scroll(lines.len(), BODY_LINES));
                    Keyed::Kept
                }
                _ => Keyed::Nothing,
            },
        }
    }

    /// The wheel turned over the page: three lines a notch, clamped to the
    /// text. Away from the page, nothing.
    pub fn scroll(&mut self, notch: f64) {
        let Mode::Page { lines, scroll, .. } = &mut self.mode else {
            return;
        };
        *scroll = if notch > 0.0 {
            scroll.saturating_sub(3)
        } else {
            (*scroll + 3).min(max_scroll(lines.len(), BODY_LINES))
        };
    }

    /// How far the page's text may scroll: the lines past the view's own.
    pub fn max_scroll(&self) -> usize {
        match &self.mode {
            Mode::Page { lines, .. } => max_scroll(lines.len(), BODY_LINES),
            _ => 0,
        }
    }

    /// Back to the search box (the Back button, or what a page's Escape
    /// could ask): the box starts over.
    pub fn go_back(&mut self) {
        self.mode = Mode::default();
    }

    /// The page's browser URL, for the buttons.
    pub fn page_url(&self) -> Option<String> {
        match &self.mode {
            Mode::Page { url, .. } if !url.is_empty() => Some(url.clone()),
            _ => None,
        }
    }

    /// A results row's title, for its click.
    pub fn result_title(&self, index: usize) -> Option<String> {
        match &self.mode {
            Mode::Results { titles, .. } => titles.get(index).cloned(),
            _ => None,
        }
    }

    /// Whether the view waits on a worker.
    pub fn waiting(&self) -> bool {
        self.pending.is_some()
    }
}

/// How far a body of `total` lines scrolls, `visible` at a time.
pub fn max_scroll(total: usize, visible: usize) -> usize {
    total.saturating_sub(visible)
}

// ===== The search box =====

/// The search box: the query and the caret's place in it, counted in
/// characters (a glyph may be several bytes).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Search {
    pub value: String,
    pub cursor: usize,
}

impl Search {
    pub const fn new() -> Self {
        Self {
            value: String::new(),
            cursor: 0,
        }
    }

    /// A typed character lands at the caret.
    pub fn insert(&mut self, ch: char) {
        let at = self
            .value
            .char_indices()
            .nth(self.cursor)
            .map_or(self.value.len(), |(i, _)| i);
        self.value.insert(at, ch);
        self.cursor += 1;
    }

    /// The character before the caret goes, the caret with it.
    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let at = self
            .value
            .char_indices()
            .nth(self.cursor - 1)
            .map_or(self.value.len(), |(i, _)| i);
        let end = self.cursor_byte();
        self.value.replace_range(at..end, "");
        self.cursor -= 1;
    }

    /// The character at the caret goes, the caret stays.
    pub fn delete(&mut self) {
        let at = self.cursor_byte();
        if at >= self.value.len() {
            return;
        }
        let end = self
            .value
            .char_indices()
            .nth(self.cursor + 1)
            .map_or(self.value.len(), |(i, _)| i);
        self.value.replace_range(at..end, "");
    }

    fn cursor_byte(&self) -> usize {
        self.value
            .char_indices()
            .nth(self.cursor)
            .map_or(self.value.len(), |(i, _)| i)
    }

    pub fn left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.value.chars().count());
    }

    pub fn home(&mut self) {
        self.cursor = 0;
    }

    pub fn end(&mut self) {
        self.cursor = self.value.chars().count();
    }

    /// The whole box, emptied — what Enter submits and closing forgets.
    pub fn take(&mut self) -> String {
        self.cursor = 0;
        std::mem::take(&mut self.value)
    }
}

/// The editing keys' effect on the box: true when the key was the box's
/// (the screen keeps it), false when it was not. Enter and the arrows are
/// the mode's, not the box's.
pub fn search_edit(view: &mut Search, key: SearchKey) -> bool {
    match key {
        SearchKey::Backspace => {
            view.backspace();
            true
        }
        SearchKey::Delete => {
            view.delete();
            true
        }
        SearchKey::Left => {
            view.left();
            true
        }
        SearchKey::Right => {
            view.right();
            true
        }
        SearchKey::Home => {
            view.home();
            true
        }
        SearchKey::End => {
            view.end();
            true
        }
        _ => false,
    }
}

// ===== The flows, as the settings route them =====

/// What the H press does with its answer, as the native-view setting sends
/// it: the page fetched into the game's own screen, or the original's
/// browser flow behind the game's link confirmation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HPlan {
    Native(String),
    Browser(String),
    Nothing,
}

pub fn h_plan(native: bool, id: Option<String>) -> HPlan {
    match (native, id) {
        (true, Some(id)) => wiki_title(&id).map_or(HPlan::Nothing, HPlan::Native),
        (false, Some(id)) => page_search_url(&id).map_or(HPlan::Nothing, HPlan::Browser),
        (_, None) => HPlan::Nothing,
    }
}

/// What the search screen's Enter does with the box: the wiki's own search
/// fetched into the results view, or the browser at the search URL.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubmitPlan {
    Native(String),
    Browser(String),
    Nothing,
}

pub fn submit_plan(native: bool, query: &str) -> SubmitPlan {
    let query = query.trim();
    if query.is_empty() {
        return SubmitPlan::Nothing;
    }
    if native {
        SubmitPlan::Native(query.to_string())
    } else {
        search_url(query).map_or(SubmitPlan::Nothing, SubmitPlan::Browser)
    }
}

/// Whether the browser opens behind the game's own link confirmation (Open /
/// Copy link / Cancel) or straight away. The confirmation is the original's
/// default behavior and the game's own dialog for links.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenPlan {
    Prompt,
    Direct,
}

pub fn open_plan(confirm: bool) -> OpenPlan {
    if confirm {
        OpenPlan::Prompt
    } else {
        OpenPlan::Direct
    }
}

// ===== The pointer, playing =====

/// Whether a playing window takes its free pointer back: the world is live
/// (`playing`), the window has focus, the view owns keyboard and mouse, the
/// pointer is free, and nothing sits over gameplay (`over_gameplay` — a
/// prompt, a dialog, a book, anything the cursor answers). The engine
/// re-grabs on a screen close and on a refocus that lost a held pointer,
/// but a browser round-trip hands the focus over while the pointer is
/// already free (the link prompt answers with the cursor), so nothing
/// re-grabs on the way back; the tick re-asserts what playing means.
pub fn should_regrab(
    playing: bool,
    focused: bool,
    keyboard_mouse: bool,
    captured: bool,
    over_gameplay: bool,
) -> bool {
    playing && focused && keyboard_mouse && !captured && !over_gameplay
}

// ===== The clicks =====

/// A click's plan on the wiki screen, from where it landed: the page's
/// buttons (browser, copy, back), a result's title, or nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClickPlan {
    None,
    /// Open the page's URL in the browser (the fallback).
    Browser,
    /// Put the page's URL on the clipboard.
    Copy,
    /// Back to the search box.
    Back,
    /// A result's title was clicked: its place in the list.
    Result(usize),
}

/// The page view's three buttons, bottom right of the panel: Browser, Copy
/// link and Back, left to right.
pub fn button_rects(panel: [f32; 4]) -> [(ClickPlan, [f32; 4]); 3] {
    let y = panel[1] + panel[3] - BUTTON_H - 6.0;
    let right = panel[0] + panel[2] - 8.0;
    let browser = [right - 3.0 * BUTTON_W - 2.0 * 4.0, y, BUTTON_W, BUTTON_H];
    let copy = [browser[0] + BUTTON_W + 4.0, y, BUTTON_W, BUTTON_H];
    let back = [copy[0] + BUTTON_W + 4.0, y, BUTTON_W, BUTTON_H];
    [
        (ClickPlan::Browser, browser),
        (ClickPlan::Copy, copy),
        (ClickPlan::Back, back),
    ]
}

/// A button's label, as the view draws it.
pub fn button_label(plan: ClickPlan) -> &'static str {
    match plan {
        ClickPlan::Browser => "Open in browser",
        ClickPlan::Copy => "Copy link",
        ClickPlan::Back => "Back",
        _ => "",
    }
}

/// Whether a click at the cursor lands in the rect.
pub fn hit(rect: [f32; 4], cursor: [f32; 2]) -> bool {
    cursor[0] >= rect[0]
        && cursor[0] < rect[0] + rect[2]
        && cursor[1] >= rect[1]
        && cursor[1] < rect[1] + rect[3]
}

/// What a press on the wiki screen means: the buttons on a page, a result's
/// row (the rows start under the heading, 12 px a line, as drawn), else
/// nothing. The cursor is the screen's, in GUI pixels.
pub fn click_plan(viewer: &Viewer, panel: [f32; 4], cursor: [f32; 2]) -> ClickPlan {
    match &viewer.mode {
        Mode::Page { .. } => button_rects(panel)
            .into_iter()
            .find(|(_, rect)| hit(*rect, cursor))
            .map_or(ClickPlan::None, |(plan, _)| plan),
        Mode::Results { titles, .. } => {
            let index = ((cursor[1] - panel[1] - 34.0) / 12.0).floor();
            if index < 0.0 || cursor[0] < panel[0] + 8.0 || cursor[0] > panel[0] + panel[2] - 8.0 {
                return ClickPlan::None;
            }
            usize::try_from(index as i64)
                .ok()
                .filter(|index| *index < titles.len())
                .map_or(ClickPlan::None, ClickPlan::Result)
        }
        _ => ClickPlan::None,
    }
}

// ===== The settings page =====

/// The settings page's rows, in the state the files hold (read fresh from
/// disk, so the page always shows what the gates act on).
pub fn page_labels(enabled: bool, native: bool, confirm: bool, reach: f32) -> Vec<String> {
    vec![
        format!("Enabled: {}", if enabled { "On" } else { "Off" }),
        format!("Wiki in game: {}", if native { "On" } else { "Off" }),
        format!(
            "Confirm before opening: {}",
            if confirm { "On" } else { "Off" }
        ),
        format!("Reach: {} blocks (click to change)", reach_text(reach)),
    ]
}

/// What one click on the page changes: the toggles toggle, the Reach
/// cycles. Past the four rows, nothing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PageClick {
    None,
    Enabled(bool),
    Native(bool),
    Confirm(bool),
    Reach(f32),
}

pub fn page_click(
    button: usize,
    enabled: bool,
    native: bool,
    confirm: bool,
    reach: f32,
) -> PageClick {
    match button {
        0 => PageClick::Enabled(!enabled),
        1 => PageClick::Native(!native),
        2 => PageClick::Confirm(!confirm),
        3 => PageClick::Reach(reach_cycle(reach)),
        _ => PageClick::None,
    }
}

/// What a page click writes, as setting-file pairs (one write per click:
/// the file holds the newest state and the gates re-read it).
pub fn stored(click: PageClick) -> Option<(&'static str, String)> {
    match click {
        PageClick::None => None,
        PageClick::Enabled(on) => Some((SETTING_ENABLED, on.to_string())),
        PageClick::Native(on) => Some((SETTING_NATIVE, on.to_string())),
        PageClick::Confirm(on) => Some((SETTING_CONFIRM, on.to_string())),
        PageClick::Reach(reach) => Some((SETTING_REACH, reach_text(reach))),
    }
}

// ===== The game's own state =====

// The wiki screen, one per thread (the window's Game drives it; the client
// is single-threaded), and a browser opening the screen helper deferred (it
// holds no `&mut game`): the next tick's helper takes it.
thread_local! {
    static VIEWER: std::cell::RefCell<Viewer> = std::cell::RefCell::new(Viewer::new());
    static BROWSER: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
    static COPY: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// What the workers publish, one at a time (a mutex: the thread is not the
/// game's, and a reader never blocks a writer beyond a move).
static OUTCOME: std::sync::Mutex<Option<Outcome>> = std::sync::Mutex::new(None);

/// The wiki screen, for the draw, key, character, click and scroll helpers.
pub fn with_viewer<R>(f: impl FnOnce(&mut Viewer) -> R) -> R {
    VIEWER.with_borrow_mut(f)
}

/// The screen's box alone (the search mode's text): None away from it.
pub fn with_search<R>(f: impl FnOnce(&mut Search) -> R) -> Option<R> {
    with_viewer(|viewer| match &mut viewer.mode {
        Mode::Search { query } => Some(f(query)),
        _ => None,
    })
}

/// A worker's answer, taken by the frame that draws next.
pub fn take_outcome() -> Option<Outcome> {
    OUTCOME.lock().ok().and_then(|mut held| held.take())
}

/// A URL's fetch on its own thread: the game never waits on the network.
/// The HTTP itself comes in from the game side (the engine's `ureq`), the
/// parse and the publish are the brain's own. A worker that dies (a panic)
/// still answers, as a failure.
pub fn fetch_async(
    kind: FetchKind,
    name: String,
    url: String,
    http: impl FnOnce(&str) -> HttpResult + Send + 'static,
) {
    let work = {
        let name = name.clone();
        let url = url.clone();
        move || match http(&url) {
            Some(body) => parse_outcome(kind, name, url, &body),
            None => Outcome::Failed { kind, name, url },
        }
    };
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work))
        .unwrap_or(Outcome::Failed { kind, name, url });
    publish_outcome(outcome);
}

/// What a body the worker read becomes, by the kind of fetch it was.
fn parse_outcome(kind: FetchKind, name: String, url: String, body: &str) -> Outcome {
    match kind {
        FetchKind::Page => {
            parse_extract(body).map_or(Outcome::Failed { kind, name, url }, Outcome::Page)
        }
        FetchKind::Search => {
            let titles = parse_search_titles(body);
            if titles.is_empty() {
                Outcome::Failed { kind, name, url }
            } else {
                Outcome::Results {
                    query: name,
                    titles,
                }
            }
        }
    }
}

fn publish_outcome(outcome: Outcome) {
    if let Ok(mut held) = OUTCOME.lock() {
        *held = Some(outcome);
    }
}

/// A browser opening the screen helper deferred, taken by the next tick.
pub fn take_browser() -> Option<String> {
    BROWSER.with_borrow_mut(|held| held.take())
}

/// A browser opening the screen helper deferred, set for the next tick.
pub fn set_browser(url: String) {
    BROWSER.with_borrow_mut(|held| *held = Some(url));
}

/// A link the screen helper put aside for the clipboard (it holds no `&mut
/// game`), taken by the next tick.
pub fn take_copy() -> Option<String> {
    COPY.with_borrow_mut(|held| held.take())
}

/// A link the screen helper put aside for the clipboard, set.
pub fn set_copy(url: String) {
    COPY.with_borrow_mut(|held| *held = Some(url));
}

/// A fresh screen when it opens, and a forgotten fetch when it closes:
/// nothing survives to leak into the next opening.
pub fn reset_screen() {
    VIEWER.with_borrow_mut(|viewer| *viewer = Viewer::new());
    BROWSER.with_borrow_mut(|held| held.take());
    COPY.with_borrow_mut(|held| held.take());
    if let Ok(mut held) = OUTCOME.lock() {
        *held = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::{LineKind, PageLine, heading_of, wrap_page};

    const OFF: Option<String> = None;
    fn setting(value: &str) -> impl Fn(&str) -> Option<String> + '_ {
        move |_| Some(value.to_string())
    }

    /// The wiki's own markers become the page's kinds: `== X ==` headers,
    /// `=== X ===` subheaders, everything else body, blanks dropped (the
    /// kinds space the page), and header text wraps as its kind.
    #[test]
    fn the_page_wraps_as_the_wiki_lays_it_out() {
        assert_eq!(heading_of("== Gameplay =="), Some((2, "Gameplay")));
        assert_eq!(heading_of("=== A == B ==="), Some((3, "A == B")));
        assert_eq!(heading_of("== ==="), None, "the marks alone are no header");
        assert_eq!(heading_of("plain text"), None);
        assert_eq!(heading_of("="), None);
        let page = wrap_page(
            "Lead line.\n\n== First ==\nBody one two.\n=== Sub ===\nDeeper.\n",
            62,
        );
        let kinds: Vec<LineKind> = page.iter().map(|l| l.kind).collect();
        assert_eq!(
            kinds,
            vec![
                LineKind::Body,
                LineKind::Heading,
                LineKind::Body,
                LineKind::Subheading,
                LineKind::Body
            ]
        );
        assert_eq!(page[0].text, "Lead line.");
        assert_eq!(page[1].text, "First");
        assert_eq!(page[3].text, "Sub");
        // A header longer than the line wraps as a header, never body.
        let long = wrap_page(
            "== A Very Long Section Title That Wraps Around Well ==\n",
            20,
        );
        assert!(long.iter().all(|l| l.kind == LineKind::Heading));
        assert!(long.len() > 1);
        let empty = wrap_page("\n\n\n", 62);
        assert!(empty.is_empty());
        let _ = PageLine {
            text: String::new(),
            kind: LineKind::Body,
        };
    }

    #[test]
    fn enabled_defaults_on_and_off_sticks() {
        assert!(setting_on(|_| OFF));
        assert!(setting_on(|key| setting("true")(key)));
        assert!(!setting_on(|key| setting("false")(key)));
        assert!(!setting_on(|key| setting(" false \n")(key)));
    }

    #[test]
    fn native_view_defaults_on() {
        assert!(native_view(|_| OFF));
        assert!(native_view(|key| setting("true")(key)));
        assert!(!native_view(|key| setting("false")(key)));
    }

    #[test]
    fn confirm_defaults_on() {
        assert!(confirm_open(|_| OFF));
        assert!(confirm_open(|key| setting("true")(key)));
        assert!(!confirm_open(|key| setting("false")(key)));
    }

    #[test]
    fn reach_defaults_and_clamps() {
        assert_eq!(reach_of(|_| OFF), DEFAULT_REACH);
        assert_eq!(reach_of(|key| setting("4.5")(key)), 4.5);
        assert_eq!(reach_of(|key| setting("nope")(key)), DEFAULT_REACH);
        assert_eq!(reach_of(|key| setting("0.1")(key)), 1.0);
        assert_eq!(reach_of(|key| setting("1000")(key)), 32.0);
    }

    #[test]
    fn reach_cycles_through_the_presets() {
        assert_eq!(reach_cycle(DEFAULT_REACH), 8.0);
        assert_eq!(reach_cycle(8.0), 12.0);
        assert_eq!(reach_cycle(12.0), 3.0);
        assert_eq!(reach_cycle(3.0), 5.2);
        // A hand-edited value lands on the first preset above it.
        assert_eq!(reach_cycle(6.0), 8.0);
    }

    #[test]
    fn reach_renders_one_decimal() {
        assert_eq!(reach_text(5.2), "5.2");
        assert_eq!(reach_text(3.0), "3.0");
    }

    #[test]
    fn ids_become_wiki_titles() {
        assert_eq!(
            wiki_title("minecraft:wither_skeleton").as_deref(),
            Some("Wither Skeleton")
        );
        assert_eq!(wiki_title("oak_stairs").as_deref(), Some("Oak Stairs"));
        assert_eq!(wiki_title("minecraft:tnt").as_deref(), Some("Tnt"));
        assert!(wiki_title("").is_none());
        assert!(wiki_title("minecraft:").is_none());
        assert!(wiki_title("minecraft:__").is_none());
    }

    #[test]
    fn queries_url_encode() {
        assert_eq!(encoded("Wither Skeleton"), "Wither+Skeleton");
        assert_eq!(encoded("oak door!"), "oak+door%21");
        assert_eq!(encoded("дуб"), "%D0%B4%D1%83%D0%B1");
        assert_eq!(encoded("a-b.2"), "a-b.2");
    }

    #[test]
    fn browser_urls() {
        assert_eq!(
            search_url("Wither Skeleton").as_deref(),
            Some("https://minecraft.wiki/?search=Wither+Skeleton")
        );
        assert!(search_url("   ").is_none());
        assert_eq!(
            page_url("Diamond Sword").as_deref(),
            Some("https://minecraft.wiki/wiki/Diamond+Sword")
        );
        assert!(page_url("").is_none());
        assert_eq!(
            page_search_url("minecraft:iron_sword").as_deref(),
            Some("https://minecraft.wiki/?search=Iron+Sword")
        );
    }

    #[test]
    fn api_urls() {
        assert_eq!(
            api_extract_url("Wither Skeleton").as_deref(),
            Some(
                "https://minecraft.wiki/api.php?action=query&format=json&prop=extracts&\
                  explaintext=1&redirects=1&titles=Wither+Skeleton"
            )
        );
        assert_eq!(
            api_search_url("diamond!").as_deref(),
            Some(
                "https://minecraft.wiki/api.php?action=query&format=json&list=search&\
                  srsearch=diamond%21"
            )
        );
        assert!(api_extract_url(" ").is_none());
        assert!(api_search_url("").is_none());
    }

    const SEARCH_BODY: &str = r#"{"batchcomplete":true,"query":{"search":[
        {"ns":0,"title":"Diamond","pageid":263},
        {"ns":0,"title":"Diamond Sword","pageid":522},
        {"ns":0,"title":"Diamond \"Ore\"","pageid":608}
      ],"searchinfo":{"totalhits":331}}}"#;

    #[test]
    fn search_answers_rank_titles() {
        assert_eq!(
            parse_search_titles(SEARCH_BODY),
            vec!["Diamond", "Diamond Sword", "Diamond \"Ore\""]
        );
        assert!(parse_search_titles("{\"query\":{}}").is_empty());
        assert!(parse_search_titles("no json here").is_empty());
    }

    const EXTRACT_BODY: &str = r#"{"batchcomplete":"","query":{"pages":{
        "263":{"pageid":263,"ns":0,"title":"Diamond","extract":"Diamond is a rare mineral.\n\n== Uses ==\nIt is used for tools and \"armor\", emoji \u00e9\u2764 and slash \/ too."}
      }}}"#;

    const MISSING_BODY: &str = r#"{"batchcomplete":"","query":{"pages":{
        "-1":{"ns":0,"title":"Nothing At All","missing":""}
      }}}"#;

    #[test]
    fn page_answers_carry_the_canonical_text() {
        let extract = parse_extract(EXTRACT_BODY).expect("the answer parses");
        assert_eq!(extract.title, "Diamond");
        assert_eq!(
            extract.body,
            "Diamond is a rare mineral.\n\n== Uses ==\nIt is used for tools and \
             \"armor\", emoji é❤ and slash / too."
        );
    }

    #[test]
    fn missing_pages_answer_nothing() {
        assert!(parse_extract(MISSING_BODY).is_none());
        assert!(parse_extract("{}").is_none());
        // An answer with no extract is no page either.
        assert!(parse_extract(r#"{"query":{"pages":{"1":{"title":"X"}}}}"#).is_none());
    }

    #[test]
    fn surrogate_pairs_decode() {
        // U+1F600 (the grinning face) as a high and a low surrogate.
        let body = r#"{"pages":{"1":{"title":"T","extract":"\ud83d\ude00 ok"}}}"#;
        let extract = parse_extract(body).expect("parses");
        assert_eq!(extract.body, "\u{1F600} ok");
    }

    #[test]
    fn text_wraps_at_the_width() {
        let lines = wrap_text("one two three four five six seven", 10);
        assert_eq!(lines, vec!["one two", "three four", "five six", "seven"]);
    }

    #[test]
    fn long_words_hard_split() {
        let lines = wrap_text("extraordinary", 6);
        assert_eq!(lines, vec!["extrao", "rdinar", "y"]);
        // A long word after a full line starts its own hard splits.
        let lines = wrap_text("hi extraordinary", 6);
        assert_eq!(lines, vec!["hi", "extrao", "rdinar", "y"]);
    }

    #[test]
    fn paragraphs_keep_their_blank_lines() {
        let lines = wrap_text("first\n\nsecond", 20);
        assert_eq!(lines, vec!["first", "", "second"]);
    }

    #[test]
    fn unicode_wraps_by_characters() {
        let lines = wrap_text("äöäöäöäöäöäö", 5);
        assert_eq!(lines, vec!["äöäöä", "öäöäö", "äö"]);
    }

    #[test]
    fn the_nearer_target_wins_and_a_wall_hides_the_mob() {
        assert_eq!(resolve_target(None, None), None);
        assert_eq!(resolve_target(Some(4.0), None), Some(Which::Block));
        assert_eq!(resolve_target(None, Some((3, 2.5))), Some(Which::Mob(3)));
        assert_eq!(
            resolve_target(Some(4.0), Some((3, 2.5))),
            Some(Which::Mob(3))
        );
        assert_eq!(
            resolve_target(Some(2.0), Some((3, 2.5))),
            Some(Which::Block)
        );
        // Equal distances are the block's (vanilla favors the block on ties).
        assert_eq!(
            resolve_target(Some(2.5), Some((3, 2.5))),
            Some(Which::Block)
        );
    }

    #[test]
    fn alt_is_the_hand_and_screens_hover() {
        assert_eq!(lookup_focus(false, false, false), Focus::Crosshair);
        assert_eq!(lookup_focus(false, false, true), Focus::Held);
        assert_eq!(lookup_focus(true, true, false), Focus::Hovered);
        // A screen with no player inventory (chat, a chest) is nothing.
        assert_eq!(lookup_focus(true, false, false), Focus::Nothing);
        // Alt wins over the screen: the hand is the hand anywhere.
        assert_eq!(lookup_focus(true, true, true), Focus::Held);
    }

    #[test]
    fn slots_map_to_the_player_inventory() {
        assert_eq!(hover_slot(0), Some(HoverSlot::Inventory(0)));
        assert_eq!(hover_slot(35), Some(HoverSlot::Inventory(35)));
        assert_eq!(hover_slot(OFFHAND_SLOT), Some(HoverSlot::Offhand));
        assert_eq!(hover_slot(ARMOR_BASE), Some(HoverSlot::Armor(0)));
        assert_eq!(hover_slot(ARMOR_BASE + 3), Some(HoverSlot::Armor(3)));
        // A chest's grid and the creative tabs' display are no player slot.
        assert_eq!(hover_slot(36), None);
        assert_eq!(hover_slot(45), None);
        assert_eq!(hover_slot(200), None);
        assert_eq!(hover_slot(504), None);
    }

    #[test]
    fn routes_are_the_mods_keys_only() {
        const H: &str = DEFAULT_OPEN_KEY;
        const B: &str = DEFAULT_SEARCH_KEY;
        let playing = |pressed, key, alt| route(pressed, true, true, false, key, H, B, alt);
        assert_eq!(playing(true, H, false), Press::Lookup);
        assert_eq!(playing(true, H, true), Press::Held);
        assert_eq!(playing(true, B, false), Press::Search);
        assert_eq!(playing(true, "key.keyboard.r", false), Press::None);
        // Releases, a disabled mod, menus and the mod's own screen are none.
        assert_eq!(playing(false, H, false), Press::None);
        assert_eq!(route(true, false, true, false, H, H, B, false), Press::None);
        assert_eq!(route(true, true, false, false, H, H, B, false), Press::None);
        assert_eq!(route(true, true, true, true, H, H, B, false), Press::None);
        // The keys move with the settings.
        assert_eq!(
            route(
                true,
                true,
                true,
                false,
                "key.keyboard.j",
                "key.keyboard.j",
                B,
                false
            ),
            Press::Lookup
        );
    }

    #[test]
    fn screen_keys_are_named() {
        assert_eq!(search_key_of("key.keyboard.enter"), SearchKey::Enter);
        assert_eq!(search_key_of("key.keyboard.keypad.enter"), SearchKey::Enter);
        assert_eq!(
            search_key_of("key.keyboard.backspace"),
            SearchKey::Backspace
        );
        assert_eq!(search_key_of("key.keyboard.delete"), SearchKey::Delete);
        assert_eq!(search_key_of("key.keyboard.left"), SearchKey::Left);
        assert_eq!(search_key_of("key.keyboard.right"), SearchKey::Right);
        assert_eq!(search_key_of("key.keyboard.up"), SearchKey::Up);
        assert_eq!(search_key_of("key.keyboard.down"), SearchKey::Down);
        assert_eq!(search_key_of("key.keyboard.home"), SearchKey::Home);
        assert_eq!(search_key_of("key.keyboard.end"), SearchKey::End);
        assert_eq!(search_key_of("key.keyboard.escape"), SearchKey::Other);
        assert_eq!(search_key_of("key.keyboard.h"), SearchKey::Other);
    }

    #[test]
    fn the_box_edits_like_a_text_field() {
        let mut view = Search::new();
        for ch in "diamond".chars() {
            view.insert(ch);
        }
        assert_eq!(view.value, "diamond");
        assert_eq!(view.cursor, 7);
        // Home, then right three times, then backspace takes the 'a' the
        // caret just passed.
        view.home();
        view.right();
        view.right();
        view.right();
        view.backspace();
        assert_eq!(view.value, "dimond");
        assert_eq!(view.cursor, 2);
        // The caret sits where the 'a' was; one right and delete at the
        // caret takes the 'm' that follows it.
        view.right();
        view.delete();
        assert_eq!(view.value, "dimnd");
        // The caret never leaves the text.
        view.left();
        view.left();
        view.left();
        view.left();
        assert_eq!(view.cursor, 0);
        view.backspace();
        assert_eq!(view.value, "dimnd");
        view.end();
        view.insert('s');
        assert_eq!(view.value, "dimnds");
    }

    #[test]
    fn the_box_holds_multibyte_characters() {
        let mut view = Search::new();
        for ch in "äöü".chars() {
            view.insert(ch);
        }
        assert_eq!(view.value, "äöü");
        assert_eq!(view.cursor, 3);
        view.left();
        view.backspace();
        assert_eq!(view.value, "äü");
        assert_eq!(view.cursor, 1);
    }

    #[test]
    fn editing_keys_are_the_boxes_and_the_rest_is_not() {
        let mut view = Search::new();
        view.insert('a');
        assert!(!search_edit(&mut view, SearchKey::Enter));
        assert!(!search_edit(&mut view, SearchKey::Other));
        for key in [
            SearchKey::Backspace,
            SearchKey::Delete,
            SearchKey::Left,
            SearchKey::Right,
            SearchKey::Home,
            SearchKey::End,
        ] {
            assert!(search_edit(&mut view, key));
        }
    }

    #[test]
    fn h_plans_route_by_the_native_setting() {
        assert_eq!(
            h_plan(true, Some("minecraft:diamond".into())),
            HPlan::Native("Diamond".into())
        );
        assert_eq!(
            h_plan(false, Some("minecraft:diamond".into())),
            HPlan::Browser("https://minecraft.wiki/?search=Diamond".into())
        );
        assert_eq!(h_plan(true, None), HPlan::Nothing);
        assert_eq!(h_plan(true, Some("minecraft:".into())), HPlan::Nothing);
    }

    #[test]
    fn submits_route_by_the_native_setting() {
        assert_eq!(
            submit_plan(true, " diamond sword "),
            SubmitPlan::Native("diamond sword".into())
        );
        assert_eq!(
            submit_plan(false, "diamond sword"),
            SubmitPlan::Browser("https://minecraft.wiki/?search=diamond+sword".into())
        );
        assert_eq!(submit_plan(true, "  "), SubmitPlan::Nothing);
    }

    #[test]
    fn the_confirmation_gates_the_browser() {
        assert_eq!(open_plan(true), OpenPlan::Prompt);
        assert_eq!(open_plan(false), OpenPlan::Direct);
    }

    #[test]
    fn the_free_pointer_of_a_playing_window_is_taken_back() {
        assert!(should_regrab(true, true, true, false, false));
        // A held pointer, an unfocused window, a view that does not own the
        // keyboard, anything over gameplay and anything but playing each
        // keep the tick's hands off.
        assert!(!should_regrab(true, true, true, true, false));
        assert!(!should_regrab(true, false, true, false, false));
        assert!(!should_regrab(true, true, false, false, false));
        assert!(!should_regrab(true, true, true, false, true));
        assert!(!should_regrab(false, true, true, false, false));
    }

    #[test]
    fn the_screen_starts_on_the_search_box() {
        let viewer = Viewer::new();
        assert!(matches!(viewer.mode, Mode::Search { .. }));
        assert!(viewer.pending.is_none());
        assert_eq!(viewer.max_scroll(), 0);
    }

    #[test]
    fn a_page_fetch_loads_then_shows_the_text() {
        let mut viewer = Viewer::new();
        viewer.start(FetchKind::Page, "Wither Skeleton");
        assert!(matches!(viewer.mode, Mode::Loading { .. }));
        viewer.apply(Outcome::Page(Extract {
            title: "Wither Skeleton".into(),
            body: "The wither skeleton is a tall mob.\n\nIt carries a stone sword.".into(),
        }));
        let Mode::Page { title, lines, .. } = &viewer.mode else {
            panic!("the page shows");
        };
        assert_eq!(title, "Wither Skeleton");
        assert_eq!(
            lines,
            &wrap_page(
                "The wither skeleton is a tall mob.\n\nIt carries a stone sword.",
                WRAP_WIDTH
            )
        );
    }

    #[test]
    fn a_failed_page_still_offers_the_browser() {
        let mut viewer = Viewer::new();
        viewer.start(FetchKind::Page, "Wither Skeleton");
        viewer.apply(Outcome::Failed {
            kind: FetchKind::Page,
            name: "Wither Skeleton".into(),
            url: api_extract_url("Wither Skeleton").unwrap(),
        });
        let Mode::Page { url, failed, .. } = &viewer.mode else {
            panic!("the page shows its failure");
        };
        assert!(*failed);
        assert_eq!(url.as_str(), "https://minecraft.wiki/wiki/Wither+Skeleton");
    }

    #[test]
    fn a_failed_search_gives_the_box_back() {
        let mut viewer = Viewer::new();
        viewer.start(FetchKind::Search, "zzz no results");
        viewer.apply(Outcome::Failed {
            kind: FetchKind::Search,
            name: "zzz no results".into(),
            url: String::new(),
        });
        let Mode::Search { query } = &viewer.mode else {
            panic!("the box is back");
        };
        assert_eq!(query.value, "zzz no results");
        assert_eq!(query.cursor, 14);
    }

    #[test]
    fn answers_without_a_fetch_waiting_are_dropped() {
        let mut viewer = Viewer::new();
        viewer.apply(Outcome::Page(Extract::default()));
        assert!(matches!(viewer.mode, Mode::Search { .. }));
    }

    #[test]
    fn search_keys_ask_for_the_right_things() {
        let mut viewer = Viewer::new();
        let Mode::Search { query } = &mut viewer.mode else {
            panic!()
        };
        for ch in "wiki".chars() {
            query.insert(ch);
        }
        // The editing keys stay in the box; Enter hands the query over.
        assert_eq!(viewer.key(SearchKey::Left), Keyed::Kept);
        assert_eq!(viewer.key(SearchKey::Backspace), Keyed::Kept);
        assert_eq!(viewer.key(SearchKey::Other), Keyed::Nothing);
        assert_eq!(viewer.key(SearchKey::Enter), Keyed::Submit("wii".into()));
        // Enter empties the box, as a submitted query should.
        let Mode::Search { query } = &mut viewer.mode else {
            panic!()
        };
        assert!(query.value.is_empty());
    }

    #[test]
    fn result_keys_pick_and_open() {
        let mut viewer = Viewer::new();
        viewer.start(FetchKind::Search, "diamond");
        viewer.apply(Outcome::Results {
            query: "diamond".into(),
            titles: vec!["Diamond".into(), "Diamond Sword".into()],
        });
        assert_eq!(viewer.key(SearchKey::Up), Keyed::Kept);
        assert_eq!(
            viewer.selected_mode_selected(),
            Some(0),
            "the selection stays put at the top"
        );
        assert_eq!(viewer.key(SearchKey::Down), Keyed::Kept);
        assert_eq!(viewer.selected_mode_selected(), Some(1));
        assert_eq!(viewer.key(SearchKey::Down), Keyed::Kept);
        assert_eq!(
            viewer.selected_mode_selected(),
            Some(1),
            "the selection stops at the last"
        );
        assert_eq!(
            viewer.key(SearchKey::Enter),
            Keyed::Open("Diamond Sword".into())
        );
        // Letters are not the results'.
        assert_eq!(viewer.key(SearchKey::Other), Keyed::Nothing);
    }

    #[test]
    fn page_keys_scroll() {
        let mut viewer = Viewer::new();
        let body = (1..=40)
            .map(|n| format!("line {n}"))
            .collect::<Vec<_>>()
            .join("\n");
        viewer.start(FetchKind::Page, "Long");
        viewer.apply(Outcome::Page(Extract {
            title: "Long".into(),
            body,
        }));
        assert_eq!(viewer.max_scroll(), 40 - BODY_LINES);
        assert_eq!(viewer.key(SearchKey::Down), Keyed::Kept);
        assert_eq!(viewer.page_scrolled(), Some(3));
        for _ in 0..30 {
            viewer.key(SearchKey::Down);
        }
        assert_eq!(
            viewer.page_scrolled(),
            Some(40 - BODY_LINES),
            "the scroll stops at the text's end"
        );
        viewer.key(SearchKey::Up);
        viewer.scroll(1.0);
        assert_eq!(
            viewer.page_scrolled(),
            Some(40 - BODY_LINES - 6),
            "the wheel scrolls three lines a notch"
        );
        viewer.scroll(-1.0);
        assert_eq!(viewer.page_scrolled(), Some(40 - BODY_LINES - 3));
        // Letters are not the page's.
        assert_eq!(viewer.key(SearchKey::Other), Keyed::Nothing);
    }

    #[test]
    fn buttons_sit_bottom_right_and_take_clicks() {
        let panel = [10.0, 20.0, PANEL_W, PANEL_H];
        let rects = button_rects(panel);
        assert_eq!(rects.len(), 3);
        let (plan, rect) = rects[0];
        assert_eq!(plan, ClickPlan::Browser);
        assert_eq!(button_label(plan), "Open in browser");
        // The click inside hits; a pixel outside misses.
        let inside = [rect[0] + 1.0, rect[1] + 1.0];
        let outside = [rect[0] - 1.0, rect[1] + 1.0];
        let mut viewer = Viewer::new();
        viewer.mode = Mode::Page {
            title: "T".into(),
            url: "https://minecraft.wiki/wiki/T".into(),
            lines: vec![],
            scroll: 0,
            failed: false,
        };
        assert_eq!(click_plan(&viewer, panel, inside), ClickPlan::Browser);
        assert_eq!(click_plan(&viewer, panel, outside), ClickPlan::None);
        assert_eq!(button_label(rects[2].0), "Back");
    }

    #[test]
    fn result_rows_take_clicks() {
        let panel = [10.0, 20.0, PANEL_W, PANEL_H];
        let mut viewer = Viewer::new();
        viewer.mode = Mode::Results {
            titles: vec!["Diamond".into(), "Diamond Sword".into()],
            selected: 0,
        };
        // The rows start 34 px under the panel's top, 12 px a line.
        let first = [panel[0] + 100.0, panel[1] + 34.0 + 1.0];
        let second = [panel[0] + 100.0, panel[1] + 34.0 + 13.0];
        let past = [panel[0] + 100.0, panel[1] + 34.0 + 25.0];
        assert_eq!(click_plan(&viewer, panel, first), ClickPlan::Result(0));
        assert_eq!(click_plan(&viewer, panel, second), ClickPlan::Result(1));
        assert_eq!(click_plan(&viewer, panel, past), ClickPlan::None);
    }

    #[test]
    fn the_page_shows_the_files_state() {
        let rows = page_labels(true, false, true, 8.0);
        assert_eq!(rows.len(), 4);
        assert_eq!(rows[0], "Enabled: On");
        assert_eq!(rows[1], "Wiki in game: Off");
        assert_eq!(rows[2], "Confirm before opening: On");
        assert_eq!(rows[3], "Reach: 8.0 blocks (click to change)");
    }

    #[test]
    fn page_clicks_write_the_settings() {
        assert_eq!(
            page_click(0, true, true, true, 5.2),
            PageClick::Enabled(false)
        );
        assert_eq!(
            page_click(1, true, true, true, 5.2),
            PageClick::Native(false)
        );
        assert_eq!(
            page_click(2, true, true, true, 5.2),
            PageClick::Confirm(false)
        );
        assert_eq!(page_click(3, true, true, true, 5.2), PageClick::Reach(8.0));
        assert_eq!(page_click(4, true, true, true, 5.2), PageClick::None);
        assert!(stored(PageClick::None).is_none());
        assert_eq!(
            stored(PageClick::Native(false)),
            Some((SETTING_NATIVE, "false".to_string()))
        );
        assert_eq!(
            stored(PageClick::Reach(12.0)),
            Some((SETTING_REACH, "12.0".to_string()))
        );
    }

    // The mode's private bits, for the tests above.
    impl Viewer {
        fn selected_mode_selected(&self) -> Option<usize> {
            match &self.mode {
                Mode::Results { selected, .. } => Some(*selected),
                _ => None,
            }
        }

        fn page_scrolled(&self) -> Option<usize> {
            match &self.mode {
                Mode::Page { scroll, .. } => Some(*scroll),
                _ => None,
            }
        }
    }
}
