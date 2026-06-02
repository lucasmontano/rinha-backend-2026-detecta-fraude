use std::env;
use std::io::{Read, Write};

const DIMS: usize = 14;
const MAGIC_FLAT: &[u8; 8] = b"RINHA26\0";
const MAGIC_KD: &[u8; 8] = b"RINHAK1\0";

#[derive(Copy, Clone)]
struct Record {
    vector: [i16; DIMS],
    label: u8,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: build_kd <input.ridx> <output.kdx>");
        std::process::exit(2);
    }

    let mut records = match load_flat(&args[1]) {
        Ok(records) => records,
        Err(err) => {
            eprintln!("failed to read {}: {err}", args[1]);
            std::process::exit(1);
        }
    };

    build_kd_order(&mut records, 0);

    if let Err(err) = write_kd(&args[2], &records) {
        eprintln!("failed to write {}: {err}", args[2]);
        std::process::exit(1);
    }

    eprintln!("wrote {} KD-ordered records to {}", records.len(), args[2]);
}

fn load_flat(path: &str) -> std::io::Result<Vec<Record>> {
    let mut file = std::fs::File::open(path)?;
    let mut magic = [0u8; 8];
    file.read_exact(&mut magic)?;
    if &magic != MAGIC_FLAT {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "input is not a flat Rinha index",
        ));
    }

    let mut count_bytes = [0u8; 4];
    file.read_exact(&mut count_bytes)?;
    let count = u32::from_le_bytes(count_bytes) as usize;

    let mut records = Vec::with_capacity(count);
    let mut raw = [0u8; DIMS * 2 + 1];
    for _ in 0..count {
        file.read_exact(&mut raw)?;
        let mut vector = [0i16; DIMS];
        for dim in 0..DIMS {
            let offset = dim * 2;
            vector[dim] = i16::from_le_bytes([raw[offset], raw[offset + 1]]);
        }
        records.push(Record {
            vector,
            label: raw[DIMS * 2],
        });
    }
    Ok(records)
}

fn write_kd(path: &str, records: &[Record]) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    file.write_all(MAGIC_KD)?;
    file.write_all(&(records.len() as u32).to_le_bytes())?;
    for record in records {
        for value in record.vector {
            file.write_all(&value.to_le_bytes())?;
        }
        file.write_all(&[record.label])?;
    }
    Ok(())
}

fn build_kd_order(records: &mut [Record], depth: usize) {
    if records.len() <= 1 {
        return;
    }

    let dim = depth % DIMS;
    let mid = records.len() / 2;
    records.select_nth_unstable_by(mid, |left, right| left.vector[dim].cmp(&right.vector[dim]));

    let (left, rest) = records.split_at_mut(mid);
    let (_, right) = rest.split_at_mut(1);
    build_kd_order(left, depth + 1);
    build_kd_order(right, depth + 1);
}
