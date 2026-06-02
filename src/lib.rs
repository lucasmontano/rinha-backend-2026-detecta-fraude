#![allow(clippy::needless_range_loop)]

pub mod consts;
pub mod index;
pub mod parse;
pub mod response;
#[cfg(target_os = "linux")]
pub mod server;
pub mod time;
pub mod vectorize;

pub const DIM: usize = 14;
pub const STORE_DIM: usize = 16;
pub const K: usize = 5;
pub const SCALE: f64 = 10_000.0;

pub type QVec = [i16; STORE_DIM];
pub type QVector = QVec;

#[inline]
pub fn quantize(v: f64) -> i16 {
    if v <= -1.0 {
        return -(SCALE as i16);
    }
    if v <= 0.0 {
        return 0;
    }
    if v >= 1.0 {
        return SCALE as i16;
    }
    (v * SCALE).round() as i16
}
