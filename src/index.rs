use crate::{DIM, K, SCALE, STORE_DIM};
use std::path::Path;

pub mod stats {
    #[cfg(feature = "knn_stats")]
    use std::cell::Cell;

    #[cfg(feature = "knn_stats")]
    thread_local! {
        static NODES: Cell<u32> = Cell::new(0);
        static LEAVES: Cell<u32> = Cell::new(0);
        static BLOCKS: Cell<u32> = Cell::new(0);
        static PARTS: Cell<u32> = Cell::new(0);
        static PRIMARY_HIT: Cell<bool> = Cell::new(false);
        static EARLY_HIT: Cell<bool> = Cell::new(false);
    }

    #[inline(always)]
    pub fn inc_nodes() {
        #[cfg(feature = "knn_stats")]
        NODES.with(|c| c.set(c.get() + 1));
    }
    #[inline(always)]
    pub fn inc_leaves() {
        #[cfg(feature = "knn_stats")]
        LEAVES.with(|c| c.set(c.get() + 1));
    }
    #[inline(always)]
    pub fn inc_blocks() {
        #[cfg(feature = "knn_stats")]
        BLOCKS.with(|c| c.set(c.get() + 1));
    }
    #[inline(always)]
    pub fn inc_parts() {
        #[cfg(feature = "knn_stats")]
        PARTS.with(|c| c.set(c.get() + 1));
    }
    #[inline(always)]
    pub fn set_primary_hit() {
        #[cfg(feature = "knn_stats")]
        PRIMARY_HIT.with(|c| c.set(true));
    }
    #[inline(always)]
    pub fn set_early_hit() {
        #[cfg(feature = "knn_stats")]
        EARLY_HIT.with(|c| c.set(true));
    }

    #[cfg(feature = "knn_stats")]
    #[derive(Clone, Copy, Debug, Default)]
    pub struct QueryStats {
        pub nodes: u32,
        pub leaves: u32,
        pub blocks: u32,
        pub partitions: u32,
        pub primary_hit: bool,
        pub early_hit: bool,
    }

    #[cfg(feature = "knn_stats")]
    pub fn reset() {
        NODES.with(|c| c.set(0));
        LEAVES.with(|c| c.set(0));
        BLOCKS.with(|c| c.set(0));
        PARTS.with(|c| c.set(0));
        PRIMARY_HIT.with(|c| c.set(false));
        EARLY_HIT.with(|c| c.set(false));
    }

    #[cfg(feature = "knn_stats")]
    pub fn snapshot() -> QueryStats {
        QueryStats {
            nodes: NODES.with(|c| c.get()),
            leaves: LEAVES.with(|c| c.get()),
            blocks: BLOCKS.with(|c| c.get()),
            partitions: PARTS.with(|c| c.get()),
            primary_hit: PRIMARY_HIT.with(|c| c.get()),
            early_hit: EARLY_HIT.with(|c| c.get()),
        }
    }
}

pub const LABEL_LEGIT: u8 = 0;
pub const LABEL_FRAUD: u8 = 1;

pub const MAGIC: [u8; 8] = *b"DFKNN001";
pub const VERSION: u32 = 4;
pub const IVF_VERSION: u32 = 5;
pub const KD_PAIR_VERSION: u32 = 6;
pub const HEADER_SIZE: usize = 64;
pub const PART_SIZE: usize = 76;
pub const NODE_SIZE: usize = 80;
pub const LANES: usize = 8;
pub const BLOCK_BYTES: usize = DIM * LANES * 2;
pub const IVF_PAIRS: usize = DIM / 2;
pub const MCC_TABLE_SIZE: usize = 1024;
pub const DEFAULT_LEAF_SIZE: usize = 128;
pub const EARLY_DISTANCE_MILLI: i32 = 140;
pub const EARLY_DISTANCE_LIMIT: i64 = {
    let v = (SCALE as i32 * EARLY_DISTANCE_MILLI / 1000) as i64;
    v * v
};
pub const IVF_CLUSTER_COUNT: usize = 4096;
pub const IVF_NPROBE: usize = 12;
pub const IVF_REPAIR_CAND_LIMIT: usize = 1024;
pub const IVF_REPAIR_MIN: u8 = 1;
pub const IVF_REPAIR_MAX: u8 = 4;
pub const IVF_CONFIDENT_DISTANCE_LIMIT: i64 = EARLY_DISTANCE_LIMIT;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Header {
    pub magic: [u8; 8],
    pub version: u32,
    pub scale: u32,
    pub dim: u32,
    pub store_dim: u32,
    pub n_points: u32,
    pub part_count: u32,
    pub node_count: u32,
    pub block_count: u32,
    pub mcc_table_offset: u32,
    pub _pad: [u8; 20],
}

const _: () = assert!(std::mem::size_of::<Header>() == HEADER_SIZE);

#[derive(Clone, Copy)]
struct IvfOffsets {
    centroids: usize,
    bbox_min: usize,
    bbox_max: usize,
    offsets: usize,
    counts: usize,
    vectors: usize,
    labels: usize,
    mcc_table: usize,
    end: usize,
}

#[inline]
fn align_to(v: usize, align: usize) -> usize {
    (v + align - 1) & !(align - 1)
}

fn ivf_offsets(cluster_count: usize, block_count: usize) -> IvfOffsets {
    let centroids = HEADER_SIZE;
    let bbox_min = centroids + cluster_count * STORE_DIM * 2;
    let bbox_max = bbox_min + cluster_count * STORE_DIM * 2;
    let offsets = align_to(bbox_max + cluster_count * STORE_DIM * 2, 4);
    let counts = offsets + (cluster_count + 1) * 4;
    let vectors = align_to(counts + cluster_count * 4, 2);
    let labels = vectors + block_count * BLOCK_BYTES;
    let mcc_table = labels + block_count * LANES;
    let end = mcc_table + MCC_TABLE_SIZE * 2;
    IvfOffsets {
        centroids,
        bbox_min,
        bbox_max,
        offsets,
        counts,
        vectors,
        labels,
        mcc_table,
        end,
    }
}

#[inline(always)]
fn ivf_pair_offset(d: usize, lane: usize) -> usize {
    (d / 2) * LANES * 2 + lane * 2 + (d & 1)
}

pub struct IndexReader {
    _map: memmap2::Mmap,
    base: *const u8,
    len: usize,
    partitions_off: usize,
    nodes_off: usize,
    vectors_off: usize,
    labels_off: usize,
    mcc_table_off: usize,
    ivf_centroids_off: usize,
    ivf_min_off: usize,
    ivf_max_off: usize,
    ivf_offsets_off: usize,
    ivf_counts_off: usize,
    header: Header,
    part_by_key: [i32; 256],
}

unsafe impl Send for IndexReader {}
unsafe impl Sync for IndexReader {}

