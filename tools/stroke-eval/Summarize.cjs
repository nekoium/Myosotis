// Summarize diagnostic data; this is not the proposed release gate.
const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '../../.tmp/stroke-eval');
const csv = file => {
  const [header, ...rows] = fs.readFileSync(file, 'utf8').trim().split(/\r?\n/).map(line => line.split(','));
  return rows.map(row => Object.fromEntries(header.map((key, i) => [key, Number(row[i])])));
};
const percentile = (values, p) => {
  values.sort((a, b) => a - b);
  return Number(values[Math.max(0, Math.ceil(values.length * p) - 1)].toFixed(3));
};
const summary = [];
for (const name of fs.readdirSync(path.join(root, 'results')).sort()) {
  const folder = path.join(root, 'results', name);
  const rows = csv(path.join(folder, 'frames.csv'));
  const fidelity = csv(path.join(folder, 'fidelity.csv'));
  for (const count of [...new Set(rows.map(row => row.history_strokes))].sort((a,b)=>a-b)) {
    const subset = rows.filter(row => row.history_strokes === count);
    const curve = fidelity.find(row => row.history_strokes === count);
    const record = {
      run: name, history: count, frames: subset.length,
      paint_p50_ms: percentile(subset.map(row => row.paint_ms), .5),
      paint_p95_ms: percentile(subset.map(row => row.paint_ms), .95),
      dispatch_p95_ms: percentile(subset.map(row => row.input_ms), .95),
      combined_p95_ms: percentile(subset.map(row => row.paint_ms + row.input_ms), .95),
      private_mb: Number((Math.max(...subset.map(row => row.private_bytes)) / 1e6).toFixed(1)),
    };
    if (curve) Object.assign(record, {
      curve_p95_distance_px: curve.p95_distance_px,
      curve_missing_percent: Number((100 * curve.missing_over_3px / curve.samples).toFixed(1)),
      inject_max_lateness_ms: curve.max_inject_lateness_ms,
    });
    const staticFile = path.join(folder, `static-${count}.csv`);
    if (fs.existsSync(staticFile)) record.static_p95_ms = percentile(csv(staticFile).map(row=>row.paint_ms), .95);
    summary.push(record);
  }
  const activeFile = path.join(folder, 'active.csv');
  if (fs.existsSync(activeFile)) {
    const active = csv(activeFile);
    for (const limit of [80,320,640]) {
      const subset = active.filter(row=>row.samples > limit && row.samples <= limit+6);
      summary.push({run:name, active_samples:limit, frames:subset.length,
        paint_p95_ms:percentile(subset.map(row=>row.paint_ms),.95),
        combined_p95_ms:percentile(subset.map(row=>row.paint_ms+row.input_ms),.95)});
    }
  }
}
fs.writeFileSync(path.join(root, 'summary.json'), JSON.stringify(summary, null, 2));
for (const record of summary) console.log(JSON.stringify(record));
