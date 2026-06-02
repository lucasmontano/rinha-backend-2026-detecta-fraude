#[cfg(feature = "builder")]
fn main() {
    use detecta_fraude::index::build::Builder;
    use detecta_fraude::index::{LABEL_FRAUD, LABEL_LEGIT};
    use detecta_fraude::{quantize, DIM, STORE_DIM};
    use flate2::read::GzDecoder;
    use std::env;
    use std::fs::File;
    use std::io::{BufReader, Read};
    use std::time::Instant;

    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("uso: {} <references.json.gz> <out.bin>", args[0]);
        std::process::exit(2);
    }
    let input = &args[1];
    let output = &args[2];

    let t0 = Instant::now();
    eprintln!("[builder] descomprimindo {}", input);
    let file = File::open(input).expect("abrir input");
    let mut decoder = GzDecoder::new(BufReader::with_capacity(1 << 20, file));
    let mut data = Vec::with_capacity(320 * 1024 * 1024);
    decoder.read_to_end(&mut data).expect("descomprimir");
    eprintln!(
        "[builder] descomprimido em {:.1}s ({:.1} MB)",
        t0.elapsed().as_secs_f32(),
        data.len() as f32 / 1e6
    );

    let t1 = Instant::now();
    let mut b = Builder::new();
    let mut count: usize = 0;
    let mut fraud: usize = 0;
    parse_references(&data, |vec, label| {
        let mut q = [0i16; STORE_DIM];
        for i in 0..DIM {
            q[i] = quantize(vec[i]);
        }
        let l = if label == b"fraud" {
            LABEL_FRAUD
        } else {
            LABEL_LEGIT
        };
        if l == LABEL_FRAUD {
            fraud += 1;
        }
        b.add(q, l);
        count += 1;
    });
    eprintln!(
        "[builder] parse: {} pontos ({} fraud) em {:.1}s",
        count,
        fraud,
        t1.elapsed().as_secs_f32()
    );

    let t2 = Instant::now();
    b.write_to(std::path::Path::new(output))
        .expect("escrever output");
    eprintln!("[builder] escrita: {:.1}s", t2.elapsed().as_secs_f32());
    eprintln!("[builder] total: {:.1}s", t0.elapsed().as_secs_f32());
}

#[cfg(feature = "builder")]
fn parse_references(buf: &[u8], mut cb: impl FnMut(&[f64; detecta_fraude::DIM], &[u8])) {
    use detecta_fraude::DIM;
    let mut i = 0;
    while i < buf.len() {
        match buf[i] {
            b'{' => break,
            _ => i += 1,
        }
    }
    while i < buf.len() {
        while i < buf.len() && buf[i] != b'{' {
            i += 1;
        }
        if i >= buf.len() {
            break;
        }
        let vec_pos = find(buf, i, b"\"vector\"").expect("vector key");
        let bracket = find_byte(buf, vec_pos, b'[').expect("vector [");
        let (vec, after) = parse_float_array(buf, bracket + 1);
        if vec.len() != DIM {
            panic!("vetor com {} dim", vec.len());
        }
        let mut arr = [0f64; DIM];
        arr.copy_from_slice(&vec);
        let lbl_pos = find(buf, after, b"\"label\"").expect("label key");
        let q1 = find_byte(buf, lbl_pos + 7, b'"').expect("label quote");
        let q2 = find_byte(buf, q1 + 1, b'"').expect("label quote end");
        let label = &buf[q1 + 1..q2];
        cb(&arr, label);
        let mut j = q2 + 1;
        let mut depth = 1;
        while j < buf.len() && depth > 0 {
            match buf[j] {
                b'{' | b'[' => depth += 1,
                b'}' | b']' => depth -= 1,
                _ => {}
            }
            j += 1;
        }
        i = j;
    }
}

#[cfg(feature = "builder")]
fn parse_float_array(buf: &[u8], mut i: usize) -> (Vec<f64>, usize) {
    let mut out: Vec<f64> = Vec::with_capacity(detecta_fraude::DIM);
    loop {
        while i < buf.len() && (buf[i].is_ascii_whitespace() || buf[i] == b',') {
            i += 1;
        }
        if i < buf.len() && buf[i] == b']' {
            return (out, i + 1);
        }
        let start = i;
        while i < buf.len() {
            let c = buf[i];
            if (b'0'..=b'9').contains(&c)
                || c == b'.'
                || c == b'-'
                || c == b'+'
                || c == b'e'
                || c == b'E'
            {
                i += 1;
            } else {
                break;
            }
        }
        let s = std::str::from_utf8(&buf[start..i]).expect("utf8");
        let v: f64 = s.parse().expect("float");
        out.push(v);
    }
}

#[cfg(feature = "builder")]
fn find(buf: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || buf.len() < from + needle.len() {
        return None;
    }
    let end = buf.len() - needle.len();
    let mut i = from;
    while i <= end {
        if buf[i..i + needle.len()] == *needle {
            return Some(i);
        }
        i += 1;
    }
    None
}

#[cfg(feature = "builder")]
fn find_byte(buf: &[u8], from: usize, byte: u8) -> Option<usize> {
    buf[from..]
        .iter()
        .position(|&b| b == byte)
        .map(|p| p + from)
}

#[cfg(not(feature = "builder"))]
fn main() {
    eprintln!("index-builder requires feature 'builder' (build with --features builder)");
    std::process::exit(1);
}
