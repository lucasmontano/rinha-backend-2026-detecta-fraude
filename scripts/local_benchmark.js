#!/usr/bin/env node
const fs = require("fs");
const http = require("http");
const { performance } = require("perf_hooks");

const DEFAULT_TARGET = "http://127.0.0.1:9999/fraud-score";

function parseArgs(argv) {
  const args = {
    target: process.env.TARGET_URL || DEFAULT_TARGET,
    data: process.env.TEST_DATA || "",
    out: process.env.OUT || "",
    concurrency: Number(process.env.CONCURRENCY || 16),
    maxRequests: Number(process.env.MAX_REQUESTS || 0),
  };

  for (let i = 2; i < argv.length; i += 1) {
    const arg = argv[i];
    const next = argv[i + 1];
    if (arg === "--target") {
      args.target = next;
      i += 1;
    } else if (arg === "--data") {
      args.data = next;
      i += 1;
    } else if (arg === "--out") {
      args.out = next;
      i += 1;
    } else if (arg === "--concurrency") {
      args.concurrency = Number(next);
      i += 1;
    } else if (arg === "--max-requests") {
      args.maxRequests = Number(next);
      i += 1;
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }

  if (!args.data) throw new Error("--data is required");
  if (!Number.isFinite(args.concurrency) || args.concurrency < 1) {
    throw new Error("--concurrency must be a positive number");
  }
  if (!Number.isFinite(args.maxRequests) || args.maxRequests < 0) {
    throw new Error("--max-requests must be zero or a positive number");
  }

  return args;
}

function classify(expectedApproved, status, body) {
  if (status !== 200) return "error";

  let parsed;
  try {
    parsed = JSON.parse(body);
  } catch {
    return "error";
  }

  if (typeof parsed.approved !== "boolean") return "error";

  if (expectedApproved === parsed.approved) {
    return parsed.approved ? "tn" : "tp";
  }

  return parsed.approved ? "fn" : "fp";
}

function scoreResult(expectedStats, p99, counts, meta) {
  const K = 1000;
  const T_MAX_MS = 1000;
  const P99_MIN_MS = 1;
  const P99_MAX_MS = 2000;
  const EPSILON_MIN = 0.001;
  const BETA = 300;
  const TX_CORTE = 0.15;
  const SCORE_P99_CORTE = -3000;
  const SCORE_DET_CORTE = -3000;

  const tp = counts.tp;
  const tn = counts.tn;
  const fp = counts.fp;
  const fn = counts.fn;
  const errs = counts.error;
  const n = tp + tn + fp + fn + errs;
  const weightedErrors = fp + fn * 3 + errs * 5;
  const failures = fp + fn + errs;
  const epsilon = n > 0 ? weightedErrors / n : 0;
  const failureRate = n > 0 ? failures / n : 0;

  let p99Score;
  let p99CutTriggered = false;
  if (p99 <= 0) {
    p99Score = 0;
  } else if (p99 > P99_MAX_MS) {
    p99Score = SCORE_P99_CORTE;
    p99CutTriggered = true;
  } else {
    p99Score = K * Math.log10(T_MAX_MS / Math.max(p99, P99_MIN_MS));
  }

  let detScore;
  let rateComponent = 0;
  let absolutePenalty = 0;
  let detCutTriggered = false;
  if (failureRate > TX_CORTE) {
    detScore = SCORE_DET_CORTE;
    detCutTriggered = true;
  } else {
    rateComponent = K * Math.log10(1 / Math.max(epsilon, EPSILON_MIN));
    absolutePenalty = -BETA * Math.log10(1 + weightedErrors);
    detScore = rateComponent + absolutePenalty;
  }

  return {
    expected: expectedStats,
    p99,
    scoring: {
      breakdown: {
        false_positive_detections: fp,
        false_negative_detections: fn,
        true_positive_detections: tp,
        true_negative_detections: tn,
        http_errors: errs,
      },
      failure_rate: Math.round(failureRate * 1_000_000) / 1_000_000,
      weighted_errors_E: weightedErrors,
      error_rate_epsilon: epsilon,
      p99_score: {
        value: p99Score,
        cut_triggered: p99CutTriggered,
      },
      detection_score: {
        value: detScore,
        rate_component: detCutTriggered ? null : rateComponent,
        absolute_penalty: detCutTriggered ? null : absolutePenalty,
        cut_triggered: detCutTriggered,
      },
      final_score: p99Score + detScore,
    },
    local: meta,
  };
}

function percentile(values, percentileValue) {
  if (values.length === 0) return 0;
  const sorted = [...values].sort((a, b) => a - b);
  const index = Math.min(
    sorted.length - 1,
    Math.max(0, Math.ceil((percentileValue / 100) * sorted.length) - 1),
  );
  return sorted[index];
}

function postJson(target, agent, payload) {
  const url = new URL(target);
  const body = JSON.stringify(payload);
  const started = performance.now();

  return new Promise((resolve) => {
    const request = http.request(
      {
        method: "POST",
        protocol: url.protocol,
        hostname: url.hostname,
        port: url.port,
        path: `${url.pathname}${url.search}`,
        headers: {
          "Content-Type": "application/json",
          "Content-Length": Buffer.byteLength(body),
        },
        timeout: 2001,
        agent,
      },
      (response) => {
        let responseBody = "";
        response.setEncoding("utf8");
        response.on("data", (chunk) => {
          responseBody += chunk;
        });
        response.on("end", () => {
          resolve({
            status: response.statusCode || 0,
            body: responseBody,
            latency: performance.now() - started,
          });
        });
      },
    );

    request.on("timeout", () => {
      request.destroy(new Error("request timeout"));
    });
    request.on("error", () => {
      resolve({
        status: 0,
        body: "",
        latency: performance.now() - started,
      });
    });
    request.write(body);
    request.end();
  });
}

async function run() {
  const args = parseArgs(process.argv);
  const raw = JSON.parse(fs.readFileSync(args.data, "utf8"));
  const entries = raw.entries || raw;
  const expectedStats = raw.stats || { total: entries.length };
  const requestedEntries =
    args.maxRequests > 0 ? entries.slice(0, args.maxRequests) : entries;

  const agent = new http.Agent({
    keepAlive: true,
    maxSockets: args.concurrency,
    maxFreeSockets: args.concurrency,
  });

  const counts = { tp: 0, tn: 0, fp: 0, fn: 0, error: 0 };
  const latencies = [];
  let nextIndex = 0;
  const startedAt = new Date();
  const started = performance.now();

  async function worker() {
    while (true) {
      const index = nextIndex;
      nextIndex += 1;
      if (index >= requestedEntries.length) return;

      const entry = requestedEntries[index];
      const response = await postJson(args.target, agent, entry.request);
      latencies.push(response.latency);
      const bucket = classify(entry.expected_approved, response.status, response.body);
      counts[bucket] += 1;
    }
  }

  await Promise.all(
    Array.from(
      { length: Math.min(args.concurrency, requestedEntries.length) },
      () => worker(),
    ),
  );

  agent.destroy();
  const durationMs = performance.now() - started;
  const p99 = percentile(latencies, 99);
  const result = scoreResult(expectedStats, p99, counts, {
    target: args.target,
    data: args.data,
    started_at: startedAt.toISOString(),
    finished_at: new Date().toISOString(),
    total_requests: requestedEntries.length,
    concurrency: args.concurrency,
    duration_ms: durationMs,
    requests_per_second:
      durationMs > 0 ? requestedEntries.length / (durationMs / 1000) : 0,
    latency_ms: {
      min: Math.min(...latencies),
      mean: latencies.reduce((sum, value) => sum + value, 0) / latencies.length,
      p50: percentile(latencies, 50),
      p90: percentile(latencies, 90),
      p95: percentile(latencies, 95),
      p99,
      max: Math.max(...latencies),
    },
  });

  const output = JSON.stringify(result, null, 2);
  if (args.out) {
    fs.mkdirSync(require("path").dirname(args.out), { recursive: true });
    fs.writeFileSync(args.out, output);
  }
  process.stdout.write(`${output}\n`);
}

run().catch((error) => {
  console.error(error.stack || error.message);
  process.exit(1);
});
