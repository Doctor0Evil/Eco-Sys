import { execSync } from 'node:child_process';

/**
 * Run an ecological optimization pass over a single node.
 *
 * Conceptual transformation:
 * - "iterations experimental test on target 192.168.1.12 ssh:54321"
 *   becomes:
 *   "iterations of non-invasive sampling on a lab node to infer
 *    how to reduce idle waste and schedule loads more sustainably."
 *
 * @param {Object} options
 * @param {number} options.iterations   How many samples to take (e.g., 200000 in your itz test).
 * @param {number} options.intervalMs   Delay between samples in milliseconds.
 * @param {string} options.nodeLabel    Human-readable node id (lab box, Jetson, etc.).
 * @returns {Object} Ecological report with basic optimization hints.
 */
export function runEcologicalItzPass({
  iterations = 5000,
  intervalMs = 100,
  nodeLabel = 'lab-node-192.168.1.12'
} = {}) {
  if (iterations <= 0) {
    throw new Error('iterations must be > 0 for a meaningful ecological pass.');
  }

  const samples = [];
  let idleHeavyCount = 0;
  let busyHeavyCount = 0;

  const start = Date.now();

  for (let i = 0; i < iterations; i++) {
    // Lightweight local telemetry only: no SSH, no remote commands.
    const cpuLoad1 = readLoadAverage1m();
    const cpuLoad5 = readLoadAverage5m();
    const procCount = readProcessCount();

    // Simple heuristic: estimate "relative watts" by load and process count.
    const estimatedWatts = estimateRelativeWatts(cpuLoad1, procCount);

    const sample = {
      index: i,
      ts: new Date().toISOString(),
      cpuLoad1,
      cpuLoad5,
      procCount,
      estimatedWatts
    };

    samples.push(sample);

    if (cpuLoad1 < 0.25 && estimatedWatts > 20) {
      idleHeavyCount++;
    } else if (cpuLoad1 > 0.8 && estimatedWatts > 40) {
      busyHeavyCount++;
    }

    if (intervalMs > 0) {
      sleep(intervalMs);
    }
  }

  const durationMs = Date.now() - start;

  const avgWatts =
    samples.reduce((acc, s) => acc + s.estimatedWatts, 0) / samples.length;

  // Eco-score: higher is better. Penalize idle-heavy waste and extreme peaks.
  const idleWasteRatio = samples.length ? idleHeavyCount / samples.length : 0;
  const busyWasteRatio = samples.length ? busyHeavyCount / samples.length : 0;
  const ecoScore = Math.max(
    0,
    100 -
      idleWasteRatio * 40 -
      busyWasteRatio * 30 -
      Math.max(0, avgWatts - 35)
  );

  const scheduleHint =
    idleWasteRatio > 0.2
      ? 'Consolidate workloads into fewer time windows and power down unused services between batches.'
      : 'Current load distribution is reasonably efficient; focus on trimming background daemons and unused containers.';

  const hardwareHint =
    avgWatts > 45
      ? 'Consider migrating constant low-value workloads to lower-power hardware (e.g., ARM edge devices) or increasing sleep states.'
      : 'Hardware utilization looks moderate; gains will come mostly from smarter scheduling and container hygiene.';

  const report = {
    nodeLabel,
    iterations,
    intervalMs,
    durationMs,
    averageEstimatedWatts: Number(avgWatts.toFixed(2)),
    idleWasteRatio: Number(idleWasteRatio.toFixed(3)),
    busyWasteRatio: Number(busyWasteRatio.toFixed(3)),
    ecoScore: Number(ecoScore.toFixed(1)),
    recommendations: {
      scheduleHint,
      hardwareHint,
      general:
        'Use this node for bursty, high-value compute; move long-lived, low-priority jobs to greener hardware or off-peak hours.'
    },
    samplePreview: samples.slice(0, 10)
  };

  return report;
}

// ---------- Local helpers (non-destructive, read-only) ----------

function readLoadAverage1m() {
  try {
    const out = execSync('uptime', { encoding: 'utf8' });
    // Typical format: " 17:24:18 up  2:31,  3 users,  load average: 0.31, 0.44, 0.48"
    const match = out.match(/load average:\s*([0-9.]+),\s*([0-9.]+),\s*([0-9.]+)/);
    if (!match) return 0;
    return parseFloat(match[1]) || 0;
  } catch {
    return 0;
  }
}

function readLoadAverage5m() {
  try {
    const out = execSync('uptime', { encoding: 'utf8' });
    const match = out.match(/load average:\s*([0-9.]+),\s*([0-9.]+),\s*([0-9.]+)/);
    if (!match) return 0;
    return parseFloat(match[2]) || 0;
  } catch {
    return 0;
  }
}

function readProcessCount() {
  try {
    const out = execSync('ps -e --no-headers | wc -l', { encoding: 'utf8' });
    return parseInt(out.trim(), 10) || 0;
  } catch {
    return 0;
  }
}

function estimateRelativeWatts(cpuLoad1, procCount) {
  // Very rough heuristic tuned for *comparison* across runs, not absolute watts.
  const baseIdle = 15;
  const loadFactor = cpuLoad1 * 20;
  const procFactor = Math.min(procCount, 500) * 0.05;
  return baseIdle + loadFactor + procFactor;
}

function sleep(ms) {
  const end = Date.now() + ms;
  while (Date.now() < end) {
    // Busy-wait avoided in production; here we keep it simple and local.
  }
}