impl IndexReader {
    pub fn open(path: &Path) -> std::io::Result<Self> {
        let file = std::fs::File::open(path)?;
        let map = unsafe { memmap2::MmapOptions::new().populate().map(&file)? };
        let base = map.as_ptr();
        let len = map.len();
        if len < HEADER_SIZE {
            return Err(invalid("index too small"));
        }

        let header: Header = unsafe { std::ptr::read_unaligned(base as *const Header) };
        if header.magic != MAGIC
            || (header.version != VERSION
                && header.version != IVF_VERSION
                && header.version != KD_PAIR_VERSION)
        {
            return Err(invalid("bad magic/version"));
        }
        if header.scale != SCALE as u32
            || header.dim as usize != DIM
            || header.store_dim as usize != STORE_DIM
        {
            return Err(invalid("dim/scale mismatch"));
        }

        let mut part_by_key = [-1i32; 256];
        let (
            partitions_off,
            nodes_off,
            vectors_off,
            labels_off,
            mcc_table_off,
            ivf_centroids_off,
            ivf_min_off,
            ivf_max_off,
            ivf_offsets_off,
            ivf_counts_off,
        ) = if header.version == IVF_VERSION {
            let layout = ivf_offsets(header.part_count as usize, header.block_count as usize);
            if layout.end != len || header.mcc_table_offset as usize != layout.mcc_table {
                return Err(invalid("ivf index size mismatch"));
            }
            (
                0,
                0,
                layout.vectors,
                layout.labels,
                layout.mcc_table,
                layout.centroids,
                layout.bbox_min,
                layout.bbox_max,
                layout.offsets,
                layout.counts,
            )
        } else {
            let partitions_off = HEADER_SIZE;
            let nodes_off = partitions_off + header.part_count as usize * PART_SIZE;
            let vectors_off = nodes_off + header.node_count as usize * NODE_SIZE;
            let labels_off = vectors_off + header.block_count as usize * BLOCK_BYTES;
            let mcc_table_off = labels_off + header.block_count as usize * LANES;
            let end = mcc_table_off + MCC_TABLE_SIZE * 2;
            if end != len || header.mcc_table_offset as usize != mcc_table_off {
                return Err(invalid("index size mismatch"));
            }
            for i in 0..header.part_count as usize {
                let off = partitions_off + i * PART_SIZE;
                let key = read_u32_at(base, off);
                if (key as usize) < part_by_key.len() {
                    part_by_key[key as usize] = i as i32;
                }
            }
            (
                partitions_off,
                nodes_off,
                vectors_off,
                labels_off,
                mcc_table_off,
                0,
                0,
                0,
                0,
                0,
            )
        };

        let idx = IndexReader {
            _map: map,
            base,
            len,
            partitions_off,
            nodes_off,
            vectors_off,
            labels_off,
            mcc_table_off,
            ivf_centroids_off,
            ivf_min_off,
            ivf_max_off,
            ivf_offsets_off,
            ivf_counts_off,
            header,
            part_by_key,
        };
        idx.advise();
        idx.prefetch();
        idx.lock_if_requested();
        Ok(idx)
    }

    #[inline]
    pub fn n_points(&self) -> u32 {
        self.header.n_points
    }

    #[inline]
    pub fn part_count(&self) -> u32 {
        self.header.part_count
    }

    #[inline]
    pub fn node_count(&self) -> u32 {
        self.header.node_count
    }

    #[inline]
    pub fn block_count(&self) -> u32 {
        self.header.block_count
    }

    #[inline]
    pub fn mcc_risk(&self, mcc: u32) -> i16 {
        let idx = (mcc as usize) % MCC_TABLE_SIZE;
        let off = self.mcc_table_off + idx * 2;
        read_i16_at(self.base, off)
    }

    #[inline]
    fn partitions_ptr(&self) -> *const u8 {
        unsafe { self.base.add(self.partitions_off) }
    }

    #[inline]
    fn nodes_ptr(&self) -> *const u8 {
        unsafe { self.base.add(self.nodes_off) }
    }

    #[inline]
    fn vectors_ptr(&self) -> *const i16 {
        unsafe { self.base.add(self.vectors_off) as *const i16 }
    }

    #[inline]
    fn labels_ptr(&self) -> *const u8 {
        unsafe { self.base.add(self.labels_off) }
    }

    #[inline]
    fn ivf_centroids_ptr(&self) -> *const u8 {
        unsafe { self.base.add(self.ivf_centroids_off) }
    }

    #[inline]
    fn ivf_min_ptr(&self) -> *const u8 {
        unsafe { self.base.add(self.ivf_min_off) }
    }

    #[inline]
    fn ivf_max_ptr(&self) -> *const u8 {
        unsafe { self.base.add(self.ivf_max_off) }
    }

    #[inline]
    fn is_ivf(&self) -> bool {
        self.header.version == IVF_VERSION
    }

    #[inline]
    fn is_kd_pair(&self) -> bool {
        self.header.version == KD_PAIR_VERSION
    }

    #[inline]
    fn ivf_offset(&self, cluster: usize) -> u32 {
        read_u32_at(self.base, self.ivf_offsets_off + cluster * 4)
    }

    #[inline]
    fn ivf_count(&self, cluster: usize) -> u32 {
        read_u32_at(self.base, self.ivf_counts_off + cluster * 4)
    }

    #[inline]
    fn part_by_key(&self, key: u32) -> i32 {
        self.part_by_key[(key & 0xff) as usize]
    }

    #[inline]
    pub fn fraud_count(&self, query: &[i16; STORE_DIM]) -> u8 {
        #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
        {
            if self.is_ivf() {
                return unsafe { fraud_count_ivf_index_avx2(self, query) };
            }
            if self.is_kd_pair() {
                return unsafe { fraud_count_pair_avx2(self, query) };
            }
            return unsafe { fraud_count_exact_avx2(self, query) };
        }
        #[cfg(all(target_arch = "x86_64", not(target_feature = "avx2")))]
        {
            if std::is_x86_feature_detected!("avx2") {
                if self.is_ivf() {
                    return unsafe { fraud_count_ivf_index_avx2(self, query) };
                }
                if self.is_kd_pair() {
                    return unsafe { fraud_count_pair_avx2(self, query) };
                }
                return unsafe { fraud_count_exact_avx2(self, query) };
            }
        }
        if self.is_ivf() {
            return fraud_count_ivf_index_scalar(self, query);
        }
        fraud_count_scalar(self, query)
    }

    #[inline]
    pub fn fraud_count_exact(&self, query: &[i16; STORE_DIM]) -> u8 {
        if self.is_ivf() {
            return self.fraud_count(query);
        }
        #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
        {
            if self.is_kd_pair() {
                return unsafe { fraud_count_pair_avx2(self, query) };
            }
            return unsafe { fraud_count_exact_avx2(self, query) };
        }
        #[cfg(all(target_arch = "x86_64", not(target_feature = "avx2")))]
        {
            if std::is_x86_feature_detected!("avx2") {
                if self.is_kd_pair() {
                    return unsafe { fraud_count_pair_avx2(self, query) };
                }
                return unsafe { fraud_count_exact_avx2(self, query) };
            }
        }
        fraud_count_scalar(self, query)
    }

