#!/usr/bin/env python3
import argparse
import gzip
import re
import struct
from pathlib import Path

MAGIC = b"RINHA26\0"
SCALE = 10000.0
DIMS = 14
RECORD_RE = re.compile(
    rb'\{\s*"vector"\s*:\s*\[([^\]]+)\]\s*,\s*"label"\s*:\s*"(fraud|legit)"\s*\}',
    re.S,
)


def quantize(value: float) -> int:
    scaled = round(value * SCALE)
    if scaled < -32768:
        return -32768
    if scaled > 32767:
        return 32767
    return scaled


def open_input(path: Path):
    if path.suffix == ".gz":
        return gzip.open(path, "rb")
    return path.open("rb")


def iter_records(path: Path):
    buffer = b""
    with open_input(path) as source:
        while True:
            chunk = source.read(1024 * 1024)
            if not chunk:
                break
            buffer += chunk
            while True:
                match = RECORD_RE.search(buffer)
                if match is None:
                    if len(buffer) > 16384:
                        buffer = buffer[-16384:]
                    break
                yield match.group(1), match.group(2)
                buffer = buffer[match.end() :]

    while True:
        match = RECORD_RE.search(buffer)
        if match is None:
            break
        yield match.group(1), match.group(2)
        buffer = buffer[match.end() :]


def build_index(input_path: Path, output_path: Path):
    output_path.parent.mkdir(parents=True, exist_ok=True)
    count = 0
    with output_path.open("wb") as output:
        output.write(MAGIC)
        output.write(struct.pack("<I", 0))

        for vector_bytes, label_bytes in iter_records(input_path):
            values = [float(part.strip()) for part in vector_bytes.split(b",")]
            if len(values) != DIMS:
                raise ValueError(f"expected {DIMS} dimensions, got {len(values)}")
            output.write(struct.pack("<" + "h" * DIMS, *(quantize(value) for value in values)))
            output.write(b"\x01" if label_bytes == b"fraud" else b"\x00")
            count += 1

        output.seek(len(MAGIC))
        output.write(struct.pack("<I", count))

    print(f"wrote {count} records to {output_path}")


def main():
    parser = argparse.ArgumentParser(description="Build compact Rinha reference index")
    parser.add_argument("input", type=Path, help="references.json or references.json.gz")
    parser.add_argument("output", type=Path, help="output .ridx file")
    args = parser.parse_args()
    build_index(args.input, args.output)


if __name__ == "__main__":
    main()
