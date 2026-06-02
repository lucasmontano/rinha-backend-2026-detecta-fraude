pub const MAX_AMOUNT: f64 = 10_000.0;
pub const MAX_INSTALLMENTS: f64 = 12.0;
pub const AMOUNT_VS_AVG_RATIO: f64 = 10.0;
pub const MAX_MINUTES: f64 = 1_440.0;
pub const MAX_KM: f64 = 1_000.0;
pub const MAX_TX_COUNT_24H: f64 = 20.0;
pub const MAX_MERCHANT_AVG_AMOUNT: f64 = 10_000.0;

pub const DEFAULT_MCC_RISK: f64 = 0.5;

pub const MCC_RISK: &[(&[u8; 4], f64)] = &[
    (b"5411", 0.15),
    (b"5812", 0.30),
    (b"5912", 0.20),
    (b"5944", 0.45),
    (b"7801", 0.80),
    (b"7802", 0.75),
    (b"7995", 0.85),
    (b"4511", 0.35),
    (b"5311", 0.25),
    (b"5999", 0.50),
];

#[inline]
pub fn mcc_risk_lookup(mcc: &[u8]) -> f64 {
    if mcc.len() != 4 {
        return DEFAULT_MCC_RISK;
    }
    let bytes: &[u8; 4] = mcc.try_into().unwrap();
    for (k, v) in MCC_RISK {
        if *k == bytes {
            return *v;
        }
    }
    DEFAULT_MCC_RISK
}
