//! AV-38: magnetic snap (spec §12.4). Pure maths on physical card rects; `window.rs` feeds it from
//! `WM_MOVING` during a native drag and from the saved state at launch.

use serde::{Deserialize, Serialize};

use crate::window::Rect;

/// Logical px: how close the card must come to its snapped spot to catch.
pub const MAGNET: f64 = 16.0;

/// The edge or corner the card is held to; every size change keeps it fixed.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Anchor {
    #[default]
    Float,
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Clone, Copy, PartialEq)]
enum Side {
    Low,
    High,
}

impl Anchor {
    fn from_sides(h: Option<Side>, v: Option<Side>) -> Anchor {
        use Side::*;
        match (h, v) {
            (None, None) => Anchor::Float,
            (Some(Low), None) => Anchor::Left,
            (Some(High), None) => Anchor::Right,
            (None, Some(Low)) => Anchor::Top,
            (None, Some(High)) => Anchor::Bottom,
            (Some(Low), Some(Low)) => Anchor::TopLeft,
            (Some(High), Some(Low)) => Anchor::TopRight,
            (Some(Low), Some(High)) => Anchor::BottomLeft,
            (Some(High), Some(High)) => Anchor::BottomRight,
        }
    }

    fn sides(self) -> (Option<Side>, Option<Side>) {
        use Side::*;
        match self {
            Anchor::Float => (None, None),
            Anchor::Left => (Some(Low), None),
            Anchor::Right => (Some(High), None),
            Anchor::Top => (None, Some(Low)),
            Anchor::Bottom => (None, Some(High)),
            Anchor::TopLeft => (Some(Low), Some(Low)),
            Anchor::TopRight => (Some(High), Some(Low)),
            Anchor::BottomLeft => (Some(Low), Some(High)),
            Anchor::BottomRight => (Some(High), Some(High)),
        }
    }
}

/// One axis: the offset that puts the card `margin` inside the nearer edge it is within `magnet`
/// of, measured from that snapped spot.
fn axis(lo: i32, len: i32, work_lo: i32, work_len: i32, margin: i32, magnet: i32) -> (i32, Option<Side>) {
    let to_lo = work_lo + margin - lo;
    let to_hi = work_lo + work_len - margin - (lo + len);
    match (to_lo.abs() <= magnet, to_hi.abs() <= magnet) {
        (true, true) if to_lo.abs() <= to_hi.abs() => (to_lo, Some(Side::Low)),
        (_, true) => (to_hi, Some(Side::High)),
        (true, false) => (to_lo, Some(Side::Low)),
        (false, false) => (0, None),
    }
}

/// Snaps `card` to `work`'s edges it is near; two edges at once is a corner.
pub fn snap(card: Rect, work: Rect, margin: i32, magnet: i32) -> (Rect, Anchor) {
    let (dx, h) = axis(card.x, card.w, work.x, work.w, margin, magnet);
    let (dy, v) = axis(card.y, card.h, work.y, work.h, margin, magnet);
    (Rect { x: card.x + dx, y: card.y + dy, ..card }, Anchor::from_sides(h, v))
}

/// Card origin for `anchor` on `work`: anchored axes sit `margin` in from their edge, free axes keep
/// `at`. Keeps a bottom- or right-anchored card on its edge whatever size it restores at.
pub fn anchored_origin(anchor: Anchor, work: Rect, margin: i32, size: (i32, i32), at: (i32, i32)) -> (i32, i32) {
    let place = |side: Option<Side>, lo: i32, len: i32, own: i32, keep: i32| match side {
        Some(Side::Low) => lo + margin,
        Some(Side::High) => lo + len - margin - own,
        None => keep,
    };
    let (h, v) = anchor.sides();
    (place(h, work.x, work.w, size.0, at.0), place(v, work.y, work.h, size.1, at.1))
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORK: Rect = Rect { x: 0, y: 0, w: 1920, h: 1040 };

    fn card(x: i32, y: i32) -> Rect {
        Rect { x, y, w: 354, h: 300 }
    }

    #[test]
    fn snap_catches_within_magnet_and_floats_beyond() {
        // Snapped left spot is x = 12; caught for x in [-4, 28].
        assert_eq!(snap(card(28, 400), WORK, 12, 16), (card(12, 400), Anchor::Left));
        assert_eq!(snap(card(-4, 400), WORK, 12, 16), (card(12, 400), Anchor::Left));
        assert_eq!(snap(card(29, 400), WORK, 12, 16), (card(29, 400), Anchor::Float));
    }

    #[test]
    fn snap_each_edge_and_corner() {
        let right = 1920 - 12 - 354;
        let bottom = 1040 - 12 - 300;
        assert_eq!(snap(card(right + 10, 400), WORK, 12, 16).1, Anchor::Right);
        assert_eq!(snap(card(600, 5), WORK, 12, 16), (card(600, 12), Anchor::Top));
        assert_eq!(snap(card(600, bottom - 16), WORK, 12, 16), (card(600, bottom), Anchor::Bottom));
        assert_eq!(snap(card(right - 3, bottom + 7), WORK, 12, 16), (card(right, bottom), Anchor::BottomRight));
        assert_eq!(snap(card(0, 0), WORK, 12, 16), (card(12, 12), Anchor::TopLeft));
        assert_eq!(snap(card(right, 20), WORK, 12, 16).1, Anchor::TopRight);
        assert_eq!(snap(card(14, bottom), WORK, 12, 16).1, Anchor::BottomLeft);
    }

    #[test]
    fn snap_uses_the_given_work_area_on_any_monitor() {
        let left_mon = Rect { x: -2560, y: -200, w: 2560, h: 1392 };
        let (r, a) = snap(card(-2560, -200 + 1392 - 300 - 5), left_mon, 15, 20);
        assert_eq!(a, Anchor::BottomLeft);
        assert_eq!((r.x, r.y + r.h), (-2545, -200 + 1392 - 15));
    }

    #[test]
    fn snap_prefers_the_nearer_edge_when_both_are_in_range() {
        let narrow = Rect { x: 0, y: 0, w: 390, h: 1040 };
        assert_eq!(snap(card(10, 400), narrow, 12, 16).1, Anchor::Left);
        assert_eq!(snap(card(30, 400), narrow, 12, 16).1, Anchor::Right);
    }

    #[test]
    fn anchored_origin_holds_anchored_edges_and_keeps_free_axes() {
        let size = (354, 120);
        assert_eq!(anchored_origin(Anchor::BottomRight, WORK, 12, size, (0, 0)), (1920 - 12 - 354, 1040 - 12 - 120));
        assert_eq!(anchored_origin(Anchor::Left, WORK, 12, size, (500, 333)), (12, 333));
        assert_eq!(anchored_origin(Anchor::Bottom, WORK, 12, size, (500, 0)), (500, 1040 - 12 - 120));
        assert_eq!(anchored_origin(Anchor::Float, WORK, 12, size, (500, 333)), (500, 333));
    }

    #[test]
    fn anchor_serialises_kebab_case_and_defaults_to_float() {
        assert_eq!(serde_json::to_string(&Anchor::TopRight).unwrap(), "\"top-right\"");
        assert_eq!(Anchor::default(), Anchor::Float);
    }
}
