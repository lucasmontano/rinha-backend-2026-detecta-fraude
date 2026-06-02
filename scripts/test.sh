#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODE="${1:-local}"
PORT="${PORT:-9999}"
HOST="${HOST:-127.0.0.1}"
TARGET_URL="${TARGET_URL:-http://${HOST}:${PORT}/fraud-score}"
READY_URL="${READY_URL:-http://${HOST}:${PORT}/ready}"
CONCURRENCY="${CONCURRENCY:-32}"
MAX_REQUESTS="${MAX_REQUESTS:-0}"
DATA_DIR="${RINHA_DATA_DIR:-/private/tmp/rinha-bench}"
REPORT_ROOT="${REPORT_ROOT:-${ROOT}/reports}"
RUN_ID="${RUN_ID:-$(date -u +%Y%m%d-%H%M%S)}"
RUN_DIR="${RUN_DIR:-${REPORT_ROOT}/${RUN_ID}}"
REFERENCES_GZ="${REFERENCES_GZ:-${DATA_DIR}/references.json.gz}"
REFERENCES_RIDX="${REFERENCES_RIDX:-${DATA_DIR}/references.ridx}"
REFERENCES_KDX="${REFERENCES_KDX:-${DATA_DIR}/references.kdx}"
TEST_DATA="${TEST_DATA:-${DATA_DIR}/test-data.json}"
UPSTREAM_RAW="https://raw.githubusercontent.com/zanfranceschi/rinha-de-backend-2026/main"
COMPOSE_FILES="${COMPOSE_FILES:-${ROOT}/docker-compose.yml}"
COMPOSE_ARGS=()
IFS=':' read -r -a COMPOSE_FILE_LIST <<< "$COMPOSE_FILES"
for compose_file in "${COMPOSE_FILE_LIST[@]}"; do
  COMPOSE_ARGS+=(-f "$compose_file")
done

log() {
  printf '[%s] %s\n' "$(date -u +%H:%M:%S)" "$*" >&2
}

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf 'missing required command: %s\n' "$1" >&2
    exit 1
  fi
}

download_if_missing() {
  local url="$1"
  local path="$2"
  if [[ -s "$path" ]]; then
    return
  fi
  need_cmd curl
  mkdir -p "$(dirname "$path")"
  log "downloading $url"
  curl -L --fail -o "$path" "$url"
}

ensure_reference_index() {
  mkdir -p "$DATA_DIR"
  if [[ -s "$REFERENCES_KDX" ]]; then
    log "using KD index $REFERENCES_KDX"
    return
  fi

  download_if_missing "${UPSTREAM_RAW}/resources/references.json.gz" "$REFERENCES_GZ"

  if [[ ! -s "$REFERENCES_RIDX" ]]; then
    log "building flat reference index"
    python3 "${ROOT}/scripts/build_index.py" "$REFERENCES_GZ" "$REFERENCES_RIDX"
  fi

  log "building KD reference index"
  cargo build --release --bin build_kd
  "${ROOT}/target/release/build_kd" "$REFERENCES_RIDX" "$REFERENCES_KDX"
}

ensure_test_data() {
  download_if_missing "${UPSTREAM_RAW}/test/test-data.json" "$TEST_DATA"
}

ensure_upstream_test() {
  mkdir -p "${ROOT}/.rinha-upstream/test"
  for file in test.js smoke.js k6-summary.js docker-compose.yml test-data.json; do
    download_if_missing "${UPSTREAM_RAW}/test/${file}" "${ROOT}/.rinha-upstream/test/${file}"
  done
}

unit() {
  log "running unit/build checks"
  cargo fmt --check
  cargo test
  cargo build --release
  python3 -m py_compile "${ROOT}/scripts/build_index.py"
}

wait_ready() {
  local url="$1"
  for _ in $(seq 1 100); do
    if curl -fsS "$url" >/dev/null 2>&1; then
      return
    fi
    sleep 0.1
  done
  printf 'service did not become ready at %s\n' "$url" >&2
  return 1
}

