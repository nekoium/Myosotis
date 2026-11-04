// Summarize only the final binaries' uncontaminated matched runs.
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const root = path.resolve(__dirname, '../..');
const results = path.join(root, '.tmp/stroke-eval/results');
const runs = [1, 2, 3, 4, 5];
const csv = file => {
  const [header, ...lines] = fs.readFileSync(file, 'utf8').trim().split(/\r?\n/);
  const keys = header.split(',');
  return lines.filter(Boolean).map(line => Object.fromEntries(line.split(',').map((value, i) => [keys[i], Number(value)])));
};
const stats = values => {
  const a = [...values].sort((a, b) => a - b);
  if (!a.length) throw new Error('Empty measurement group');
  const p = q => a[Math.ceil(a.length * q) - 1];
  return { count: a.length, p50: p(.5), p95: p(.95), p99: p(.99), max: a.at(-1) };
};
const summary = { runs, excluded: { earlyV2: 'superseded by frozen final-source builds; external interference affected early batches 6 and 7' }, builds: {} };
for (const build of ['baseline', 'clean', 'natural']) {
  const folders = runs.map(run => path.join(results, `final-${build}-${run}`));
  const rows = folders.flatMap(folder => csv(path.join(folder, 'frames.csv')));
  const fidelity = folders.flatMap(folder => csv(path.join(folder, 'fidelity.csv')));
  const exe = path.join(root, build === 'baseline' ? '.tmp/stroke-trials/baseline/myosotis.exe' : `.tmp/stroke-v2/${build}-final/myosotis.exe`);
  const data = { sha256: crypto.createHash('sha256').update(fs.readFileSync(exe)).digest('hex'), bytes: fs.statSync(exe).size, densities: {} };
  for (const density of [0, 5, 25]) {
    const group = rows.filter(r => r.history_strokes === density);
    if (group.length !== 120) throw new Error(`Expected 120 observations: ${build}/${density}`);
    const curves = fidelity.filter(r => r.history_strokes === density);
    data.densities[density] = {
      input: stats(group.map(r => r.input_ms)), paint: stats(group.map(r => r.paint_ms)),
      combined: stats(group.map(r => r.input_ms + r.paint_ms)),
      privateBytes: { min: Math.min(...group.map(r => r.private_bytes)), max: Math.max(...group.map(r => r.private_bytes)) },
      perRunPaintP95: folders.map(folder => stats(csv(path.join(folder, 'frames.csv')).filter(r => r.history_strokes === density).map(r => r.paint_ms)).p95),
      missingOver3px: curves.reduce((n, r) => n + r.missing_over_3px, 0),
      referencePositions: curves.reduce((n, r) => n + r.samples, 0),
      maxInjectionLateness: Math.max(...curves.map(r => r.max_inject_lateness_ms)),
    };
  }
  data.secondaryEvidence = 'Baseline long/stress/history controls and 60-second sustained runs predate final palette integration; final candidate long/stress/history measurements use frozen final-source builds.';
  const longFolder = build === 'baseline' ? `v2-${build}-long-2` : `final-${build}-long-1`;
  const long = csv(path.join(results, longFolder, 'active.csv'));
  data.longStroke = {};
  for (const [index, length] of [80, 320, 640, 5000, 20000].entries()) {
    const group = long.slice(index * 6, (index + 1) * 6);
    data.longStroke[length] = { catchupPaint: group[0].paint_ms, steadyPaint: stats(group.slice(1).map(r => r.paint_ms)) };
  }
  const prefix = build === 'baseline' ? 'v2' : 'final';
  data.stress = csv(path.join(results, `${prefix}-${build}-stress-1/frames.csv`));
  data.stressFidelity = csv(path.join(results, `${prefix}-${build}-stress-1/fidelity.csv`));
  data.history = csv(path.join(results, `${prefix}-${build}-history-1/history-costs.csv`));
  summary.builds[build] = data;
}
const out = path.join(root, '.tmp/stroke-v2/summary.json');
fs.writeFileSync(out, JSON.stringify(summary, null, 2) + '\n');
for (const [build, data] of Object.entries(summary.builds)) {
  console.log(build, 'bytes', data.bytes);
  for (const [density, values] of Object.entries(data.densities)) console.log(' density', density, 'paint p95', values.paint.p95, 'input p95', values.input.p95, 'combined max', values.combined.max, 'missing', values.missingOver3px, 'run p95', values.perRunPaintP95.join(','));
  for (const [length, values] of Object.entries(data.longStroke)) console.log(' length', length, 'catchup', values.catchupPaint, 'steady p95', values.steadyPaint.p95);
  for (const density of [100, 250]) console.log(' stress', density, stats(data.stress.filter(r => r.history_strokes === density).map(r => r.paint_ms)));
}
console.log('Saved', out);
