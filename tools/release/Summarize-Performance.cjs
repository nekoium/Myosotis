// Aggregate the five alternating release comparisons; fail if evidence is incomplete.
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const root = path.resolve(__dirname, '../..');
const output = path.join(root, 'docs/releases/0.2.0');
const results = path.join(root, '.tmp/stroke-eval/results');
const csv = file => {
  const lines = fs.readFileSync(file, 'utf8').trim().split(/\r?\n/);
  const columns = lines.shift().split(',');
  return lines.map(line => Object.fromEntries(line.split(',').map((value, i) => [columns[i], Number(value)])));
};
const percentile = (values, p) => {
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.ceil(p * sorted.length) - 1];
};
const summary = {
  measurement: 'External forced-paint wall time; not input-to-display latency',
  canvas: { width: 1400, height: 850, unit: 'physical pixels', displayScaling: '200%' },
  runs: 5, framesPerRunPerDensity: 24, warmupsPerRunPerDensity: 5,
  replay: { rateHz: 240, secondsPerCurve: 2, referencePositionsPerCurve: 460 },
  order: ['baseline,natural', 'natural,baseline', 'baseline,natural', 'natural,baseline', 'baseline,natural'],
  builds: {},
};
const framesOutput = ['build,run,history_strokes,frame,input_ms,paint_ms,private_bytes'];
const fidelityOutput = ['build,run,history_strokes,median_distance_px,p95_distance_px,missing_over_3px,reference_samples,max_injection_lateness_ms'];
for (const [build, relative] of [['baseline', '.tmp/stroke-trials/baseline/myosotis.exe'], ['natural', '.tmp/release-0.2.0/build/release/myosotis.exe']]) {
  const executable = fs.readFileSync(path.join(root, relative));
  const frames = [], fidelity = [];
  for (let run = 1; run <= 5; run++) {
    const folder = path.join(results, `release-020-${build}-${run}`);
    const f = csv(path.join(folder, 'frames.csv'));
    const q = csv(path.join(folder, 'fidelity.csv'));
    frames.push(...f); fidelity.push(...q);
    for (const row of f) framesOutput.push([build, run, row.history_strokes, row.frame, row.input_ms, row.paint_ms, row.private_bytes].join(','));
    for (const row of q) fidelityOutput.push([build, run, ...Object.values(row)].join(','));
  }
  const data = { version: build === 'baseline' ? '0.1.2' : '0.2.0', sha256: crypto.createHash('sha256').update(executable).digest('hex'), bytes: executable.length, densities: {} };
  for (const density of [0, 25, 100, 250]) {
    const f = frames.filter(row => row.history_strokes === density);
    const q = fidelity.filter(row => row.history_strokes === density);
    if (f.length !== 120 || q.length !== 5 || q.some(row => row.samples !== 460 || !Number.isFinite(row.max_inject_lateness_ms) || Object.values(row).some(value => !Number.isFinite(value)))) throw Error(`Incomplete evidence: ${build}/${density}`);
    data.densities[density] = {
      count: f.length, paintP50: percentile(f.map(row => row.paint_ms), .5), paintP95: percentile(f.map(row => row.paint_ms), .95), paintMax: Math.max(...f.map(row => row.paint_ms)),
      inputP95: percentile(f.map(row => row.input_ms), .95),
      missingOver3px: q.reduce((sum, row) => sum + row.missing_over_3px, 0),
      referencePositions: q.reduce((sum, row) => sum + row.samples, 0),
      maxInjectionLatenessMs: Math.max(...q.map(row => row.max_inject_lateness_ms)),
    };
  }
  summary.builds[build] = data;
}
const packagedExe = fs.readFileSync(path.join(root, 'dist/Myosotis/myosotis.exe'));
summary.packagedBuild = { sha256: crypto.createHash('sha256').update(packagedExe).digest('hex'), bytes: packagedExe.length };
const sustained = csv(path.join(results, 'release-020-sustained-2/fidelity.csv'));
if (sustained.length !== 1 || sustained[0].samples !== 14380 || sustained[0].missing_over_3px !== 0) throw Error('Incomplete sustained replay');
summary.sustained = { seconds: 60, rateHz: 240, historicalStrokes: 25, ...sustained[0] };
summary.longStroke = csv(path.join(results, 'release-020-long-1/active.csv'));
summary.history = csv(path.join(results, 'release-020-history-1/history-costs.csv'));
fs.mkdirSync(output, { recursive: true });
fs.writeFileSync(path.join(output, 'performance.json'), JSON.stringify(summary, null, 2) + '\n');
fs.writeFileSync(path.join(output, 'performance-frames.csv'), framesOutput.join('\n') + '\n');
fs.writeFileSync(path.join(output, 'performance-fidelity.csv'), fidelityOutput.join('\n') + '\n');
for (const density of [0, 25, 100, 250]) console.log(density, ...['baseline', 'natural'].map(build => `${build}: ${summary.builds[build].densities[density].paintP95} ms p95; missing ${summary.builds[build].densities[density].missingOver3px}/${summary.builds[build].densities[density].referencePositions}`));