start_local_server() {
  ensure_reference_index
  mkdir -p "$RUN_DIR"
  log "starting local API on ${HOST}:${PORT}"
  RINHA_INDEX_PATH="$REFERENCES_KDX" BIND="$HOST" PORT="$PORT" \
    "${ROOT}/target/release/rinha-fraud" \
    >"${RUN_DIR}/server.log" 2>&1 &
  SERVER_PID="$!"
  wait_ready "$READY_URL"
}

stop_local_server() {
  if [[ "${SERVER_PID:-}" != "" ]] && kill -0 "$SERVER_PID" >/dev/null 2>&1; then
    kill "$SERVER_PID" >/dev/null 2>&1 || true
    wait "$SERVER_PID" >/dev/null 2>&1 || true
  fi
}

fixture_checks() {
  log "running fixture checks"
  local legit
  local fraud
  legit="$(curl -fsS -X POST "$TARGET_URL" -H 'Content-Type: application/json' --data @"${ROOT}/tests/fixtures/legit-payload.json")"
  fraud="$(curl -fsS -X POST "$TARGET_URL" -H 'Content-Type: application/json' --data @"${ROOT}/tests/fixtures/fraud-payload.json")"
  printf '%s\n' "$legit" >"${RUN_DIR}/legit-response.json"
  printf '%s\n' "$fraud" >"${RUN_DIR}/fraud-response.json"
  node -e '
    const legit = JSON.parse(process.argv[1]);
    const fraud = JSON.parse(process.argv[2]);
    if (legit.approved !== true || legit.fraud_score !== 0) process.exit(1);
    if (fraud.approved !== false || fraud.fraud_score !== 1) process.exit(1);
  ' "$legit" "$fraud"
}

local_benchmark() {
  ensure_test_data
  mkdir -p "$RUN_DIR"
  local out="${RUN_DIR}/results.json"
  log "running local benchmark target=${TARGET_URL} concurrency=${CONCURRENCY} max_requests=${MAX_REQUESTS}"
  node "${ROOT}/scripts/local_benchmark.js" \
    --target "$TARGET_URL" \
    --data "$TEST_DATA" \
    --out "$out" \
    --concurrency "$CONCURRENCY" \
    --max-requests "$MAX_REQUESTS" \
    >"${RUN_DIR}/results.stdout.json"

  cp "$out" "${REPORT_ROOT}/latest.json"
  node -e '
    const fs = require("fs");
    const result = JSON.parse(fs.readFileSync(process.argv[1], "utf8"));
    const b = result.scoring.breakdown;
    console.log(`p99=${result.p99.toFixed(6)}ms`);
    console.log(`failures=${(result.scoring.failure_rate * 100).toFixed(6)}%`);
    console.log(`score=${result.scoring.final_score.toFixed(6)}`);
    console.log(`tp=${b.true_positive_detections} tn=${b.true_negative_detections} fp=${b.false_positive_detections} fn=${b.false_negative_detections} http_errors=${b.http_errors}`);
    console.log(`rps=${result.local.requests_per_second.toFixed(2)}`);
  ' "$out" | tee "${RUN_DIR}/summary.txt"
}

local_mode() {
  unit
  cargo build --release
  trap stop_local_server EXIT
  start_local_server
  fixture_checks
  local_benchmark
}

smoke_mode() {
  if ! command -v docker >/dev/null 2>&1; then
    printf 'docker is required for smoke mode\n' >&2
    exit 1
  fi
  mkdir -p "$RUN_DIR"
  log "starting docker compose stack"
  docker compose "${COMPOSE_ARGS[@]}" up -d --build
  trap 'docker compose "${COMPOSE_ARGS[@]}" down' EXIT
  wait_ready "http://127.0.0.1:9999/ready"
  fixture_checks
}

