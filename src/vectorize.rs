use crate::consts::*;
use crate::parse::RawPayload;
use crate::{DIM, STORE_DIM};

#[inline]
fn clamp01(v: f64) -> f64 {
    if v < 0.0 {
        0.0
    } else if v > 1.0 {
        1.0
    } else {
        v
    }
}

#[inline(always)]
fn quantize_clamped(v: f64) -> i16 {
    if v <= 0.0 {
        0
    } else if v >= 1.0 {
        crate::SCALE as i16
    } else {
        (v * crate::SCALE + 0.5) as i16
    }
}

#[inline(always)]
fn quantize_ratio_u32(v: u32, max: u32) -> i16 {
    if v == 0 {
        0
    } else if v >= max {
        crate::SCALE as i16
    } else {
        (((v * crate::SCALE as u32) + (max / 2)) / max) as i16
    }
}

pub fn vectorize_f64(p: &RawPayload<'_>) -> [f64; DIM] {
    let mut v = [0.0f64; DIM];

    v[0] = clamp01(p.amount / MAX_AMOUNT);
    v[1] = clamp01(p.installments as f64 / MAX_INSTALLMENTS);

    let ratio = if p.customer_avg_amount > 0.0 {
        p.amount / p.customer_avg_amount
    } else {
        f64::INFINITY
    };
    v[2] = clamp01(ratio / AMOUNT_VS_AVG_RATIO);

    v[3] = p.requested_at.hour as f64 / 23.0;
    v[4] = p.requested_at.weekday as f64 / 6.0;

    if p.has_last_tx {
        let minutes = (p.requested_at.epoch_minutes - p.last_tx_stamp.epoch_minutes) as f64;
        v[5] = clamp01(minutes / MAX_MINUTES);
        v[6] = clamp01(p.last_tx_km / MAX_KM);
    } else {
        v[5] = -1.0;
        v[6] = -1.0;
    }

    v[7] = clamp01(p.km_from_home / MAX_KM);
    v[8] = clamp01(p.tx_count_24h as f64 / MAX_TX_COUNT_24H);
    v[9] = if p.is_online { 1.0 } else { 0.0 };
    v[10] = if p.card_present { 1.0 } else { 0.0 };
    v[11] = if p.merchant_known { 0.0 } else { 1.0 };
    v[12] = mcc_risk_lookup(p.merchant_mcc);
    v[13] = clamp01(p.merchant_avg_amount / MAX_MERCHANT_AVG_AMOUNT);

    v
}

pub fn vectorize_q(p: &RawPayload<'_>) -> [i16; STORE_DIM] {
    let mut q = [0i16; STORE_DIM];
    q[0] = quantize_clamped(p.amount / MAX_AMOUNT);
    q[1] = quantize_ratio_u32(p.installments, MAX_INSTALLMENTS as u32);

    let ratio = if p.customer_avg_amount > 0.0 {
        p.amount / p.customer_avg_amount
    } else {
        f64::INFINITY
    };
    q[2] = quantize_clamped(ratio / AMOUNT_VS_AVG_RATIO);

    q[3] = quantize_ratio_u32(u32::from(p.requested_at.hour), 23);
    q[4] = quantize_ratio_u32(u32::from(p.requested_at.weekday), 6);

    if p.has_last_tx {
        let minutes = (p.requested_at.epoch_minutes - p.last_tx_stamp.epoch_minutes) as f64;
        q[5] = quantize_clamped(minutes / MAX_MINUTES);
        q[6] = quantize_clamped(p.last_tx_km / MAX_KM);
    } else {
        q[5] = -(crate::SCALE as i16);
        q[6] = -(crate::SCALE as i16);
    }

    q[7] = quantize_clamped(p.km_from_home / MAX_KM);
    q[8] = quantize_ratio_u32(p.tx_count_24h, MAX_TX_COUNT_24H as u32);
    q[9] = if p.is_online { crate::SCALE as i16 } else { 0 };
    q[10] = if p.card_present {
        crate::SCALE as i16
    } else {
        0
    };
    q[11] = if p.merchant_known {
        0
    } else {
        crate::SCALE as i16
    };
    q[12] = mcc_risk_q_lookup(p.merchant_mcc);
    q[13] = quantize_clamped(p.merchant_avg_amount / MAX_MERCHANT_AVG_AMOUNT);
    q
}
