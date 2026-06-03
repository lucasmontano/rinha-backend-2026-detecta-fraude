#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODE="${1:-unit}"
PORT="${PORT:-9999}"
HOST="${HOST:-127.0.0.1}"
TARGET_URL="${TARGET_URL:-http://${HOST}:${PORT}/fraud-score}"
READY_URL="${READY_URL:-http://${HOST}:${PORT}/ready}"
COMPOSE_FILES="${COMPOSE_FILES:-${ROOT}/docker-compose.yml}"
COMPOSE_ARGS=()
IFS=':' read -r -a COMPOSE_FILE_LIST <<< "$COMPOSE_FILES"
for compose_file in "${COMPOSE_FILE_LIST[@]}"; do
  COMPOSE_ARGS+=(-f "$compose_file")
done

log() {
  printf '[%s] %s\n' "$(date -u +%H:%M:%S)" "$*" >&2
}

unit() {
  log "running unit/build checks"
  cargo fmt --check
  cargo check --features builder --bin index-builder --bin rinha-fraud --tests
  cargo test --features builder
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

smoke_payload() {
  cat <<'JSON'
{
  "id": "smoke-synthetic-0001",
  "transaction": {
    "amount": 123.45,
    "installments": 2,
    "requested_at": "2026-03-12T10:15:00Z"
  },
  "customer": {
    "avg_amount": 100.00,
    "tx_count_24h": 1,
    "known_merchants": ["SMOKE-MERCHANT-001"]
  },
  "merchant": {
    "id": "SMOKE-MERCHANT-001",
    "mcc": "5411",
    "avg_amount": 120.00
  },
  "terminal": {
    "is_online": true,
    "card_present": false,
    "km_from_home": 3.5
  },
  "last_transaction": null
}
JSON
}

smoke_request() {
  log "running synthetic smoke request"
  local response
  response="$(smoke_payload | curl -fsS -X POST "$TARGET_URL" -H 'Content-Type: application/json' --data-binary @-)"
  node -e '
    const response = JSON.parse(process.argv[1]);
    if (typeof response.approved !== "boolean") process.exit(1);
    if (typeof response.fraud_score !== "number") process.exit(1);
    if (response.fraud_score < 0 || response.fraud_score > 1) process.exit(1);
  ' "$response"
  printf '%s\n' "$response"
}

smoke_mode() {
  if ! command -v docker >/dev/null 2>&1; then
    printf 'docker is required for smoke mode\n' >&2
    exit 1
  fi
  log "starting docker compose stack"
  docker compose "${COMPOSE_ARGS[@]}" up -d --build
  trap 'docker compose "${COMPOSE_ARGS[@]}" down' EXIT
  wait_ready "$READY_URL"
  smoke_request
}

case "$MODE" in
  unit)
    unit
    ;;
  smoke)
    smoke_mode
    ;;
  *)
    cat >&2 <<USAGE
usage: $0 [unit|smoke]

Environment:
  PORT=${PORT}
  COMPOSE_FILES=${COMPOSE_FILES}
USAGE
    exit 2
    ;;
esac