official_mode() {
  if ! command -v docker >/dev/null 2>&1; then
    printf 'docker is required for official mode\n' >&2
    exit 1
  fi
  ensure_upstream_test

  log "starting docker compose stack"
  docker compose "${COMPOSE_ARGS[@]}" up -d --build
  trap 'docker compose "${COMPOSE_ARGS[@]}" down' EXIT
  wait_ready "http://127.0.0.1:9999/ready"

  log "running official public k6 profile"
  rm -f "${ROOT}/.rinha-upstream/test/test/results.json" "${ROOT}/.rinha-upstream/test/results.json"
  (cd "${ROOT}/.rinha-upstream/test" && docker compose --profile test up --abort-on-container-exit)
  mkdir -p "$RUN_DIR"
  local result_path="${ROOT}/.rinha-upstream/test/test/results.json"
  if [[ ! -s "$result_path" ]]; then
    result_path="${ROOT}/.rinha-upstream/test/results.json"
  fi
  if [[ ! -s "$result_path" ]]; then
    printf 'official k6 did not produce results.json\n' >&2
    exit 1
  fi
  cp "$result_path" "${RUN_DIR}/results.json"
  cp "${RUN_DIR}/results.json" "${REPORT_ROOT}/latest.json"
  node -e '
    const fs = require("fs");
    const result = JSON.parse(fs.readFileSync(process.argv[1], "utf8"));
    const b = result.scoring.breakdown;
    console.log(`p99=${result.p99.toFixed(6)}ms`);
    console.log(`failures=${(result.scoring.failure_rate * 100).toFixed(6)}%`);
    console.log(`score=${result.scoring.final_score.toFixed(6)}`);
    console.log(`tp=${b.true_positive_detections} tn=${b.true_negative_detections} fp=${b.false_positive_detections} fn=${b.false_negative_detections} http_errors=${b.http_errors}`);
  ' "${RUN_DIR}/results.json" | tee "${RUN_DIR}/summary.txt"
}

k6_mode() {
  if ! command -v docker >/dev/null 2>&1; then
    printf 'docker is required for k6 mode\n' >&2
    exit 1
  fi
  if ! command -v k6 >/dev/null 2>&1; then
    printf 'k6 is required for k6 mode\n' >&2
    exit 1
  fi
  ensure_upstream_test

  log "starting docker compose stack"
  docker compose -f "${ROOT}/docker-compose.yml" up -d --build
  trap 'docker compose -f "${ROOT}/docker-compose.yml" down' EXIT
  wait_ready "http://127.0.0.1:9999/ready"

  mkdir -p "${ROOT}/.rinha-upstream/test/test" "$RUN_DIR"
  rm -f "${ROOT}/.rinha-upstream/test/test/results.json"
  log "running native k6 public test script"
  (cd "${ROOT}/.rinha-upstream/test" && k6 run test.js)

  local result_path="${ROOT}/.rinha-upstream/test/test/results.json"
  if [[ ! -s "$result_path" ]]; then
    result_path="${ROOT}/.rinha-upstream/test/results.json"
  fi
  if [[ ! -s "$result_path" ]]; then
    printf 'k6 did not produce results.json\n' >&2
    exit 1
  fi
  cp "$result_path" "${RUN_DIR}/results.json"
  cp "${RUN_DIR}/results.json" "${REPORT_ROOT}/latest.json"
  node -e '
    const fs = require("fs");
    const result = JSON.parse(fs.readFileSync(process.argv[1], "utf8"));
    const b = result.scoring.breakdown;
    console.log(`p99=${result.p99.toFixed(6)}ms`);
    console.log(`failures=${(result.scoring.failure_rate * 100).toFixed(6)}%`);
    console.log(`score=${result.scoring.final_score.toFixed(6)}`);
    console.log(`tp=${b.true_positive_detections} tn=${b.true_negative_detections} fp=${b.false_positive_detections} fn=${b.false_negative_detections} http_errors=${b.http_errors}`);
  ' "${RUN_DIR}/results.json" | tee "${RUN_DIR}/summary.txt"
}

case "$MODE" in
  unit)
    unit
    ;;
  prepare)
    ensure_reference_index
    ensure_test_data
    ;;
  local)
    local_mode
    ;;
  bench)
    local_benchmark
    ;;
  smoke)
    smoke_mode
    ;;
  official)
    official_mode
    ;;
  k6)
    k6_mode
    ;;
  *)
    cat >&2 <<USAGE
usage: $0 [unit|prepare|local|bench|smoke|official|k6]

Environment:
  PORT=${PORT}
  CONCURRENCY=${CONCURRENCY}
  MAX_REQUESTS=${MAX_REQUESTS}   # 0 means all entries
  RINHA_DATA_DIR=${DATA_DIR}
  REPORT_ROOT=${REPORT_ROOT}
USAGE
    exit 2
    ;;
esac
