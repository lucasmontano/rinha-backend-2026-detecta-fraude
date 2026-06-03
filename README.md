# Rinha 2026 Fraud Detector

Rust implementation for the Rinha de Backend 2026 fraud-score challenge.

The service exposes:

- `GET /ready`
- `POST /fraud-score`

The Docker runtime follows the documented Rinha detection flow: it vectorizes each request into the 14 normalized dimensions, searches the 5 nearest vectors in the official reference dataset, returns `fraud_score = frauds / 5`, and approves when the score is below `0.6`.

The image build downloads `resources/references.json.gz` from the official Rinha repository, preprocesses it into a compact mmap-backed KD index, and ships that index at `/index/index.bin`.

## Compliance

The submitted runtime does not use preview, final, sample, or test payloads as fraud references, lookup keys, training data, model-generation data, or decision tables.

Only the official `resources/references.json.gz` file is used to build the shipped reference index. Incoming transaction IDs are ignored by the parser, and the runtime decision path is request payload -> 14-dimensional vector -> nearest-neighbor search over the official reference index -> `fraud_count / 5`.

Local scripts intentionally avoid downloading or consuming `test/test-data.json`, preview labels, expected results, or example payload fixtures.

## Layout

- `src/main.rs`: Linux API entry point and worker startup.
- `src/index.rs`: mmap-backed partitioned reference index and nearest-neighbor search.
- `src/parse.rs`, `src/vectorize.rs`, `src/response.rs`, `src/server.rs`: request parsing, 14-dimensional vectorization, static responses, and epoll/fd handling.
- `src/bin/index-builder.rs`: converter from the official `references.json.gz` to the runtime index.
- `scripts/test.sh`: unit checks and synthetic smoke automation.
- `scripts/prepare_submission_branch.sh`: creates a minimal `submission` branch after a public amd64 image has been pushed.
- `Dockerfile`: cross-builds the optimized `linux/amd64` service image from Apple Silicon.
- `Dockerfile.native` and `docker-compose.native.yml`: ARM64 debug path for local correctness on Apple Silicon. Its latency is not comparable to the amd64 leaderboard.
- `docker-compose.yml`: fd-passing load balancer on port `9999` plus two API instances.

## Test

```sh
./scripts/test.sh unit
COMPOSE_FILES=docker-compose.yml:docker-compose.native.yml ./scripts/test.sh smoke
docker compose -f docker-compose.yml build
```

Docker builds need network access to download the official reference dataset during the build stage.

## Apple Silicon

`docker-compose.yml` intentionally targets `linux/amd64`; the resulting AVX2 binary will not run correctly under QEMU on Apple Silicon. Use the native override for local smoke checks, then build/push the amd64 image for submission on the official architecture.

For an official-style image on Apple Silicon, build and push an amd64 image:

```sh
docker buildx build --platform linux/amd64 -t <your-user>/rinha-fraud:<tag> --push .
docker buildx imagetools inspect <your-user>/rinha-fraud:<tag>
```

Then create the official minimal branch using the immutable linux/amd64 manifest digest:

```sh
scripts/prepare_submission_branch.sh docker.io/<your-user>/rinha-fraud@sha256:<linux-amd64-manifest-digest>
```

The prepared `submission` branch contains only `docker-compose.yml` and `info.json`, and references the public image by digest to avoid mutable tag cache ambiguity.