    #[cfg(target_os = "linux")]
    fn advise(&self) {
        const MADV_HUGEPAGE: libc::c_int = 14;
        const MADV_RANDOM: libc::c_int = 1;
        const MADV_WILLNEED: libc::c_int = 3;
        unsafe {
            libc::madvise(self.base as *mut _, self.len, MADV_HUGEPAGE);
            let hot_start = self.vectors_off;
            let hot_len = self.len - hot_start;
            libc::madvise(self.base.add(hot_start) as *mut _, hot_len, MADV_HUGEPAGE);
            libc::madvise(self.base.add(hot_start) as *mut _, hot_len, MADV_RANDOM);
            libc::madvise(self.base.add(hot_start) as *mut _, hot_len, MADV_WILLNEED);
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn advise(&self) {}

    fn prefetch(&self) {
        const PAGE: usize = 4096;
        let mut acc = 0u8;
        let mut i = 0usize;
        while i < self.len {
            acc ^= unsafe { std::ptr::read_volatile(self.base.add(i)) };
            i += PAGE;
        }
        if acc == 0xFE {
            eprintln!("[index] prefetch sentinel hit");
        }
    }

    #[cfg(target_os = "linux")]
    fn lock_if_requested(&self) {
        if std::env::var("INDEX_MLOCK").ok().as_deref() != Some("1") {
            return;
        }
        let rc = unsafe { libc::mlock(self.base as *const _, self.len) };
        if rc != 0 {
            eprintln!("[index] mlock failed: {}", std::io::Error::last_os_error());
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn lock_if_requested(&self) {}
}

fn invalid(msg: &'static str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, msg)
}

#[inline]
fn read_i16_at(base: *const u8, off: usize) -> i16 {
    unsafe { std::ptr::read_unaligned(base.add(off) as *const i16) }
}

#[inline]
fn read_i32_at(base: *const u8, off: usize) -> i32 {
    unsafe { std::ptr::read_unaligned(base.add(off) as *const i32) }
}

#[inline]
fn read_u32_at(base: *const u8, off: usize) -> u32 {
    unsafe { std::ptr::read_unaligned(base.add(off) as *const u32) }
}

#[inline]
pub fn partition_key(v: &[i16; STORE_DIM]) -> u32 {
    let mut key = 0u32;
    if v[5] >= 0 {
        key |= 1 << 0;
    }
    if v[9] > 0 {
        key |= 1 << 1;
    }
    if v[10] > 0 {
        key |= 1 << 2;
    }
    if v[11] > 0 {
        key |= 1 << 3;
    }
    let mr = v[12];
    if mr <= 2047 {
    } else if mr <= 4095 {
        key |= 1 << 4;
    } else if mr <= 6143 {
        key |= 2 << 4;
    } else {
        key |= 3 << 4;
    }
    if v[2] > 4096 {
        key |= 1 << 6;
    }
    if v[8] > 2048 {
        key |= 1 << 7;
    }
    key
}

#[inline]
fn lower_bound_dim(q: i16, lo: i16, hi: i16) -> i64 {
    let diff = if q < lo {
        lo as i64 - q as i64
    } else if q > hi {
        q as i64 - hi as i64
    } else {
        0
    };
    diff * diff
}

#[inline]
pub fn lower_bound_vec(
    q: &[i16; STORE_DIM],
    min: &[i16; STORE_DIM],
    max: &[i16; STORE_DIM],
) -> i64 {
    let mut acc = 0i64;
    let mut d = 0usize;
    while d < DIM {
        acc += lower_bound_dim(q[d], min[d], max[d]);
        d += 1;
    }
    acc
}

#[inline(always)]
fn lower_bound_ptr_scalar(q: &[i16; STORE_DIM], min: *const i16, max: *const i16) -> i64 {
    let mut acc = 0i64;
    let mut d = 0usize;
    while d < DIM {
        let lo = unsafe { *min.add(d) };
        let hi = unsafe { *max.add(d) };
        acc += lower_bound_dim(q[d], lo, hi);
        d += 1;
    }
    acc
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn lower_bound_ptr_avx2(q: &[i16; STORE_DIM], min: *const i16, max: *const i16) -> i64 {
    use std::arch::x86_64::*;

    let qv = _mm256_loadu_si256(q.as_ptr() as *const __m256i);
    let mn = _mm256_loadu_si256(min as *const __m256i);
    let mx = _mm256_loadu_si256(max as *const __m256i);
    let zero = _mm256_setzero_si256();
    let below = _mm256_max_epi16(_mm256_sub_epi16(mn, qv), zero);
    let above = _mm256_max_epi16(_mm256_sub_epi16(qv, mx), zero);
    let gap = _mm256_max_epi16(below, above);
    let sq_pairs = _mm256_madd_epi16(gap, gap);
    let mut vals = [0i32; LANES];
    _mm256_storeu_si256(vals.as_mut_ptr() as *mut __m256i, sq_pairs);
    vals[0] as i64
        + vals[1] as i64
        + vals[2] as i64
        + vals[3] as i64
        + vals[4] as i64
        + vals[5] as i64
        + vals[6] as i64
        + vals[7] as i64
}

#[inline]
fn read_partition_meta(idx: &IndexReader, part_idx: usize) -> (i32, i32) {
    let p = idx.partitions_ptr();
    let off = part_idx * PART_SIZE;
    let root = read_i32_at(p, off + 4);
    let len = read_i32_at(p, off + 8);
    (root, len)
}

#[inline]
fn read_node_meta(idx: &IndexReader, node_idx: usize) -> (i32, i32, i32, i32) {
    let p = idx.nodes_ptr();
    let off = node_idx * NODE_SIZE;
    let left = read_i32_at(p, off);
    let right = read_i32_at(p, off + 4);
    let start = read_i32_at(p, off + 8);
    let len = read_i32_at(p, off + 12);
    (left, right, start, len)
}

#[inline(always)]
fn partition_bounds_ptr(idx: &IndexReader, part_idx: usize) -> (*const i16, *const i16) {
    let p = idx.partitions_ptr();
    let off = part_idx * PART_SIZE;
    unsafe { (p.add(off + 12) as *const i16, p.add(off + 44) as *const i16) }
}

#[inline(always)]
fn node_bounds_ptr(idx: &IndexReader, node_idx: usize) -> (*const i16, *const i16) {
    let p = idx.nodes_ptr();
    let off = node_idx * NODE_SIZE;
    unsafe { (p.add(off + 16) as *const i16, p.add(off + 48) as *const i16) }
}

#[inline(always)]
fn lower_bound_partition_scalar(idx: &IndexReader, part_idx: usize, q: &[i16; STORE_DIM]) -> i64 {
    let (min, max) = partition_bounds_ptr(idx, part_idx);
    lower_bound_ptr_scalar(q, min, max)
}

#[inline(always)]
fn lower_bound_node_scalar(idx: &IndexReader, node_idx: usize, q: &[i16; STORE_DIM]) -> i64 {
    let (min, max) = node_bounds_ptr(idx, node_idx);
    lower_bound_ptr_scalar(q, min, max)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn lower_bound_partition_avx2(
    idx: &IndexReader,
    part_idx: usize,
    q: &[i16; STORE_DIM],
) -> i64 {
    let (min, max) = partition_bounds_ptr(idx, part_idx);
    lower_bound_ptr_avx2(q, min, max)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn lower_bound_node_avx2(idx: &IndexReader, node_idx: usize, q: &[i16; STORE_DIM]) -> i64 {
    let (min, max) = node_bounds_ptr(idx, node_idx);
    lower_bound_ptr_avx2(q, min, max)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn lower_bound_ivf_cluster_avx2(
    idx: &IndexReader,
    cluster: usize,
    q: &[i16; STORE_DIM],
) -> i64 {
    let min = idx.ivf_min_ptr().add(cluster * STORE_DIM * 2) as *const i16;
    let max = idx.ivf_max_ptr().add(cluster * STORE_DIM * 2) as *const i16;
    lower_bound_ptr_avx2(q, min, max)
}

#[inline]
fn read_qv(base: *const u8, off: usize) -> [i16; STORE_DIM] {
    let mut v = [0i16; STORE_DIM];
    for i in 0..STORE_DIM {
        v[i] = read_i16_at(base, off + i * 2);
    }
    v
}

#[inline(always)]
fn insert_best(dist: i64, label: u8, dists: &mut [i64; K], labels: &mut [u8; K]) {
    if dist >= dists[K - 1] {
        return;
    }
    let mut pos = K - 1;
    while pos > 0 && dist < dists[pos - 1] {
        dists[pos] = dists[pos - 1];
        labels[pos] = labels[pos - 1];
        pos -= 1;
    }
    dists[pos] = dist;
    labels[pos] = label;
}

#[inline(always)]
fn sum_labels(labels: &[u8; K]) -> u8 {
    let mut n = 0u8;
    for &l in labels {
        n += l;
    }
    n
}

#[inline(always)]
fn early_done(_best: &[i64; K]) -> bool {
    false
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn distance_qv_avx2(a: &[i16; STORE_DIM], b: &[i16; STORE_DIM]) -> i64 {
    use std::arch::x86_64::*;

    let av = _mm256_loadu_si256(a.as_ptr() as *const __m256i);
    let bv = _mm256_loadu_si256(b.as_ptr() as *const __m256i);
    let diff = _mm256_sub_epi16(av, bv);
    let sq_pairs = _mm256_madd_epi16(diff, diff);
    let mut vals = [0i32; LANES];
    _mm256_storeu_si256(vals.as_mut_ptr() as *mut __m256i, sq_pairs);
    vals[0] as i64
        + vals[1] as i64
        + vals[2] as i64
        + vals[3] as i64
        + vals[4] as i64
        + vals[5] as i64
        + vals[6] as i64
        + vals[7] as i64
}

#[inline(always)]
fn distance_qv_scalar(a: &[i16; STORE_DIM], b: &[i16; STORE_DIM]) -> i64 {
    let mut acc = 0i64;
    for d in 0..DIM {
        let diff = a[d] as i64 - b[d] as i64;
        acc += diff * diff;
    }
    acc
}

#[inline(always)]
fn insert_cluster_probe<const N: usize>(
    cluster: usize,
    dist: i64,
    probes: &mut [(usize, i64); N],
    count: &mut usize,
) {
    if *count == N && dist >= probes[N - 1].1 {
        return;
    }
    let mut pos = if *count < N {
        let pos = *count;
        *count += 1;
        pos
    } else {
        N - 1
    };
    while pos > 0 && dist < probes[pos - 1].1 {
        probes[pos] = probes[pos - 1];
        pos -= 1;
    }
    probes[pos] = (cluster, dist);
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fraud_count_ivf_index_avx2(idx: &IndexReader, query: &[i16; STORE_DIM]) -> u8 {
    use std::arch::x86_64::*;

    let mut best_dists = [i64::MAX; K];
    let mut best_labels = [0u8; K];
    let mut probes = [(usize::MAX, i64::MAX); IVF_NPROBE];
    let mut probe_count = 0usize;
    let centroids = idx.ivf_centroids_ptr();
    let mut q_pairs = [_mm256_setzero_si256(); IVF_PAIRS];
    for p in 0..IVF_PAIRS {
        let lo = query[p * 2] as u16 as u32;
        let hi = query[p * 2 + 1] as u16 as u32;
        q_pairs[p] = _mm256_set1_epi32((lo | (hi << 16)) as i32);
    }

    for c in 0..idx.part_count() as usize {
        let centroid = read_qv(centroids, c * STORE_DIM * 2);
        let dist = distance_qv_avx2(query, &centroid);
        insert_cluster_probe(c, dist, &mut probes, &mut probe_count);
    }

    for &(cluster, _dist) in &probes[..probe_count] {
        scan_ivf_cluster_avx2(idx, cluster, &q_pairs, &mut best_dists, &mut best_labels);
    }

    let mut count = sum_labels(&best_labels);
    if (IVF_REPAIR_MIN..=IVF_REPAIR_MAX).contains(&count)
        || best_dists[K - 1] > IVF_CONFIDENT_DISTANCE_LIMIT
    {
        repair_ivf_avx2(
            idx,
            query,
            &q_pairs,
            &probes[..probe_count],
            &mut best_dists,
            &mut best_labels,
        );
        count = sum_labels(&best_labels);
    }
    count
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn repair_ivf_avx2(
    idx: &IndexReader,
    query: &[i16; STORE_DIM],
    q_pairs: &[std::arch::x86_64::__m256i; IVF_PAIRS],
    skip: &[(usize, i64)],
    best_dists: &mut [i64; K],
    best_labels: &mut [u8; K],
) {
    let mut cands = [(usize::MAX, i64::MAX); IVF_REPAIR_CAND_LIMIT];
    let mut cand_count = 0usize;

    for c in 0..idx.part_count() as usize {
        let mut seen = false;
        for &(probe, _dist) in skip {
            if probe == c {
                seen = true;
                break;
            }
        }
        if seen || idx.ivf_count(c) == 0 {
            continue;
        }
        let lb = lower_bound_ivf_cluster_avx2(idx, c, query);
        if lb < best_dists[K - 1] {
            insert_cluster_probe(c, lb, &mut cands, &mut cand_count);
        }
    }

    for &(cluster, lb) in &cands[..cand_count] {
        if lb >= best_dists[K - 1] {
            break;
        }
        scan_ivf_cluster_avx2(idx, cluster, q_pairs, best_dists, best_labels);
        let count = sum_labels(best_labels);
        if count < IVF_REPAIR_MIN || count > IVF_REPAIR_MAX {
            break;
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn scan_ivf_cluster_avx2(
    idx: &IndexReader,
    cluster: usize,
    q_pairs: &[std::arch::x86_64::__m256i; IVF_PAIRS],
    best_dists: &mut [i64; K],
    best_labels: &mut [u8; K],
) {
    let start_block = idx.ivf_offset(cluster) as usize;
    let end_block = idx.ivf_offset(cluster + 1) as usize;
    let total_len = idx.ivf_count(cluster) as usize;
    if total_len == 0 {
        return;
    }

    let labels_ptr = idx.labels_ptr();
    let vectors_ptr = idx.vectors_ptr();

    for (i, block_idx) in (start_block..end_block).enumerate() {
        let labels_base = block_idx * LANES;
        let block_off_i16 = block_idx * DIM * LANES;
        let dists = distance_ivf_block8(vectors_ptr, block_off_i16, q_pairs);
        let lane_count = (total_len - i * LANES).min(LANES);
        for (lane, &dist) in dists.iter().enumerate().take(lane_count) {
            if dist < best_dists[K - 1] {
                let label = *labels_ptr.add(labels_base + lane);
                insert_best(dist, label, best_dists, best_labels);
            }
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn distance_ivf_block8(
    vectors: *const i16,
    block_off_i16: usize,
    q_pairs: &[std::arch::x86_64::__m256i; IVF_PAIRS],
) -> [i64; LANES] {
    use std::arch::x86_64::*;

    let base = vectors.add(block_off_i16);
    let mut acc = _mm256_setzero_si256();
    for p in 0..IVF_PAIRS {
        let packed = _mm256_loadu_si256(base.add(p * LANES * 2) as *const __m256i);
        let diff = _mm256_sub_epi16(q_pairs[p], packed);
        acc = _mm256_add_epi32(acc, _mm256_madd_epi16(diff, diff));
    }
    let mut vals = [0u32; LANES];
    _mm256_storeu_si256(vals.as_mut_ptr() as *mut __m256i, acc);
    [
        vals[0] as i64,
        vals[1] as i64,
        vals[2] as i64,
        vals[3] as i64,
        vals[4] as i64,
        vals[5] as i64,
        vals[6] as i64,
        vals[7] as i64,
    ]
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn distance_pair_block8(
    vectors: *const i16,
    block_off_i16: usize,
    q_pairs: &[std::arch::x86_64::__m256i; IVF_PAIRS],
) -> [i64; LANES] {
    use std::arch::x86_64::*;

    let base = vectors.add(block_off_i16);
    let mut acc = _mm256_setzero_si256();
    for p in 0..IVF_PAIRS {
        let packed = _mm256_loadu_si256(base.add(p * LANES * 2) as *const __m256i);
        let diff = _mm256_sub_epi16(q_pairs[p], packed);
        acc = _mm256_add_epi32(acc, _mm256_madd_epi16(diff, diff));
    }

    let mut vals = [0i32; LANES];
    _mm256_storeu_si256(vals.as_mut_ptr() as *mut __m256i, acc);
    [
        vals[0] as i64,
        vals[1] as i64,
        vals[2] as i64,
        vals[3] as i64,
        vals[4] as i64,
        vals[5] as i64,
        vals[6] as i64,
        vals[7] as i64,
    ]
}

fn fraud_count_ivf_index_scalar(idx: &IndexReader, query: &[i16; STORE_DIM]) -> u8 {
    let mut best_dists = [i64::MAX; K];
    let mut best_labels = [0u8; K];
    let mut probes = [(usize::MAX, i64::MAX); IVF_NPROBE];
    let mut probe_count = 0usize;
    let centroids = idx.ivf_centroids_ptr();

    for c in 0..idx.part_count() as usize {
        let centroid = read_qv(centroids, c * STORE_DIM * 2);
        let dist = distance_qv_scalar(query, &centroid);
        insert_cluster_probe(c, dist, &mut probes, &mut probe_count);
    }
    for &(cluster, _dist) in &probes[..probe_count] {
        scan_ivf_cluster_scalar(idx, cluster, query, &mut best_dists, &mut best_labels);
    }
    sum_labels(&best_labels)
}

fn scan_ivf_cluster_scalar(
    idx: &IndexReader,
    cluster: usize,
    query: &[i16; STORE_DIM],
    best_dists: &mut [i64; K],
    best_labels: &mut [u8; K],
) {
    let start_block = idx.ivf_offset(cluster) as usize;
    let end_block = idx.ivf_offset(cluster + 1) as usize;
    let total_len = idx.ivf_count(cluster) as usize;
    let labels_ptr = idx.labels_ptr();
    let vectors_ptr = idx.vectors_ptr();
    for (i, block_idx) in (start_block..end_block).enumerate() {
        let lane_count = (total_len - i * LANES).min(LANES);
        for lane in 0..lane_count {
            let mut dist = 0i64;
            let block_off = block_idx * DIM * LANES;
            for d in 0..DIM {
                let v = unsafe { *vectors_ptr.add(block_off + ivf_pair_offset(d, lane)) };
                let diff = v as i64 - query[d] as i64;
                dist += diff * diff;
            }
            if dist < best_dists[K - 1] {
                let label = unsafe { *labels_ptr.add(block_idx * LANES + lane) };
                insert_best(dist, label, best_dists, best_labels);
            }
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn query_pairs_avx2(query: &[i16; STORE_DIM]) -> [std::arch::x86_64::__m256i; IVF_PAIRS] {
    use std::arch::x86_64::*;

    let mut q_pairs = [_mm256_setzero_si256(); IVF_PAIRS];
    for p in 0..IVF_PAIRS {
        let lo = query[p * 2] as u16 as u32;
        let hi = query[p * 2 + 1] as u16 as u32;
        q_pairs[p] = _mm256_set1_epi32((lo | (hi << 16)) as i32);
    }
    q_pairs
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fraud_count_pair_avx2(idx: &IndexReader, query: &[i16; STORE_DIM]) -> u8 {
    let q_pairs = query_pairs_avx2(query);
    let mut best_dists = [i64::MAX; K];
    let mut best_labels = [0u8; K];

    let key = partition_key(query);
    let primary = idx.part_by_key(key);
    if primary >= 0 {
        let (root, _len) = read_partition_meta(idx, primary as usize);
        stats::inc_parts();
        if search_node_pair_avx2(
            idx,
            root,
            0,
            query,
            &q_pairs,
            &mut best_dists,
            &mut best_labels,
        ) {
            stats::set_primary_hit();
            stats::set_early_hit();
            return sum_labels(&best_labels);
        }
    }

    let mut probes = [(0i32, 0i64); 256];
    let mut n = 0usize;
    for p in 0..idx.part_count() as i32 {
        if p == primary {
            continue;
        }
        let lb = lower_bound_partition_avx2(idx, p as usize, query);
        if lb >= best_dists[K - 1] {
            continue;
        }
        probes[n] = (p, lb);
        n += 1;
    }
    probes[..n].sort_unstable_by_key(|&(_, lb)| lb);

    for &(part_idx, lb) in &probes[..n] {
        if lb >= best_dists[K - 1] {
            break;
        }
        let (root, _len) = read_partition_meta(idx, part_idx as usize);
        stats::inc_parts();
        if search_node_pair_avx2(
            idx,
            root,
            lb,
            query,
            &q_pairs,
            &mut best_dists,
            &mut best_labels,
        ) {
            stats::set_early_hit();
            break;
        }
    }

    sum_labels(&best_labels)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn search_node_pair_avx2(
    idx: &IndexReader,
    root: i32,
    root_bound: i64,
    query: &[i16; STORE_DIM],
    q_pairs: &[std::arch::x86_64::__m256i; IVF_PAIRS],
    best_dists: &mut [i64; K],
    best_labels: &mut [u8; K],
) -> bool {
    if root < 0 || root as u32 >= idx.node_count() {
        return false;
    }

    let mut stack_node = [0i32; 128];
    let mut stack_bound = [0i64; 128];
    let mut sp = 0usize;
    let mut current = root;
    let mut current_bound = root_bound;

    loop {
        if current_bound < best_dists[K - 1] {
            let (left, right, start, len) = read_node_meta(idx, current as usize);
            stats::inc_nodes();
            if left < 0 {
                if scan_leaf_pair_avx2(idx, start, len, q_pairs, best_dists, best_labels) {
                    return true;
                }
            } else {
                let lb = lower_bound_node_avx2(idx, left as usize, query);
                let rb = lower_bound_node_avx2(idx, right as usize, query);
                let (near, near_b, far, far_b) = if lb <= rb {
                    (left, lb, right, rb)
                } else {
                    (right, rb, left, lb)
                };
                if far_b < best_dists[K - 1] && sp < stack_node.len() {
                    stack_node[sp] = far;
                    stack_bound[sp] = far_b;
                    sp += 1;
                }
                current = near;
                current_bound = near_b;
                continue;
            }
        }

        if sp == 0 {
            break;
        }
        sp -= 1;
        current = stack_node[sp];
        current_bound = stack_bound[sp];
    }
    early_done(best_dists)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn scan_leaf_pair_avx2(
    idx: &IndexReader,
    start_block: i32,
    len: i32,
    q_pairs: &[std::arch::x86_64::__m256i; IVF_PAIRS],
    best_dists: &mut [i64; K],
    best_labels: &mut [u8; K],
) -> bool {
    let blocks = (len as usize).div_ceil(LANES);
    let labels_ptr = idx.labels_ptr();
    let vectors_ptr = idx.vectors_ptr();
    let total_len = len as usize;
    stats::inc_leaves();

    for b in 0..blocks {
        stats::inc_blocks();
        let block_idx = start_block as usize + b;
        let labels_base = block_idx * LANES;
        let block_off_i16 = block_idx * DIM * LANES;
        let dists = distance_pair_block8(vectors_ptr, block_off_i16, q_pairs);
        let lane_count = (total_len - b * LANES).min(LANES);
        for (lane, &d) in dists.iter().enumerate().take(lane_count) {
            if d < best_dists[K - 1] {
                let label = *labels_ptr.add(labels_base + lane);
                insert_best(d, label, best_dists, best_labels);
            }
        }
        if early_done(best_dists) {
            return true;
        }
    }
    false
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fraud_count_exact_avx2(idx: &IndexReader, query: &[i16; STORE_DIM]) -> u8 {
    let mut best_dists = [i64::MAX; K];
    let mut best_labels = [0u8; K];

    let key = partition_key(query);
    let primary = idx.part_by_key(key);
    if primary >= 0 {
        let (root, _len) = read_partition_meta(idx, primary as usize);
        if search_node_avx2(idx, root, 0, query, &mut best_dists, &mut best_labels) {
            return sum_labels(&best_labels);
        }
    }

    let mut probes = [(0i32, 0i64); 256];
    let mut n = 0usize;
    for p in 0..idx.part_count() as i32 {
        if p == primary {
            continue;
        }
        let lb = lower_bound_partition_avx2(idx, p as usize, query);
        if lb >= best_dists[K - 1] {
            continue;
        }
        probes[n] = (p, lb);
        n += 1;
    }
    probes[..n].sort_unstable_by_key(|&(_, lb)| lb);

    for &(part_idx, lb) in &probes[..n] {
        if lb >= best_dists[K - 1] {
            break;
        }
        let (root, _len) = read_partition_meta(idx, part_idx as usize);
        if search_node_avx2(idx, root, lb, query, &mut best_dists, &mut best_labels) {
            break;
        }
    }

    sum_labels(&best_labels)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn search_node_avx2(
    idx: &IndexReader,
    root: i32,
    root_bound: i64,
    query: &[i16; STORE_DIM],
    best_dists: &mut [i64; K],
    best_labels: &mut [u8; K],
) -> bool {
    if root < 0 || root as u32 >= idx.node_count() {
        return false;
    }

    let mut stack_node = [0i32; 128];
    let mut stack_bound = [0i64; 128];
    let mut sp = 0usize;
    let mut current = root;
    let mut current_bound = root_bound;

    loop {
        if current_bound < best_dists[K - 1] {
            let (left, right, start, len) = read_node_meta(idx, current as usize);
            if left < 0 {
                if scan_leaf_avx2(idx, start, len, query, best_dists, best_labels) {
                    return true;
                }
            } else {
                let lb = lower_bound_node_avx2(idx, left as usize, query);
                let rb = lower_bound_node_avx2(idx, right as usize, query);
                let (near, near_b, far, far_b) = if lb <= rb {
                    (left, lb, right, rb)
                } else {
                    (right, rb, left, lb)
                };
                if far_b < best_dists[K - 1] && sp < stack_node.len() {
                    stack_node[sp] = far;
                    stack_bound[sp] = far_b;
                    sp += 1;
                }
                current = near;
                current_bound = near_b;
                continue;
            }
        }

        if sp == 0 {
            break;
        }
        sp -= 1;
        current = stack_node[sp];
        current_bound = stack_bound[sp];
    }
    early_done(best_dists)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn scan_leaf_avx2(
    idx: &IndexReader,
    start_block: i32,
    len: i32,
    query: &[i16; STORE_DIM],
    best_dists: &mut [i64; K],
    best_labels: &mut [u8; K],
) -> bool {
    use std::arch::x86_64::*;

    let blocks = (len as usize).div_ceil(LANES);
    let labels_ptr = idx.labels_ptr();
    let vectors_ptr = idx.vectors_ptr();

    let pair_layout = idx.is_kd_pair();
    let mut q_broadcast = [_mm256_setzero_si256(); DIM];
    if !pair_layout {
        for d in 0..DIM {
            q_broadcast[d] = _mm256_set1_epi32(query[d] as i32);
        }
    }
    let mut q_pairs = [_mm256_setzero_si256(); IVF_PAIRS];
    if pair_layout {
        for p in 0..IVF_PAIRS {
            let lo = query[p * 2] as u16 as u32;
            let hi = query[p * 2 + 1] as u16 as u32;
            q_pairs[p] = _mm256_set1_epi32((lo | (hi << 16)) as i32);
        }
    }

    let total_len = len as usize;
    for b in 0..blocks {
        let block_idx = start_block as usize + b;
        let labels_base = block_idx * LANES;
        let block_off_i16 = block_idx * DIM * LANES;
        let dists = if pair_layout {
            distance_pair_block8(vectors_ptr, block_off_i16, &q_pairs)
        } else {
            distance_block8(vectors_ptr, block_off_i16, &q_broadcast)
        };
        let lane_count = (total_len - b * LANES).min(LANES);
        for (lane, &d) in dists.iter().enumerate().take(lane_count) {
            if d < best_dists[K - 1] {
                let label = *labels_ptr.add(labels_base + lane);
                insert_best(d, label, best_dists, best_labels);
            }
        }
        if early_done(best_dists) {
            return true;
        }
    }
    false
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn distance_block8(
    vectors: *const i16,
    block_off_i16: usize,
    q: &[std::arch::x86_64::__m256i; DIM],
) -> [i64; LANES] {
    use std::arch::x86_64::*;

    let mut acc_lo = _mm256_setzero_si256();
    let mut acc_hi = _mm256_setzero_si256();
    let base = vectors.add(block_off_i16);
    for (d, qd) in q.iter().enumerate().take(DIM) {
        let packed = _mm_loadu_si128(base.add(d * LANES) as *const __m128i);
        let values = _mm256_cvtepi16_epi32(packed);
        let diff = _mm256_sub_epi32(values, *qd);
        let sq = _mm256_mullo_epi32(diff, diff);
        let sq_lo = _mm256_castsi256_si128(sq);
        let sq_hi = _mm256_extracti128_si256(sq, 1);
        acc_lo = _mm256_add_epi64(acc_lo, _mm256_cvtepi32_epi64(sq_lo));
        acc_hi = _mm256_add_epi64(acc_hi, _mm256_cvtepi32_epi64(sq_hi));
    }
    let mut out = [0i64; LANES];
    _mm256_storeu_si256(out.as_mut_ptr() as *mut __m256i, acc_lo);
    _mm256_storeu_si256(out.as_mut_ptr().add(4) as *mut __m256i, acc_hi);
    out
}

fn fraud_count_scalar(idx: &IndexReader, query: &[i16; STORE_DIM]) -> u8 {
    let mut best_dists = [i64::MAX; K];
    let mut best_labels = [0u8; K];

    let key = partition_key(query);
    let primary = idx.part_by_key(key);
    if primary >= 0 {
        let (root, _len) = read_partition_meta(idx, primary as usize);
        if search_node_scalar(idx, root, 0, query, &mut best_dists, &mut best_labels) {
            return sum_labels(&best_labels);
        }
    }

    let mut probes = [(0i32, 0i64); 256];
    let mut n = 0usize;
    for p in 0..idx.part_count() as i32 {
        if p == primary {
            continue;
        }
        let lb = lower_bound_partition_scalar(idx, p as usize, query);
        if lb >= best_dists[K - 1] {
            continue;
        }
        probes[n] = (p, lb);
        n += 1;
    }
    probes[..n].sort_unstable_by_key(|&(_, lb)| lb);

    for &(part_idx, lb) in &probes[..n] {
        if lb >= best_dists[K - 1] {
            break;
        }
        let (root, _len) = read_partition_meta(idx, part_idx as usize);
        if search_node_scalar(idx, root, lb, query, &mut best_dists, &mut best_labels) {
            break;
        }
    }
    sum_labels(&best_labels)
}

fn search_node_scalar(
    idx: &IndexReader,
    root: i32,
    root_bound: i64,
    query: &[i16; STORE_DIM],
    best_dists: &mut [i64; K],
    best_labels: &mut [u8; K],
) -> bool {
    if root < 0 || root as u32 >= idx.node_count() {
        return false;
    }

    let mut stack_node = [0i32; 128];
    let mut stack_bound = [0i64; 128];
    let mut sp = 0usize;
    let mut current = root;
    let mut current_bound = root_bound;

    loop {
        if current_bound < best_dists[K - 1] {
            let (left, right, start, len) = read_node_meta(idx, current as usize);
            if left < 0 {
                if scan_leaf_scalar(idx, start, len, query, best_dists, best_labels) {
                    return true;
                }
            } else {
                let lb = lower_bound_node_scalar(idx, left as usize, query);
                let rb = lower_bound_node_scalar(idx, right as usize, query);
                let (near, near_b, far, far_b) = if lb <= rb {
                    (left, lb, right, rb)
                } else {
                    (right, rb, left, lb)
                };
                if far_b < best_dists[K - 1] && sp < stack_node.len() {
                    stack_node[sp] = far;
                    stack_bound[sp] = far_b;
                    sp += 1;
                }
                current = near;
                current_bound = near_b;
                continue;
            }
        }

        if sp == 0 {
            break;
        }
        sp -= 1;
        current = stack_node[sp];
        current_bound = stack_bound[sp];
    }
    early_done(best_dists)
}

fn scan_leaf_scalar(
    idx: &IndexReader,
    start_block: i32,
    len: i32,
    query: &[i16; STORE_DIM],
    best_dists: &mut [i64; K],
    best_labels: &mut [u8; K],
) -> bool {
    let blocks = (len as usize).div_ceil(LANES);
    let labels_ptr = idx.labels_ptr();
    let vectors_ptr = idx.vectors_ptr();
    let total_len = len as usize;
    for b in 0..blocks {
        let block_idx = start_block as usize + b;
        let lane_count = (total_len - b * LANES).min(LANES);
        for lane in 0..lane_count {
            let mut dist = 0i64;
            let block_off = block_idx * DIM * LANES;
            for d in 0..DIM {
                let off = if idx.is_kd_pair() {
                    ivf_pair_offset(d, lane)
                } else {
                    d * LANES + lane
                };
                let v = unsafe { *vectors_ptr.add(block_off + off) };
                let diff = v as i64 - query[d] as i64;
                dist += diff * diff;
            }
            if dist < best_dists[K - 1] {
                let label = unsafe { *labels_ptr.add(block_idx * LANES + lane) };
                insert_best(dist, label, best_dists, best_labels);
            }
        }
        if early_done(best_dists) {
            return true;
        }
    }
    false
}

#[cfg(feature = "builder")]
pub mod build {
    use super::*;
    use crate::consts::{DEFAULT_MCC_RISK, MCC_RISK};
    use crate::quantize;
    use std::cmp::Ordering;
    use std::collections::BinaryHeap;
    use std::io::Write;
    use std::path::Path;

    pub struct Builder {
        vectors: Vec<[i16; STORE_DIM]>,
        labels: Vec<u8>,
    }

    #[derive(Clone, Copy)]
    struct BuildNode {
        left: i32,
        right: i32,
        start: i32,
        len: i32,
        min: [i16; STORE_DIM],
        max: [i16; STORE_DIM],
    }

    #[derive(Clone, Copy)]
    struct PartitionRoot {
        key: u32,
        root: i32,
    }

    #[derive(Clone, Copy, Eq, PartialEq)]
    struct SplitRange {
        start: usize,
        end: usize,
    }

    impl SplitRange {
        #[inline]
        fn len(self) -> usize {
            self.end - self.start
        }
    }

    impl Ord for SplitRange {
        fn cmp(&self, other: &Self) -> Ordering {
            self.len().cmp(&other.len())
        }
    }

    impl PartialOrd for SplitRange {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Builder {
        pub fn new() -> Self {
            Builder {
                vectors: Vec::new(),
                labels: Vec::new(),
            }
        }

        pub fn add(&mut self, v: [i16; STORE_DIM], label: u8) {
            self.vectors.push(v);
            self.labels.push(label);
        }

        pub fn write_to(&self, path: &Path) -> std::io::Result<()> {
            assert!(!self.vectors.is_empty());
            assert_eq!(self.vectors.len(), self.labels.len());
            if std::env::var("RINHA_BUILD_IVF").ok().as_deref() == Some("1") {
                return write_ivf_to(&self.vectors, &self.labels, path);
            }
            write_kd_pair_to(&self.vectors, &self.labels, path)
        }
    }

    fn build_mcc_table() -> [i16; MCC_TABLE_SIZE] {
        let mut table = [quantize(DEFAULT_MCC_RISK); MCC_TABLE_SIZE];
        for (mcc, risk) in MCC_RISK {
            let mut code = 0u32;
            for &b in mcc.iter() {
                code = code * 10 + (b - b'0') as u32;
            }
            table[(code as usize) % MCC_TABLE_SIZE] = quantize(*risk);
        }
        table
    }

    fn write_kd_pair_to(
        vectors: &[[i16; STORE_DIM]],
        labels: &[u8],
        path: &Path,
    ) -> std::io::Result<()> {
        let leaf_size = kd_leaf_size();
        let mut buckets: Vec<Vec<usize>> = vec![Vec::new(); 256];
        for (i, v) in vectors.iter().enumerate() {
            buckets[partition_key(v) as usize].push(i);
        }

        let mut nodes: Vec<BuildNode> = Vec::new();
        let mut blocks: Vec<([i16; STORE_DIM], u8)> = Vec::with_capacity(vectors.len() + LANES);
        let mut roots: Vec<PartitionRoot> = Vec::new();

        for (key, indices) in buckets.iter().enumerate() {
            if indices.is_empty() {
                continue;
            }
            let root = build_tree(vectors, labels, indices, leaf_size, &mut blocks, &mut nodes);
            roots.push(PartitionRoot {
                key: key as u32,
                root: root as i32,
            });
        }

        assert_eq!(blocks.len() % LANES, 0);
        let block_count = blocks.len() / LANES;
        let partitions_off = HEADER_SIZE;
        let nodes_off = partitions_off + roots.len() * PART_SIZE;
        let vectors_off = nodes_off + nodes.len() * NODE_SIZE;
        let labels_off = vectors_off + block_count * BLOCK_BYTES;
        let mcc_table_off = labels_off + block_count * LANES;
        let total = mcc_table_off + MCC_TABLE_SIZE * 2;
        let mut out = vec![0u8; total];

        let header = Header {
            magic: MAGIC,
            version: KD_PAIR_VERSION,
            scale: SCALE as u32,
            dim: DIM as u32,
            store_dim: STORE_DIM as u32,
            n_points: vectors.len() as u32,
            part_count: roots.len() as u32,
            node_count: nodes.len() as u32,
            block_count: block_count as u32,
            mcc_table_offset: mcc_table_off as u32,
            _pad: [0; 20],
        };
        let header_bytes = unsafe {
            std::slice::from_raw_parts(&header as *const Header as *const u8, HEADER_SIZE)
        };
        out[..HEADER_SIZE].copy_from_slice(header_bytes);

        for (i, r) in roots.iter().enumerate() {
            let off = partitions_off + i * PART_SIZE;
            let n = &nodes[r.root as usize];
            out[off..off + 4].copy_from_slice(&r.key.to_le_bytes());
            out[off + 4..off + 8].copy_from_slice(&r.root.to_le_bytes());
            out[off + 8..off + 12].copy_from_slice(&n.len.to_le_bytes());
            write_qv(&mut out[off + 12..off + 44], &n.min);
            write_qv(&mut out[off + 44..off + 76], &n.max);
        }

        for (i, n) in nodes.iter().enumerate() {
            let off = nodes_off + i * NODE_SIZE;
            out[off..off + 4].copy_from_slice(&n.left.to_le_bytes());
            out[off + 4..off + 8].copy_from_slice(&n.right.to_le_bytes());
            let start_block = if n.left < 0 {
                n.start / LANES as i32
            } else {
                n.start
            };
            out[off + 8..off + 12].copy_from_slice(&start_block.to_le_bytes());
            out[off + 12..off + 16].copy_from_slice(&n.len.to_le_bytes());
            write_qv(&mut out[off + 16..off + 48], &n.min);
            write_qv(&mut out[off + 48..off + 80], &n.max);
        }

        for b in 0..block_count {
            let block_off = vectors_off + b * BLOCK_BYTES;
            for d in 0..DIM {
                for lane in 0..LANES {
                    let slot = b * LANES + lane;
                    let dst = block_off + ivf_pair_offset(d, lane) * 2;
                    out[dst..dst + 2].copy_from_slice(&blocks[slot].0[d].to_le_bytes());
                }
            }
        }

        for b in 0..block_count {
            let base = labels_off + b * LANES;
            for lane in 0..LANES {
                out[base + lane] = blocks[b * LANES + lane].1;
            }
        }

        let mcc_table = build_mcc_table();
        for (i, &v) in mcc_table.iter().enumerate() {
            let off = mcc_table_off + i * 2;
            out[off..off + 2].copy_from_slice(&v.to_le_bytes());
        }

        let mut f = std::io::BufWriter::with_capacity(8 << 20, std::fs::File::create(path)?);
        f.write_all(&out)?;
        f.flush()?;
        Ok(())
    }

    fn kd_leaf_size() -> usize {
        std::env::var("KD_LEAF_SIZE")
            .ok()
            .and_then(|v| v.parse().ok())
            .filter(|&v| (LANES..=1024).contains(&v))
            .unwrap_or(DEFAULT_LEAF_SIZE)
    }

    fn write_ivf_to(
        vectors: &[[i16; STORE_DIM]],
        labels: &[u8],
        path: &Path,
    ) -> std::io::Result<()> {
        let mut indices: Vec<usize> = (0..vectors.len()).collect();
        let ranges = split_ivf_ranges(vectors, &mut indices, IVF_CLUSTER_COUNT);
        let cluster_count = ranges.len();

        let mut centroids = vec![[0i16; STORE_DIM]; cluster_count];
        let mut mins = vec![[0i16; STORE_DIM]; cluster_count];
        let mut maxs = vec![[0i16; STORE_DIM]; cluster_count];
        let mut counts = vec![0u32; cluster_count];
        let mut block_offsets = vec![0u32; cluster_count + 1];

        for (c, range) in ranges.iter().enumerate() {
            let slice = &indices[range.start..range.end];
            counts[c] = slice.len() as u32;
            block_offsets[c + 1] = block_offsets[c] + slice.len().div_ceil(LANES) as u32;
            let (lo, hi) = bounds(vectors, slice);
            mins[c] = lo;
            maxs[c] = hi;

            let mut sums = [0i64; STORE_DIM];
            for &idx in slice {
                for d in 0..DIM {
                    sums[d] += vectors[idx][d] as i64;
                }
            }
            for d in 0..DIM {
                centroids[c][d] = (sums[d] / slice.len() as i64) as i16;
            }
        }

        for (c, range) in ranges.iter().enumerate() {
            let centroid = centroids[c];
            indices[range.start..range.end]
                .sort_unstable_by_key(|&i| distance_qv_scalar(&vectors[i], &centroid));
        }

        let block_count = block_offsets[cluster_count] as usize;
        let layout = ivf_offsets(cluster_count, block_count);
        let mut out = vec![0u8; layout.end];

        let header = Header {
            magic: MAGIC,
            version: IVF_VERSION,
            scale: SCALE as u32,
            dim: DIM as u32,
            store_dim: STORE_DIM as u32,
            n_points: vectors.len() as u32,
            part_count: cluster_count as u32,
            node_count: 0,
            block_count: block_count as u32,
            mcc_table_offset: layout.mcc_table as u32,
            _pad: [0; 20],
        };
        let header_bytes = unsafe {
            std::slice::from_raw_parts(&header as *const Header as *const u8, HEADER_SIZE)
        };
        out[..HEADER_SIZE].copy_from_slice(header_bytes);

        for c in 0..cluster_count {
            write_qv(
                &mut out[layout.centroids + c * STORE_DIM * 2
                    ..layout.centroids + (c + 1) * STORE_DIM * 2],
                &centroids[c],
            );
            write_qv(
                &mut out[layout.bbox_min + c * STORE_DIM * 2
                    ..layout.bbox_min + (c + 1) * STORE_DIM * 2],
                &mins[c],
            );
            write_qv(
                &mut out[layout.bbox_max + c * STORE_DIM * 2
                    ..layout.bbox_max + (c + 1) * STORE_DIM * 2],
                &maxs[c],
            );
            out[layout.counts + c * 4..layout.counts + c * 4 + 4]
                .copy_from_slice(&counts[c].to_le_bytes());
            out[layout.offsets + c * 4..layout.offsets + c * 4 + 4]
                .copy_from_slice(&block_offsets[c].to_le_bytes());
        }
        out[layout.offsets + cluster_count * 4..layout.offsets + cluster_count * 4 + 4]
            .copy_from_slice(&block_offsets[cluster_count].to_le_bytes());

        for (c, range) in ranges.iter().enumerate() {
            let start_block = block_offsets[c] as usize;
            for (pos, &orig_idx) in indices[range.start..range.end].iter().enumerate() {
                let block = start_block + pos / LANES;
                let lane = pos % LANES;
                out[layout.labels + block * LANES + lane] = labels[orig_idx];
                let block_off = layout.vectors + block * BLOCK_BYTES;
                for d in 0..DIM {
                    let dst = block_off + ivf_pair_offset(d, lane) * 2;
                    out[dst..dst + 2].copy_from_slice(&vectors[orig_idx][d].to_le_bytes());
                }
            }
        }

        let mcc_table = build_mcc_table();
        for (i, &v) in mcc_table.iter().enumerate() {
            let off = layout.mcc_table + i * 2;
            out[off..off + 2].copy_from_slice(&v.to_le_bytes());
        }

        let mut f = std::io::BufWriter::with_capacity(8 << 20, std::fs::File::create(path)?);
        f.write_all(&out)?;
        f.flush()?;
        Ok(())
    }

    fn split_ivf_ranges(
        vectors: &[[i16; STORE_DIM]],
        indices: &mut [usize],
        target_clusters: usize,
    ) -> Vec<SplitRange> {
        let mut heap = BinaryHeap::new();
        heap.push(SplitRange {
            start: 0,
            end: indices.len(),
        });

        while heap.len() < target_clusters {
            let range = match heap.pop() {
                Some(r) if r.len() > LANES => r,
                Some(r) => {
                    heap.push(r);
                    break;
                }
                None => break,
            };
            let slice = &indices[range.start..range.end];
            let (lo, hi) = bounds(vectors, slice);
            let split_dim = widest_dim(&lo, &hi);
            let mid = range.start + range.len() / 2;
            indices[range.start..range.end]
                .select_nth_unstable_by_key(mid - range.start, |&i| vectors[i][split_dim]);
            heap.push(SplitRange {
                start: range.start,
                end: mid,
            });
            heap.push(SplitRange {
                start: mid,
                end: range.end,
            });
        }

        let mut ranges = heap.into_vec();
        ranges.sort_unstable_by_key(|r| r.start);
        ranges
    }

    fn write_qv(dst: &mut [u8], v: &[i16; STORE_DIM]) {
        debug_assert_eq!(dst.len(), STORE_DIM * 2);
        for i in 0..STORE_DIM {
            dst[i * 2..i * 2 + 2].copy_from_slice(&v[i].to_le_bytes());
        }
    }

    fn bounds(
        vectors: &[[i16; STORE_DIM]],
        indices: &[usize],
    ) -> ([i16; STORE_DIM], [i16; STORE_DIM]) {
        let mut lo = [i16::MAX; STORE_DIM];
        let mut hi = [i16::MIN; STORE_DIM];
        for &i in indices {
            let v = &vectors[i];
            for d in 0..STORE_DIM {
                if v[d] < lo[d] {
                    lo[d] = v[d];
                }
                if v[d] > hi[d] {
                    hi[d] = v[d];
                }
            }
        }
        (lo, hi)
    }

    fn widest_dim(lo: &[i16; STORE_DIM], hi: &[i16; STORE_DIM]) -> usize {
        let mut best = 0usize;
        let mut best_w = i32::MIN;
        for d in 0..DIM {
            let w = hi[d] as i32 - lo[d] as i32;
            if w > best_w {
                best_w = w;
                best = d;
            }
        }
        best
    }

    fn build_tree(
        vectors: &[[i16; STORE_DIM]],
        labels: &[u8],
        indices: &[usize],
        leaf_size: usize,
        blocks: &mut Vec<([i16; STORE_DIM], u8)>,
        nodes: &mut Vec<BuildNode>,
    ) -> usize {
        let (lo, hi) = bounds(vectors, indices);
        let node_idx = nodes.len();
        nodes.push(BuildNode {
            left: -1,
            right: -1,
            start: 0,
            len: indices.len() as i32,
            min: lo,
            max: hi,
        });

        if indices.len() <= leaf_size {
            let start_slot = blocks.len() as i32;
            for &i in indices {
                blocks.push((vectors[i], labels[i]));
            }
            while blocks.len() % LANES != 0 {
                blocks.push(([i16::MAX; STORE_DIM], LABEL_LEGIT));
            }
            let node = &mut nodes[node_idx];
            node.start = start_slot;
            node.len = indices.len() as i32;
            return node_idx;
        }

        let split_dim = widest_dim(&lo, &hi);
        let mut sorted = indices.to_vec();
        sorted.sort_unstable_by_key(|&i| vectors[i][split_dim]);
        let mid = sorted.len() / 2;
        let (left_idx, right_idx) = sorted.split_at(mid);

        let left = build_tree(vectors, labels, left_idx, leaf_size, blocks, nodes);
        let right = build_tree(vectors, labels, right_idx, leaf_size, blocks, nodes);

        let left_start = nodes[left].start;
        let total_len = nodes[left].len + nodes[right].len;
        let node = &mut nodes[node_idx];
        node.left = left as i32;
        node.right = right as i32;
        node.start = left_start;
        node.len = total_len;
        node_idx
    }
}
