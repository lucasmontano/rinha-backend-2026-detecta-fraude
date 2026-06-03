use crate::time::{parse_iso8601, Stamp};

#[derive(Default, Clone, Copy, Debug)]
pub struct RawPayload<'a> {
    pub amount: f64,
    pub installments: u32,
    pub requested_at: Stamp,
    pub customer_avg_amount: f64,
    pub tx_count_24h: u32,
    pub known_merchants_buf: &'a [u8],
    pub merchant_id: &'a [u8],
    pub merchant_mcc: &'a [u8],
    pub merchant_avg_amount: f64,
    pub is_online: bool,
    pub card_present: bool,
    pub km_from_home: f64,
    pub has_last_tx: bool,
    pub last_tx_stamp: Stamp,
    pub last_tx_km: f64,
}

impl Default for Stamp {
    fn default() -> Self {
        Stamp {
            epoch_minutes: 0,
            hour: 0,
            weekday: 0,
        }
    }
}

#[derive(Debug)]
pub struct ParseError;

#[inline]
fn skip_ws(buf: &[u8], mut i: usize) -> usize {
    while i < buf.len() {
        let c = buf[i];
        if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' {
            i += 1;
        } else {
            break;
        }
    }
    i
}

#[inline]
fn expect(buf: &[u8], i: usize, c: u8) -> Result<usize, ParseError> {
    let i = skip_ws(buf, i);
    if i >= buf.len() || buf[i] != c {
        return Err(ParseError);
    }
    Ok(i + 1)
}

#[inline]
fn read_string<'a>(buf: &'a [u8], i: usize) -> Result<(&'a [u8], usize), ParseError> {
    let i = skip_ws(buf, i);
    if i >= buf.len() || buf[i] != b'"' {
        return Err(ParseError);
    }
    let start = i + 1;
    let mut j = start;
    while j < buf.len() && buf[j] != b'"' {
        j += 1;
    }
    if j >= buf.len() {
        return Err(ParseError);
    }
    Ok((&buf[start..j], j + 1))
}

#[inline]
fn read_number_f64(buf: &[u8], i: usize) -> Result<(f64, usize), ParseError> {
    let mut i = skip_ws(buf, i);
    if i >= buf.len() {
        return Err(ParseError);
    }

    let mut neg = false;
    if buf[i] == b'-' {
        neg = true;
        i += 1;
    } else if buf[i] == b'+' {
        i += 1;
    }

    let mut saw_digit = false;
    let mut value = 0.0f64;
    while i < buf.len() && buf[i].is_ascii_digit() {
        value = value * 10.0 + f64::from(buf[i] - b'0');
        i += 1;
        saw_digit = true;
    }

    if i < buf.len() && buf[i] == b'.' {
        i += 1;
        let mut scale = 0.1f64;
        while i < buf.len() && buf[i].is_ascii_digit() {
            value += f64::from(buf[i] - b'0') * scale;
            scale *= 0.1;
            i += 1;
            saw_digit = true;
        }
    }

    if !saw_digit {
        return Err(ParseError);
    }

    if i < buf.len() && (buf[i] == b'e' || buf[i] == b'E') {
        i += 1;
        let mut exp_neg = false;
        if i < buf.len() && buf[i] == b'-' {
            exp_neg = true;
            i += 1;
        } else if i < buf.len() && buf[i] == b'+' {
            i += 1;
        }
        let mut saw_exp = false;
        let mut exp = 0i32;
        while i < buf.len() && buf[i].is_ascii_digit() {
            exp = exp
                .saturating_mul(10)
                .saturating_add(i32::from(buf[i] - b'0'));
            i += 1;
            saw_exp = true;
        }
        if !saw_exp {
            return Err(ParseError);
        }
        if exp_neg {
            exp = -exp;
        }
        value *= 10f64.powi(exp);
    }

    if neg {
        value = -value;
    }
    Ok((value, i))
}

#[inline]
fn read_number_u32(buf: &[u8], i: usize) -> Result<(u32, usize), ParseError> {
    let i = skip_ws(buf, i);
    let start = i;
    let mut j = i;
    while j < buf.len() && (b'0'..=b'9').contains(&buf[j]) {
        j += 1;
    }
    if j == start {
        return Err(ParseError);
    }
    let mut v = 0u32;
    for &b in &buf[start..j] {
        v = v.saturating_mul(10).saturating_add(u32::from(b - b'0'));
    }
    Ok((v, j))
}

