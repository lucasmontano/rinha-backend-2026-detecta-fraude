# syntax=docker/dockerfile:1
FROM --platform=$BUILDPLATFORM rust:1.95-slim-bookworm AS build
WORKDIR /src
RUN apt-get update && apt-get install -y --no-install-recommends \
        gcc libc6-dev gcc-x86-64-linux-gnu libc6-dev-amd64-cross \
    && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-gnu
COPY Cargo.toml Cargo.lock ./
COPY src ./src
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc
RUN RUSTFLAGS="-C target-cpu=haswell -C target-feature=+avx2,+fma,+bmi2" \
        cargo build --release --target x86_64-unknown-linux-gnu --bin rinha-fraud \
    && x86_64-linux-gnu-gcc -O3 -march=haswell -static -flto -fno-stack-protector -DNDEBUG -s \
        -o /src/target/release/lb-c /src/src/bin/lb_c.c

FROM --platform=linux/amd64 gcr.io/distroless/cc-debian12:latest
WORKDIR /app
COPY --from=build /src/target/x86_64-unknown-linux-gnu/release/rinha-fraud /usr/local/bin/rinha-fraud
COPY --from=build /src/target/release/lb-c /usr/local/bin/rinha-lb-c
EXPOSE 8080
CMD ["/usr/local/bin/rinha-fraud"]
