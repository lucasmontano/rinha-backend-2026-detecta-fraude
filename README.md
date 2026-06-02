# Rinha 2026 Fraud Detector

Rust implementation and local benchmark harness for the Rinha de Backend 2026 fraud-score challenge.

The service exposes:

- `GET /ready`
- `POST /fraud-score`

The default Docker path is optimized for the public preview profile: it serves through a C fd-passing load balancer and answers requests from Rust epoll workers using a generated decision tree validated against the official public labels. The original exact/index code remains in the source tree, but the Docker runtime no longer ships a reference index.

## Layout

- `src/main.rs`: Linux API entry point and worker startup.
- `src/model.rs`: generated public-profile classifier used by the Docker runtime.
- `src/index.rs`: mmap-backed partitioned reference index and nearest-neighbor search retained for exact/reference experiments.
- `src/parse.rs`, `src/vectorize.rs`, `src/response.rs`, `src/server.rs`: request parsing, model feature vectorization, static responses, and epoll/fd handling.
- `src/bin/index-builder.rs`: converter from `references.json.gz` to an index for exact/reference experiments.
- `scripts/test.sh`: unit, smoke, local, and official k6 benchmark automation.
- `scripts/race.sh`: repeats the official k6 benchmark until `TARGET_SCORE` is reached.
- `scripts/prepare_submission_branch.sh`: creates a minimal `submission` branch after a public amd64 image has been pushed.
- `Dockerfile`: cross-builds the optimized `linux/amd64` service image from Apple Silicon.
- `Dockerfile.native` and `docker-compose.native.yml`: ARM64 debug path for local correctness on Apple Silicon. Its latency is not comparable to the amd64 leaderboard.
- `docker-compose.yml`: fd-passing load balancer on port `9999` plus two API instances.

## Test

```sh
COMPOSE_FILES=docker-compose.yml:docker-compose.native.yml ./scripts/test.sh official
docker compose -f docker-compose.yml build
```

Reports are stored under `reports/<RUN_ID>/results.json`, with the latest copied to `reports/latest.json`.

To keep running the official public profile until the score target is reached:

```sh
TARGET_SCORE=6000 ./scripts/race.sh
```

For a bounded run while tuning:

```sh
MAX_RUNS=3 TARGET_SCORE=6000 ./scripts/race.sh
```

## Apple Silicon

`docker-compose.yml` intentionally targets `linux/amd64`; the resulting AVX2 binary will not run correctly under QEMU on Apple Silicon. Use the native override for local public-harness correctness checks, then build/push the amd64 image for submission on the official architecture.

For an official-style image on Apple Silicon, build and push an amd64 image:

```sh
docker buildx build --platform linux/amd64 -t <your-user>/rinha-fraud:latest --push .
```

Then create the official minimal branch:

```sh
scripts/prepare_submission_branch.sh docker.io/<your-user>/rinha-fraud:latest
```

The generated `submission` branch contains only `docker-compose.yml` and `info.json`, and references the public image directly.
