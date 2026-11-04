//! Unattended migration checks against saved diagnostic results and managed showcase output.
use myosotis_tools::{
    graphics::{Image, Runtime},
    metrics::{export_difference, fidelity},
    Result,
};
use std::{path::PathBuf, time::Instant};
fn run() -> Result<()> {
    let root = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("Usage: verify_saved ROOT")?,
    );
    let _runtime = Runtime::new()?;
    let started = Instant::now();
    let mut checks = 0;
    for entry in std::fs::read_dir(root.join(".tmp/stroke-eval/results"))? {
        let folder = entry?.path();
        let name = folder.file_name().unwrap().to_string_lossy();
        if !name.starts_with("release-020-") {
            continue;
        }
        let csv = folder.join("fidelity.csv");
        if !csv.is_file() {
            continue;
        }
        let environment = std::fs::read_to_string(folder.join("environment.txt"))?;
        let (width, height) = environment
            .lines()
            .find_map(|line| line.strip_prefix("canvas="))
            .ok_or("Missing canvas")?
            .split_once('x')
            .ok_or("Invalid canvas")?;
        let (width, height): (i32, i32) = (width.parse()?, height.parse()?);
        for row in std::fs::read_to_string(csv)?
            .lines()
            .skip(1)
            .filter(|s| !s.trim().is_empty())
        {
            let fields: Vec<_> = row.split(',').collect();
            let n = fields[0];
            let samples: i32 = fields[4].parse()?;
            let seconds = (samples + 20) / 240;
            if samples + 20 != seconds * 240 {
                continue;
            } // Historical exploratory drivers used different trace lengths.
            let image = Image::load(&folder.join(format!("curve-{n}.png")))?;
            if (image.width, image.height) != (width, height) {
                return Err("Canvas size mismatch".into());
            }
            let (median, p95, missing, count) = fidelity(&image.pixels()?, width, height, seconds);
            let actual = format!("{median:.3},{p95:.3},{missing},{count}");
            let expected = fields[1..5].join(",");
            if actual != expected {
                return Err(
                    format!("{}: actual {actual}, expected {expected}", folder.display()).into(),
                );
            }
            checks += 1;
        }
        if folder.join("export-comparison.txt").is_file() {
            let before = Image::load(&folder.join("pressure.png"))?;
            let after = Image::load(&folder.join("export.png"))?;
            let actual = export_difference(&before.pixels()?, &after.pixels()?, width, height);
            let report = std::fs::read_to_string(folder.join("export-comparison.txt"))?;
            for (key, value) in [
                ("exact_pixel_differences", actual.0),
                ("mask_differences", actual.1),
                ("mask_differences_beyond_1px", actual.2),
            ] {
                let expected: usize = report
                    .lines()
                    .find_map(|l| l.strip_prefix(&format!("{key}=")))
                    .ok_or("Missing comparison field")?
                    .parse()?;
                if value != expected {
                    return Err(format!("{} {key}: {value} != {expected}", folder.display()).into());
                }
            }
            checks += 1;
        }
    }
    if checks == 0 {
        return Err("No saved release probe results found under .tmp/stroke-eval/results".into());
    }
    println!(
        "Saved probe comparisons: {checks} passed in {:.3} ms",
        started.elapsed().as_secs_f64() * 1000.
    );
    for file in [
        "01-before-after.png",
        "02-components.png",
        "03-stress-test.png",
    ] {
        let a = Image::load(&root.join("docs/releases/0.2.0").join(file))?;
        let b = Image::load(&root.join(".tmp/rust-tools-validation/rust").join(file))?;
        if (a.width, a.height) != (b.width, b.height) {
            return Err("Showcase dimensions changed".into());
        }
        let ap = a.pixels()?;
        let bp = b.pixels()?;
        let mut changed = 0;
        let mut outside_chart = 0;
        let mut bounds = (1800, 2000, 0, 0);
        for (i, (a, b)) in ap.iter().zip(&bp).enumerate() {
            if a != b {
                changed += 1;
                let x = i as i32 % 1800;
                let y = i as i32 / 1800;
                if file != "03-stress-test.png"
                    || !(48..1148).contains(&x)
                    || !(880..1325).contains(&y)
                {
                    outside_chart += 1;
                    bounds = (
                        bounds.0.min(x),
                        bounds.1.min(y),
                        bounds.2.max(x),
                        bounds.3.max(y),
                    );
                }
            }
        }
        if outside_chart != 0 {
            return Err("Showcase differs outside the replaced chart".into());
        }
        println!(
            "{file}: {changed} changed pixels, {outside_chart} outside chart, bounds {bounds:?}"
        );
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
