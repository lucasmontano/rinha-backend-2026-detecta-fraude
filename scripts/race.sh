#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_ROOT="${REPORT_ROOT:-${ROOT}/reports}"
TARGET_SCORE="${TARGET_SCORE:-6000}"
MAX_RUNS="${MAX_RUNS:-0}"
SLEEP_SECONDS="${SLEEP_SECONDS:-5}"
HISTORY="${REPORT_ROOT}/race-history.csv"

log() {
  printf '[%s] %s\n' "$(date -u +%H:%M:%S)" "$*" >&2
}

mkdir -p "$REPORT_ROOT"
if [[ ! -s "$HISTORY" ]]; then
  printf 'run_id,score,p99_ms,failures_percentage\n' >"$HISTORY"
fi

run=1
while true; do
  if [[ "$MAX_RUNS" != "0" && "$run" -gt "$MAX_RUNS" ]]; then
    log "target score ${TARGET_SCORE} not reached after ${MAX_RUNS} runs"
    exit 1
  fi

  run_id="$(date -u +%Y%m%d-%H%M%S)"
  log "starting run ${run} with RUN_ID=${run_id} target_score=${TARGET_SCORE}"
  RUN_ID="$run_id" "$ROOT/scripts/test.sh" k6

  result="${REPORT_ROOT}/${run_id}/results.json"
  if [[ ! -s "$result" ]]; then
    log "missing result file for RUN_ID=${run_id}"
    exit 1
  fi

  metrics="$(
    node -e '
      const fs = require("fs");
      const result = JSON.parse(fs.readFileSync(process.argv[1], "utf8"));
      const score = result.scoring.final_score;
      const p99 = result.p99;
      const failures = result.scoring.failure_rate * 100;
      console.log([score, p99, failures].join(" "));
    ' "$result"
  )"
  read -r score p99 failures <<<"$metrics"
  printf '%s,%s,%s,%s\n' "$run_id" "$score" "$p99" "$failures" >>"$HISTORY"

  log "run ${run} score=${score} p99=${p99}ms failures=${failures}%"
  if node -e 'process.exit(Number(process.argv[1]) >= Number(process.argv[2]) ? 0 : 1)' "$score" "$TARGET_SCORE"; then
    log "target score reached"
    exit 0
  fi

  run=$((run + 1))
  sleep "$SLEEP_SECONDS"
done