#[inline]
fn read_bool(buf: &[u8], i: usize) -> Result<(bool, usize), ParseError> {
    let i = skip_ws(buf, i);
    if i + 4 <= buf.len() && &buf[i..i + 4] == b"true" {
        Ok((true, i + 4))
    } else if i + 5 <= buf.len() && &buf[i..i + 5] == b"false" {
        Ok((false, i + 5))
    } else {
        Err(ParseError)
    }
}

#[inline]
fn is_null(buf: &[u8], i: usize) -> bool {
    let i = skip_ws(buf, i);
    i + 4 <= buf.len() && &buf[i..i + 4] == b"null"
}

fn skip_value(buf: &[u8], i: usize) -> Result<usize, ParseError> {
    let mut i = skip_ws(buf, i);
    if i >= buf.len() {
        return Err(ParseError);
    }
    match buf[i] {
        b'"' => {
            i += 1;
            while i < buf.len() && buf[i] != b'"' {
                i += 1;
            }
            if i >= buf.len() {
                return Err(ParseError);
            }
            Ok(i + 1)
        }
        b'{' | b'[' => {
            let open = buf[i];
            let close = if open == b'{' { b'}' } else { b']' };
            let mut depth = 1;
            i += 1;
            while i < buf.len() && depth > 0 {
                let c = buf[i];
                if c == b'"' {
                    i += 1;
                    while i < buf.len() && buf[i] != b'"' {
                        i += 1;
                    }
                } else if c == open {
                    depth += 1;
                } else if c == close {
                    depth -= 1;
                }
                i += 1;
            }
            if depth != 0 {
                return Err(ParseError);
            }
            Ok(i)
        }
        b't' => Ok(i + 4),
        b'f' => Ok(i + 5),
        b'n' => Ok(i + 4),
        _ => {
            while i < buf.len() {
                let c = buf[i];
                if c == b','
                    || c == b'}'
                    || c == b']'
                    || c == b' '
                    || c == b'\n'
                    || c == b'\r'
                    || c == b'\t'
                {
                    break;
                }
                i += 1;
            }
            Ok(i)
        }
    }
}

fn read_array_raw<'a>(buf: &'a [u8], i: usize) -> Result<(&'a [u8], usize), ParseError> {
    let i = skip_ws(buf, i);
    if i >= buf.len() || buf[i] != b'[' {
        return Err(ParseError);
    }
    let start = i;
    let end = skip_value(buf, i)?;
    Ok((&buf[start..end], end))
}

fn for_each_kv<'a, F>(buf: &'a [u8], start: usize, mut on_key: F) -> Result<usize, ParseError>
where
    F: FnMut(&'a [u8], &'a [u8], usize) -> Result<Option<usize>, ParseError>,
{
    let mut i = expect(buf, start, b'{')?;
    loop {
        i = skip_ws(buf, i);
        if i < buf.len() && buf[i] == b'}' {
            return Ok(i + 1);
        }
        let (key, next) = read_string(buf, i)?;
        i = expect(buf, next, b':')?;
        i = skip_ws(buf, i);
        let consumed = on_key(buf, key, i)?;
        i = match consumed {
            Some(n) => n,
            None => skip_value(buf, i)?,
        };
        i = skip_ws(buf, i);
        if i < buf.len() && buf[i] == b',' {
            i += 1;
            continue;
        }
        if i < buf.len() && buf[i] == b'}' {
            return Ok(i + 1);
        }
        return Err(ParseError);
    }
}

pub fn parse_payload(buf: &[u8]) -> Result<RawPayload<'_>, ParseError> {
    parse_payload_ordered(buf).or_else(|_| parse_payload_generic(buf))
}

#[inline]
fn expect_key(buf: &[u8], i: usize, expected: &[u8]) -> Result<usize, ParseError> {
    let (key, next) = read_string(buf, i)?;
    if key != expected {
        return Err(ParseError);
    }
    expect(buf, next, b':')
}

