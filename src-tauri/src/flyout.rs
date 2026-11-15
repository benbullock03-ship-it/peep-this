//! AV-42: a waiting session's row flies out beside the strip (spec §12.7). The flyouts live in a
//! second transparent window over the zone beside the strip, clipped to the cards by its region:
//! hovering one is never hovering the strip (AV-41), and the strip's window never resizes for them.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

use crate::snap::Anchor;
use crate::window::{self, Rect, SHADOW};

pub const LABEL: &str = "flyout";
/// Flyout card (`.ps-fly`, 300) plus its shadow and the gap to the strip (flyout.ts `GAP`).
pub const ZONE_W: f64 = 316.0;

/// Where the flyouts sit, seen from the strip: always away from the snapped edge.
#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Left,
    Right,
    Up,
    Down,
}

/// A session to fly out; `at` is its peep's centre along the strip, CSS px from the plate's edge.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Item {
    pub id: String,
    pub at: f64,
}

/// What the flyout page lays out: `origin` is where the plate starts along its axis in the zone.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct FlyView {
    pub side: Side,
    pub origin: f64,
    pub zone: [f64; 2],
    pub items: Vec<Item>,
}

fn phys(v: f64, scale: f64) -> i32 {
    (v * scale).round() as i32
}

/// A floating strip opens right, like the panel, unless only the left has room.
pub fn side_for(anchor: Anchor, card: Rect, work: Rect, zone_w: i32, shadow: i32) -> Side {
    use Anchor::*;
    match anchor {
        Right | TopRight | BottomRight => Side::Left,
        Left | TopLeft | BottomLeft => Side::Right,
        Top => Side::Down,
        Bottom => Side::Up,
        Float => {
            let right_fits = card.x + card.w + shadow + zone_w <= work.x + work.w;
            if !right_fits && card.x - zone_w >= work.x {
                Side::Left
            } else {
                Side::Right
            }
        }
    }
}

/// The flyout window's rect (physical): beside a column it spans the work area's height, off a row
/// the work area's width out to the far edge.
pub fn zone(side: Side, card: Rect, work: Rect, zone_w: i32, shadow: i32) -> Rect {
    let (right, bottom) = (card.x + card.w + shadow, card.y + card.h + shadow);
    let r = match side {
        Side::Left => Rect { x: card.x - zone_w, y: work.y, w: zone_w, h: work.h },
        Side::Right => Rect { x: right, y: work.y, w: zone_w, h: work.h },
        Side::Down => Rect { x: work.x, y: bottom, w: work.w, h: work.y + work.h - bottom },
        Side::Up => Rect { x: work.x, y: work.y, w: work.w, h: card.y - work.y },
    };
    Rect { w: r.w.max(1), h: r.h.max(1), ..r }
}

pub fn view(anchor: Anchor, card: Rect, work: Rect, scale: f64, items: Vec<Item>) -> (Rect, FlyView) {
    let (zw, s) = (phys(ZONE_W, scale), phys(SHADOW, scale));
    let side = side_for(anchor, card, work, zw, s);
    let z = zone(side, card, work, zw, s);
    let origin = match side {
        Side::Left | Side::Right => card.y - z.y,
        Side::Up | Side::Down => card.x - z.x,
    } as f64
        / scale;
    (z, FlyView { side, origin, zone: [z.w as f64 / scale, z.h as f64 / scale], items })
}

static ITEMS: Mutex<Vec<Item>> = Mutex::new(Vec::new());
/// The zone, scale and view on screen; None while the window is hidden.
static SHOWN: Mutex<Option<(Rect, f64, FlyView)>> = Mutex::new(None);

pub fn setup<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<()> {
    WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("flyout.html".into()))
        .title("Peep This")
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .focusable(false)
        .focused(false)
        .visible(false)
        .inner_size(1.0, 1.0)
        .build()?;
    Ok(())
}

/// Shows, moves or hides the flyouts after anything that changes them: the list, the mode, or the
/// strip's rect. Hidden whenever the strip is not up and still.
pub fn refresh<R: Runtime>(app: &AppHandle<R>) {
    let Some(fw) = app.get_webview_window(LABEL) else { return };
    let items = ITEMS.lock().unwrap().clone();
    let next = if items.is_empty() {
        None
    } else {
        window::still_strip(app).map(|(card, work, scale)| {
            let (r, v) = view(window::anchor(), card, work, scale, items);
            (r, scale, v)
        })
    };
    let prev = std::mem::replace(&mut *SHOWN.lock().unwrap(), next.clone());
    let Some((r, _, v)) = next else {
        if prev.is_some() {
            let _ = fw.hide();
            // The page drops its cards, so their peeps stop drawing.
            let _ = app.emit_to(LABEL, "flyouts", None::<FlyView>);
        }
        return;
    };
    let fresh = prev.as_ref().map(|p| p.0) != Some(r);
    if fresh || prev.is_none() {
        // Nothing shows until the page has laid out and sent its region.
        #[cfg(windows)]
        if let Ok(h) = fw.hwnd() {
            window::set_region(h.0 as isize, &[]);
            window::place_rect(h.0 as isize, r);
        }
    }
    if prev.is_none() {
        if let Err(e) = window::show_inactive(&fw) {
            eprintln!("peepthis: flyout show failed: {e}");
        }
    }
    if fresh || prev.is_none() || prev.as_ref().map(|p| &p.2) != Some(&v) {
        let _ = app.emit_to(LABEL, "flyouts", &v);
    }
}

