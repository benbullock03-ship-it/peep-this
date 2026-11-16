//! AV-81: account rate-limit pressure: the 80% sweat, and the 95% and 100% toasts once per limit window.

use std::sync::Mutex;

use serde_json::Value;

use crate::store::RateLimits;

pub const WARN_PCT: f64 = 80.0;
const LEVELS: [u8; 2] = [95, 100];
/// Two `resets_at` this close are one window, in case the reported time drifts between payloads.
const WINDOW_SLACK_MS: u64 = 60_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Limit {
    FiveHour,
    SevenDay,
}

impl Limit {
    fn name(self) -> &'static str {
        match self {
            Limit::FiveHour => "5-hour",
            Limit::SevenDay => "7-day",
        }
    }
}

/// The higher of the two limits, or 0 before any payload.
pub fn highest(rl: Option<&RateLimits>) -> f64 {
    rl.map_or(0.0, |r| r.five_hour_pct.max(r.seven_day_pct))
}

/// `resets_at` as epoch ms; accepts epoch seconds, epoch ms or an ISO 8601 string.
pub fn reset_ms(v: &Value) -> Option<u64> {
    match v {
        Value::Number(n) => {
            let n = n.as_f64().filter(|n| *n > 0.0)?;
            Some(if n < 1e12 { (n * 1000.0) as u64 } else { n as u64 })
        }
        Value::String(s) => parse_iso(s),
        _ => None,
    }
}

/// `YYYY-MM-DDTHH:MM:SS[.fff](Z|±HH:MM)`.
fn parse_iso(s: &str) -> Option<u64> {
    let num = |a: usize, b: usize| s.get(a..b)?.parse::<i64>().ok();
    let (y, mo, d) = (num(0, 4)?, num(5, 7)?, num(8, 10)?);
    let (h, mi, se) = (num(11, 13)?, num(14, 16)?, num(17, 19)?);
    let mut rest = s.get(19..)?;
    let mut ms = 0;
    if let Some(r) = rest.strip_prefix('.') {
        let n = r.bytes().take_while(u8::is_ascii_digit).count();
        ms = format!("{:0<3}", &r[..n.min(3)]).parse().ok()?;
        rest = &r[n..];
    }
    let offset_min = match rest {
        "" | "Z" | "z" => 0,
        _ => {
            let sign = match rest.as_bytes()[0] {
                b'+' => 1,
                b'-' => -1,
                _ => return None,
            };
            let hh: i64 = rest.get(1..3)?.parse().ok()?;
            let mm: i64 = rest.get(rest.len() - 2..)?.parse().ok()?;
            sign * (hh * 60 + mm)
        }
    };
    let secs = days_from_civil(y, mo, d) * 86_400 + h * 3600 + mi * 60 + se - offset_min * 60;
    u64::try_from(secs * 1000 + ms).ok()
}

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Per limit: the window last seen and the highest level already toasted in it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Armed {
    window: Option<u64>,
    shown: u8,
}