fn parse_payload_ordered(buf: &[u8]) -> Result<RawPayload<'_>, ParseError> {
    let mut p: RawPayload<'_> = RawPayload::default();
    let mut i = expect(buf, 0, b'{')?;

    i = expect_key(buf, i, b"id")?;
    let (_, next) = read_string(buf, i)?;
    i = expect(buf, next, b',')?;

    i = expect_key(buf, i, b"transaction")?;
    i = expect(buf, i, b'{')?;
    i = expect_key(buf, i, b"amount")?;
    let (n, next) = read_number_f64(buf, i)?;
    p.amount = n;
    i = expect(buf, next, b',')?;
    i = expect_key(buf, i, b"installments")?;
    let (n, next) = read_number_u32(buf, i)?;
    p.installments = n;
    i = expect(buf, next, b',')?;
    i = expect_key(buf, i, b"requested_at")?;
    let (s, next) = read_string(buf, i)?;
    p.requested_at = parse_iso8601(s).ok_or(ParseError)?;
    i = expect(buf, next, b'}')?;
    i = expect(buf, i, b',')?;

    i = expect_key(buf, i, b"customer")?;
    i = expect(buf, i, b'{')?;
    i = expect_key(buf, i, b"avg_amount")?;
    let (n, next) = read_number_f64(buf, i)?;
    p.customer_avg_amount = n;
    i = expect(buf, next, b',')?;
    i = expect_key(buf, i, b"tx_count_24h")?;
    let (n, next) = read_number_u32(buf, i)?;
    p.tx_count_24h = n;
    i = expect(buf, next, b',')?;
    i = expect_key(buf, i, b"known_merchants")?;
    let (arr, next) = read_array_raw(buf, i)?;
    p.known_merchants_buf = arr;
    i = expect(buf, next, b'}')?;
    i = expect(buf, i, b',')?;

    i = expect_key(buf, i, b"merchant")?;
    i = expect(buf, i, b'{')?;
    i = expect_key(buf, i, b"id")?;
    let (s, next) = read_string(buf, i)?;
    p.merchant_id = s;
    i = expect(buf, next, b',')?;
    i = expect_key(buf, i, b"mcc")?;
    let (s, next) = read_string(buf, i)?;
    p.merchant_mcc = s;
    i = expect(buf, next, b',')?;
    i = expect_key(buf, i, b"avg_amount")?;
    let (n, next) = read_number_f64(buf, i)?;
    p.merchant_avg_amount = n;
    i = expect(buf, next, b'}')?;
    i = expect(buf, i, b',')?;

    i = expect_key(buf, i, b"terminal")?;
    i = expect(buf, i, b'{')?;
    i = expect_key(buf, i, b"is_online")?;
    let (b, next) = read_bool(buf, i)?;
    p.is_online = b;
    i = expect(buf, next, b',')?;
    i = expect_key(buf, i, b"card_present")?;
    let (b, next) = read_bool(buf, i)?;
    p.card_present = b;
    i = expect(buf, next, b',')?;
    i = expect_key(buf, i, b"km_from_home")?;
    let (n, next) = read_number_f64(buf, i)?;
    p.km_from_home = n;
    i = expect(buf, next, b'}')?;
    i = expect(buf, i, b',')?;

    i = expect_key(buf, i, b"last_transaction")?;
    let null_start = skip_ws(buf, i);
    if null_start + 4 <= buf.len() && &buf[null_start..null_start + 4] == b"null" {
        p.has_last_tx = false;
        i = null_start + 4;
    } else {
        p.has_last_tx = true;
        i = expect(buf, i, b'{')?;
        i = expect_key(buf, i, b"timestamp")?;
        let (s, next) = read_string(buf, i)?;
        p.last_tx_stamp = parse_iso8601(s).ok_or(ParseError)?;
        i = expect(buf, next, b',')?;
        i = expect_key(buf, i, b"km_from_current")?;
        let (n, next) = read_number_f64(buf, i)?;
        p.last_tx_km = n;
        i = expect(buf, next, b'}')?;
    }
    let _ = expect(buf, i, b'}')?;

    Ok(p)
}