#[tauri::command]
pub fn set_flyouts(app: AppHandle, items: Vec<Item>) {
    *ITEMS.lock().unwrap() = items;
    refresh(&app);
}

#[tauri::command]
pub fn get_flyouts() -> Option<FlyView> {
    SHOWN.lock().unwrap().as_ref().map(|s| s.2.clone())
}

/// The cards' rects (CSS px in the flyout window); everything else lets clicks through.
#[tauri::command]
pub fn flyout_region(app: AppHandle, rects: Vec<[f64; 4]>) {
    let Some(scale) = SHOWN.lock().unwrap().as_ref().map(|s| s.1) else { return };
    let parts: Vec<Rect> = rects.into_iter().map(|r| window::cover(r, scale)).collect();
    #[cfg(windows)]
    if let Some(h) = app.get_webview_window(LABEL).and_then(|w| w.hwnd().ok()) {
        window::set_region(h.0 as isize, &parts);
    }
    #[cfg(not(windows))]
    let _ = (app, parts);
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORK: Rect = Rect { x: 0, y: 0, w: 2560, h: 1400 };

    #[test]
    fn flyouts_point_away_from_the_snapped_edge() {
        use Anchor::*;
        let card = Rect { x: 1000, y: 400, w: 62, h: 300 };
        for a in [Right, TopRight, BottomRight] {
            assert_eq!(side_for(a, card, WORK, 316, 6), Side::Left);
        }
        for a in [Left, TopLeft, BottomLeft] {
            assert_eq!(side_for(a, card, WORK, 316, 6), Side::Right);
        }
        assert_eq!(side_for(Top, card, WORK, 316, 6), Side::Down);
        assert_eq!(side_for(Bottom, card, WORK, 316, 6), Side::Up);
    }

    #[test]
    fn floating_strip_opens_right_unless_only_the_left_has_room() {
        let mid = Rect { x: 1000, y: 400, w: 62, h: 300 };
        assert_eq!(side_for(Anchor::Float, mid, WORK, 316, 6), Side::Right);
        let near_right = Rect { x: 2400, ..mid };
        assert_eq!(side_for(Anchor::Float, near_right, WORK, 316, 6), Side::Left);
    }

    #[test]
    fn right_edge_zone_ends_at_the_card_and_spans_the_work_height() {
        let card = Rect { x: 2486, y: 400, w: 62, h: 300 };
        let (z, v) = view(Anchor::Right, card, WORK, 1.0, vec![]);
        assert_eq!(z, Rect { x: 2486 - 316, y: 0, w: 316, h: 1400 });
        assert_eq!((v.side, v.origin, v.zone), (Side::Left, 400.0, [316.0, 1400.0]));
    }

    #[test]
    fn left_edge_zone_starts_past_the_shadow() {
        let card = Rect { x: 18, y: 300, w: 93, h: 450 };
        let (z, v) = view(Anchor::Left, card, WORK, 1.5, vec![]);
        assert_eq!((z.x, z.w), (18 + 93 + 9, 474));
        assert_eq!((v.side, v.origin), (Side::Right, 200.0));
    }

    #[test]
    fn top_edge_zone_sits_under_the_row_and_bottom_edge_above_it() {
        let row = Rect { x: 1200, y: 12, w: 400, h: 62 };
        let (z, v) = view(Anchor::Top, row, WORK, 1.0, vec![]);
        assert_eq!(z, Rect { x: 0, y: 80, w: 2560, h: 1320 });
        assert_eq!((v.side, v.origin), (Side::Down, 1200.0));
        let row = Rect { y: 1400 - 12 - 62, ..row };
        let (z, v) = view(Anchor::Bottom, row, WORK, 1.0, vec![]);
        assert_eq!(z, Rect { x: 0, y: 0, w: 2560, h: 1326 });
        assert_eq!(v.side, Side::Up);
    }

    #[test]
    fn zone_never_collapses_to_nothing() {
        let card = Rect { x: 0, y: 0, w: 62, h: 1400 };
        assert!(zone(Side::Up, card, WORK, 316, 6).h >= 1);
    }
}
