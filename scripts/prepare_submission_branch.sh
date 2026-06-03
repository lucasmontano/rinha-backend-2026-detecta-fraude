#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE="${1:-${SUBMISSION_IMAGE:-}}"
BRANCH="${SUBMISSION_BRANCH:-submission}"

if [[ -z "$IMAGE" ]]; then
  cat >&2 <<'EOF'
usage: scripts/prepare_submission_branch.sh <public-linux-amd64-image>

Example:
  scripts/prepare_submission_branch.sh ghcr.io/lucasmontano/rinha-backend-2026-detecta-fraude@sha256:<linux-amd64-manifest-digest>

The image must already be public and compatible with linux/amd64. Prefer an immutable digest over a mutable tag.
EOF
  exit 2
fi

case "$IMAGE" in
  rinha-fraud:local|*:local)
    printf 'submission image must be public, not local: %s\n' "$IMAGE" >&2
    exit 2
    ;;
esac

if [[ "$(git -C "$ROOT" status --porcelain)" != "" ]]; then
  printf 'working tree is not clean; commit or stash before preparing %s\n' "$BRANCH" >&2
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  printf 'jq is required to validate info.json\n' >&2
  exit 1
fi

if ! jq -e '
  (.participants | type == "array" and length > 0) and
  (.social | type == "array" and length > 0) and
  (."source-code-repo" | type == "string" and length > 0) and
  (.stack | type == "array" and length > 0)
' "$ROOT/info.json" >/dev/null; then
  cat >&2 <<'EOF'
info.json is incomplete for an official submission.
Fill participants, social, source-code-repo, and stack before preparing the submission branch.
EOF
  exit 1
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

cp "$ROOT/info.json" "$tmp/info.json"

cat >"$tmp/docker-compose.yml" <<EOF
services:
  lb:
    image: ${IMAGE}
    platform: linux/amd64
    command: ["/usr/local/bin/rinha-lb-c"]
    cpuset: "2,3"
    logging:
      driver: "none"
    ulimits:
      nofile:
        soft: 65535
        hard: 65535
    environment:
      LB_PORT: "9999"
      LB_BACKLOG: "65535"
      LB_ACCEPT_BATCH: "128"
      API_SOCKETS: "/sockets/api1-w0.sock,/sockets/api2-w0.sock"
    volumes:
      - sockets:/sockets
    ports:
      - "9999:9999"
    depends_on:
      - api1
      - api2
    deploy:
      resources:
        limits:
          cpus: "0.40"
          memory: "20MB"

  api1:
    image: ${IMAGE}
    platform: linux/amd64
    cpuset: "0"
    logging:
      driver: "none"
    ulimits:
      nofile:
        soft: 65535
        hard: 65535
    environment:
      API_SOCKET_PREFIX: "/sockets/api1"
      API_WORKERS: "1"
      CONN_POOL_CAP: "512"
      EPOLL_SPIN_US: "0"
      EPOLL_IDLE_US: "60"
      EPOLL_BUSY_POLL_US: "100"
      EPOLL_BUSY_POLL_BUDGET: "8"
      EPOLL_PREFER_BUSY_POLL: "1"
    volumes:
      - sockets:/sockets
    deploy:
      resources:
        limits:
          cpus: "0.30"
          memory: "165MB"

  api2:
    image: ${IMAGE}
    platform: linux/amd64
    cpuset: "1"
    logging:
      driver: "none"
    ulimits:
      nofile:
        soft: 65535
        hard: 65535
    environment:
      API_SOCKET_PREFIX: "/sockets/api2"
      API_WORKERS: "1"
      CONN_POOL_CAP: "512"
      EPOLL_SPIN_US: "0"
      EPOLL_IDLE_US: "60"
      EPOLL_BUSY_POLL_US: "100"
      EPOLL_BUSY_POLL_BUDGET: "8"
      EPOLL_PREFER_BUSY_POLL: "1"
    volumes:
      - sockets:/sockets
    deploy:
      resources:
        limits:
          cpus: "0.30"
          memory: "165MB"

volumes:
  sockets:
    driver: local
    driver_opts:
      type: tmpfs
      device: tmpfs
      o: "size=4m,uid=0,gid=0,mode=0777"
EOF

git -C "$ROOT" switch --orphan "$BRANCH"
git -C "$ROOT" rm -rf . >/dev/null 2>&1 || true
cp "$tmp/docker-compose.yml" "$ROOT/docker-compose.yml"
cp "$tmp/info.json" "$ROOT/info.json"
git -C "$ROOT" add docker-compose.yml info.json
git -C "$ROOT" commit -m "Prepare Rinha submission"

printf 'created %s with docker-compose.yml using image %s\n' "$BRANCH" "$IMAGE"