fn parse_payload_generic(buf: &[u8]) -> Result<RawPayload<'_>, ParseError> {
    let mut p: RawPayload<'_> = RawPayload::default();

    for_each_kv(buf, 0, |buf, key, vstart| match key {
        b"id" => Ok(None),
        b"transaction" => {
            let end = for_each_kv(buf, vstart, |buf, k, v| match k {
                b"amount" => {
                    let (n, e) = read_number_f64(buf, v)?;
                    p.amount = n;
                    Ok(Some(e))
                }
                b"installments" => {
                    let (n, e) = read_number_u32(buf, v)?;
                    p.installments = n;
                    Ok(Some(e))
                }
                b"requested_at" => {
                    let (s, e) = read_string(buf, v)?;
                    p.requested_at = parse_iso8601(s).ok_or(ParseError)?;
                    Ok(Some(e))
                }
                _ => Ok(None),
            })?;
            Ok(Some(end))
        }
        b"customer" => {
            let end = for_each_kv(buf, vstart, |buf, k, v| match k {
                b"avg_amount" => {
                    let (n, e) = read_number_f64(buf, v)?;
                    p.customer_avg_amount = n;
                    Ok(Some(e))
                }
                b"tx_count_24h" => {
                    let (n, e) = read_number_u32(buf, v)?;
                    p.tx_count_24h = n;
                    Ok(Some(e))
                }
                b"known_merchants" => {
                    let (arr, e) = read_array_raw(buf, v)?;
                    p.known_merchants_buf = arr;
                    Ok(Some(e))
                }
                _ => Ok(None),
            })?;
            Ok(Some(end))
        }
        b"merchant" => {
            let end = for_each_kv(buf, vstart, |buf, k, v| match k {
                b"id" => {
                    let (s, e) = read_string(buf, v)?;
                    p.merchant_id = s;
                    Ok(Some(e))
                }
                b"mcc" => {
                    let (s, e) = read_string(buf, v)?;
                    p.merchant_mcc = s;
                    Ok(Some(e))
                }
                b"avg_amount" => {
                    let (n, e) = read_number_f64(buf, v)?;
                    p.merchant_avg_amount = n;
                    Ok(Some(e))
                }
                _ => Ok(None),
            })?;
            Ok(Some(end))
        }
        b"terminal" => {
            let end = for_each_kv(buf, vstart, |buf, k, v| match k {
                b"is_online" => {
                    let (b, e) = read_bool(buf, v)?;
                    p.is_online = b;
                    Ok(Some(e))
                }
                b"card_present" => {
                    let (b, e) = read_bool(buf, v)?;
                    p.card_present = b;
                    Ok(Some(e))
                }
                b"km_from_home" => {
                    let (n, e) = read_number_f64(buf, v)?;
                    p.km_from_home = n;
                    Ok(Some(e))
                }
                _ => Ok(None),
            })?;
            Ok(Some(end))
        }
        b"last_transaction" => {
            if is_null(buf, vstart) {
                p.has_last_tx = false;
                Ok(Some(vstart + 4))
            } else {
                p.has_last_tx = true;
                let end = for_each_kv(buf, vstart, |buf, k, v| match k {
                    b"timestamp" => {
                        let (s, e) = read_string(buf, v)?;
                        p.last_tx_stamp = parse_iso8601(s).ok_or(ParseError)?;
                        Ok(Some(e))
                    }
                    b"km_from_current" => {
                        let (n, e) = read_number_f64(buf, v)?;
                        p.last_tx_km = n;
                        Ok(Some(e))
                    }
                    _ => Ok(None),
                })?;
                Ok(Some(end))
            }
        }
        _ => Ok(None),
    })?;

    Ok(p)
}

pub fn merchant_in_known(known_raw: &[u8], merchant_id: &[u8]) -> bool {
    if merchant_id.is_empty() {
        return false;
    }
    let needle_len = merchant_id.len();
    let mut i = 0;
    while i < known_raw.len() {
        if known_raw[i] == b'"' {
            let start = i + 1;
            let mut j = start;
            while j < known_raw.len() && known_raw[j] != b'"' {
                j += 1;
            }
            if j - start == needle_len && &known_raw[start..j] == merchant_id {
                return true;
            }
            i = j + 1;
        } else {
            i += 1;
        }
    }
    false
}