/// The level to toast now, if any. A new window re-arms: a later `resets_at`, or, with none known, a drop under 80%.
/// An earlier `resets_at` is a stale payload from another session and changes nothing.
pub fn next_toast(prev: Armed, pct: f64, resets_at: Option<u64>) -> (Armed, Option<u8>) {
    match (resets_at, prev.window) {
        (Some(r), Some(w)) if r + WINDOW_SLACK_MS < w => return (prev, None),
        _ => {}
    }
    let rearm = match (resets_at, prev.window) {
        (Some(r), Some(w)) => r > w + WINDOW_SLACK_MS,
        _ => pct < WARN_PCT,
    };
    let shown = if rearm { 0 } else { prev.shown };
    // A jump straight past 95 shows only the higher toast.
    let toast = LEVELS.iter().rev().copied().find(|l| pct >= f64::from(*l)).filter(|l| *l > shown);
    (Armed { window: resets_at.max(prev.window), shown: toast.unwrap_or(shown) }, toast)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LimitToast {
    pub limit: Limit,
    pub level: u8,
    pub resets_at: Option<u64>,
}

#[derive(Default)]
pub struct Alerts {
    armed: Mutex<[Armed; 2]>,
}

impl Alerts {
    pub fn check(&self, rl: &RateLimits) -> Vec<LimitToast> {
        let mut armed = self.armed.lock().unwrap();
        let readings = [
            (Limit::FiveHour, rl.five_hour_pct, rl.five_hour_resets_at),
            (Limit::SevenDay, rl.seven_day_pct, rl.seven_day_resets_at),
        ];
        let mut out = Vec::new();
        for (slot, (limit, pct, resets_at)) in armed.iter_mut().zip(readings) {
            let (next, toast) = next_toast(*slot, pct, resets_at);
            *slot = next;
            out.extend(toast.map(|level| LimitToast { limit, level, resets_at }));
        }
        out
    }
}

/// Title and body per AV-79 §04. `offset_ms` is local time minus UTC.
pub fn toast_text(t: &LimitToast, now: u64, offset_ms: i64) -> (String, String) {
    let name = t.limit.name();
    let reset = t.resets_at.filter(|r| *r > now);
    let at = reset.map(|r| clock(r, now, offset_ms));
    if t.level >= 100 {
        let body = match (reset, at) {
            (Some(r), Some(at)) => format!("New prompts wait until it resets at {at}, in {}.", until(r - now)),
            _ => "Reset time not known yet.".into(),
        };
        (format!("{name} limit reached"), body)
    } else {
        let when = at.map_or("Reset time not known yet.".into(), |at| format!("Resets at {at}."));
        (format!("{name} limit at {}%", t.level), format!("{when} Anything you start now may stop partway."))
    }
}

/// `14:20` today, `Thu 14:20` on another day.
fn clock(at: u64, now: u64, offset_ms: i64) -> String {
    let local = |t: u64| (t as i64 + offset_ms).div_euclid(1000);
    let (secs, today) = (local(at), local(now).div_euclid(86_400));
    let day = secs.div_euclid(86_400);
    let hm = format!("{:02}:{:02}", secs.rem_euclid(86_400) / 3600, secs.rem_euclid(3600) / 60);
    if day == today {
        return hm;
    }
    // 1970-01-01 was a Thursday.
    const DAYS: [&str; 7] = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"];
    format!("{} {hm}", DAYS[day.rem_euclid(7) as usize])
}

/// `12m`, `1h 12m`, `2d 3h`; rounded up to the minute.
fn until(ms: u64) -> String {
    let m = ms.div_ceil(60_000);
    match (m / 1440, m % 1440 / 60, m % 60) {
        (0, 0, m) => format!("{m}m"),
        (0, h, m) => format!("{h}h {m}m"),
        (d, h, _) => format!("{d}d {h}h"),
    }
}

/// Local time minus UTC, from the OS clock.
#[cfg(windows)]
pub fn local_offset_ms() -> i64 {
    use windows_sys::Win32::Foundation::SYSTEMTIME;
    use windows_sys::Win32::System::SystemInformation::{GetLocalTime, GetSystemTime};
    let ms = |t: &SYSTEMTIME| {
        let days = days_from_civil(t.wYear.into(), t.wMonth.into(), t.wDay.into());
        ((days * 24 + i64::from(t.wHour)) * 60 + i64::from(t.wMinute)) * 60_000
    };
    let (mut local, mut utc) = (unsafe { std::mem::zeroed::<SYSTEMTIME>() }, unsafe { std::mem::zeroed::<SYSTEMTIME>() });
    unsafe {
        GetLocalTime(&mut local);
        GetSystemTime(&mut utc);
    }
    // Round to 15 minutes so a minute ticking over between the two calls doesn't skew it.
    let diff = ms(&local) - ms(&utc);
    (diff as f64 / 900_000.0).round() as i64 * 900_000
}

#[cfg(not(windows))]
pub fn local_offset_ms() -> i64 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    const T0: u64 = 1_763_000_000_000;
    const MIN: u64 = 60_000;

    fn run(pcts: &[f64], resets_at: Option<u64>) -> Vec<Option<u8>> {
        let mut a = Armed::default();
        pcts.iter()
            .map(|p| {
                let (next, t) = next_toast(a, *p, resets_at);
                a = next;
                t
            })
            .collect()
    }

    #[test]
    fn toasts_once_at_95_and_once_at_100_per_window() {
        assert_eq!(run(&[80.0, 94.9, 95.0, 97.0, 99.0, 100.0, 100.0], Some(T0)), vec![None, None, Some(95), None, None, Some(100), None]);
        assert_eq!(run(&[50.0, 100.0, 96.0], Some(T0)), vec![None, Some(100), None], "a jump past 95 shows only the 100 toast");
    }

    #[test]
    fn a_new_window_rearms_both_levels() {
        let (a, t) = next_toast(Armed::default(), 96.0, Some(T0));
        assert_eq!(t, Some(95));
        let (a, t) = next_toast(a, 96.0, Some(T0 + 30_000));
        assert_eq!(t, None, "drift inside the slack is the same window");
        assert_eq!(next_toast(a, 90.0, Some(T0)).1, None, "a stale lower payload in the same window doesn't rearm");
        let (a, t) = next_toast(a, 97.0, Some(T0 + 5 * 60 * MIN));
        assert_eq!(t, Some(95));
        let (a, t) = next_toast(a, 20.0, Some(T0));
        assert_eq!(t, None, "a stale payload from the old window changes nothing");
        assert_eq!(next_toast(a, 97.0, Some(T0 + 5 * 60 * MIN)).1, None, "so the new window's toast is not repeated");
    }

    #[test]
    fn without_a_reset_time_a_drop_under_80_rearms() {
        assert_eq!(run(&[95.0, 90.0, 96.0, 10.0, 95.0], None), vec![Some(95), None, None, None, Some(95)]);
    }

    #[test]
    fn alerts_track_each_limit_alone() {
        let alerts = Alerts::default();
        let rl = |five: f64, seven: f64| RateLimits {
            five_hour_pct: five,
            seven_day_pct: seven,
            five_hour_resets_at: Some(T0),
            seven_day_resets_at: None,
            at: 0,
        };
        assert_eq!(alerts.check(&rl(95.0, 20.0)), vec![LimitToast { limit: Limit::FiveHour, level: 95, resets_at: Some(T0) }]);
        assert_eq!(alerts.check(&rl(96.0, 100.0)), vec![LimitToast { limit: Limit::SevenDay, level: 100, resets_at: None }]);
        assert!(alerts.check(&rl(96.0, 100.0)).is_empty());
    }

    #[test]
    fn reset_time_parses_seconds_ms_and_iso() {
        assert_eq!(reset_ms(&Value::from(1_763_000_000)), Some(T0));
        assert_eq!(reset_ms(&Value::from(T0)), Some(T0));
        assert_eq!(reset_ms(&Value::from("2026-11-13T18:00:00Z")), Some(1_794_592_800_000));
        assert_eq!(reset_ms(&Value::from("2026-11-13T19:00:00.5+01:00")), Some(1_794_592_800_500));
        assert_eq!(reset_ms(&Value::from("1970-01-01T00:00:01-00:30")), Some(1_801_000));
        assert_eq!(reset_ms(&Value::from("soon")), None);
        assert_eq!(reset_ms(&Value::Null), None);
    }

    #[test]
    fn toast_text_follows_the_design() {
        // 2026-11-13 13:08 UTC, shown at UTC+1.
        let now = 1_794_575_280_000;
        let hour = 3_600_000;
        let resets = Some(now + hour + 12 * MIN);
        let five = |level| LimitToast { limit: Limit::FiveHour, level, resets_at: resets };
        assert_eq!(
            toast_text(&five(95), now, hour as i64),
            ("5-hour limit at 95%".into(), "Resets at 15:20. Anything you start now may stop partway.".into())
        );
        assert_eq!(
            toast_text(&five(100), now, hour as i64),
            ("5-hour limit reached".into(), "New prompts wait until it resets at 15:20, in 1h 12m.".into())
        );
        let unknown = LimitToast { limit: Limit::SevenDay, level: 100, resets_at: None };
        assert_eq!(toast_text(&unknown, now, 0), ("7-day limit reached".into(), "Reset time not known yet.".into()));
        let past = LimitToast { limit: Limit::SevenDay, level: 95, resets_at: Some(now - 1) };
        assert_eq!(toast_text(&past, now, 0).1, "Reset time not known yet. Anything you start now may stop partway.");
        let later = LimitToast { limit: Limit::SevenDay, level: 100, resets_at: Some(now + 2 * 24 * hour + 3 * hour) };
        assert_eq!(toast_text(&later, now, 0).1, "New prompts wait until it resets at Sun 16:08, in 2d 3h.");
    }
}
