#[derive(Clone, Copy, Debug)]
pub struct Stamp {
    pub epoch_minutes: i64,
    pub hour: u8,
    pub weekday: u8,
}

#[inline]
fn dig2(b: &[u8], off: usize) -> u32 {
    let d0 = (b[off] - b'0') as u32;
    let d1 = (b[off + 1] - b'0') as u32;
    d0 * 10 + d1
}

#[inline]
fn dig4(b: &[u8], off: usize) -> u32 {
    let d0 = (b[off] - b'0') as u32;
    let d1 = (b[off + 1] - b'0') as u32;
    let d2 = (b[off + 2] - b'0') as u32;
    let d3 = (b[off + 3] - b'0') as u32;
    d0 * 1000 + d1 * 100 + d2 * 10 + d3
}

#[inline]
fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = y - (m <= 2) as i32;
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as i64;
    let m = m as i64;
    let d = d as i64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era as i64 * 146097 + doe - 719468
}

const DAYS_2026_01_01: i64 = 20_454;
const DAYS_BEFORE_MONTH_2026: [i64; 13] =
    [0, 0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
const DAYS_IN_MONTH_2026: [u32; 13] = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

#[inline]
pub fn parse_iso8601(buf: &[u8]) -> Option<Stamp> {
    if buf.len() < 20 {
        return None;
    }
    let year = dig4(buf, 0) as i32;
    let month = dig2(buf, 5);
    let day = dig2(buf, 8);
    let hour = dig2(buf, 11);
    let minute = dig2(buf, 14);
    let second = dig2(buf, 17);
    if month == 0 || month > 12 || day == 0 || day > 31 || hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    let days = if year == 2026 {
        if day > DAYS_IN_MONTH_2026[month as usize] {
            return None;
        }
        DAYS_2026_01_01 + DAYS_BEFORE_MONTH_2026[month as usize] + day as i64 - 1
    } else {
        days_from_civil(year, month, day)
    };
    let total_seconds = days * 86_400 + hour as i64 * 3600 + minute as i64 * 60 + second as i64;
    let epoch_minutes = total_seconds.div_euclid(60);
    let weekday = ((days.rem_euclid(7) + 3).rem_euclid(7)) as u8;
    Some(Stamp {
        epoch_minutes,
        hour: hour as u8,
        weekday,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_iso8601;

    #[test]
    fn parses_2026_fast_path() {
        let stamp = parse_iso8601(b"2026-03-11T20:23:35Z").unwrap();
        assert_eq!(stamp.epoch_minutes, 29_554_343);
        assert_eq!(stamp.hour, 20);
        assert_eq!(stamp.weekday, 2);
    }

    #[test]
    fn keeps_generic_fallback_for_other_years() {
        let stamp = parse_iso8601(b"2025-03-11T20:23:35Z").unwrap();
        assert_eq!(stamp.epoch_minutes, 29_028_743);
        assert_eq!(stamp.hour, 20);
        assert_eq!(stamp.weekday, 1);
    }
}
