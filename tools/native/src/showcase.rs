use myosotis_tools::{
    graphics::{Canvas, Image, Runtime},
    Result,
};
use std::path::PathBuf;
const INK: u32 = 0xff202634;
const MUTED: u32 = 0xff647084;
const PAPER: u32 = 0xfff3f4f7;
const ACCENT: u32 = 0xff5865d8;
struct Showcase {
    root: PathBuf,
    output: PathBuf,
}
impl Showcase {
    fn crop(&self, c: &Canvas, file: &str, source: [f32; 4], target: [f32; 4]) -> Result<()> {
        c.crop(&Image::load(&self.root.join(file))?, source, target)
    }
    fn header(&self, c: &Canvas, title: &str, subtitle: &str, index: &str) -> Result<()> {
        c.clear(PAPER)?;
        c.text("MYOSOTIS  /  0.2.0", 64., 42., 22., ACCENT, true)?;
        c.text(index, 1580., 42., 20., MUTED, false)?;
        c.text(title, 64., 91., 48., INK, true)?;
        c.text(subtitle, 66., 158., 23., MUTED, false)
    }
    fn badges(&self, c: &Canvas, file: &str, y: f32) -> Result<()> {
        for (i, (x, sy)) in [(864., 396.), (1268., 474.), (1268., 630.)]
            .into_iter()
            .enumerate()
        {
            self.crop(
                c,
                file,
                [x - 44., sy - 34., 88., 68.],
                [1040. + i as f32 * 210., y, 132., 102.],
            )?;
        }
        Ok(())
    }
    fn before_after(&self) -> Result<()> {
        let c = Canvas::new(1800, 1400)?;
        self.header(
            &c,
            "A more natural mark.",
            "Refined coral ink, tapered strokes, and numbers with room to breathe.",
            "01 / 03",
        )?;
        c.text("BEFORE  /  0.1.2", 64., 224., 24., MUTED, true)?;
        c.text("AFTER  /  0.2.0", 916., 224., 24., ACCENT, true)?;
        self.crop(&c, BEFORE, [62., 98., 1354., 866.], [64., 272., 820., 524.])?;
        self.crop(&c, AFTER, [62., 98., 1354., 866.], [916., 272., 820., 524.])?;
        c.text(
            "Original coral + blue controls",
            64.,
            811.,
            22.,
            MUTED,
            false,
        )?;
        c.text(
            "Refined coral + periwinkle controls",
            916.,
            811.,
            22.,
            MUTED,
            false,
        )?;
        c.fill(0xffdde1e9, 64., 866., 1672., 1.)?;
        c.text("Strokes  /  enlarged detail", 64., 902., 28., INK, true)?;
        c.text(
            "Numbered marks  /  enlarged detail",
            916.,
            902.,
            28.,
            INK,
            true,
        )?;
        c.text("0.1.2", 64., 980., 22., MUTED, false)?;
        c.text("0.2.0", 64., 1130., 22., ACCENT, true)?;
        self.crop(&c, BEFORE, [133., 326., 506., 54.], [172., 969., 708., 76.])?;
        self.crop(&c, AFTER, [133., 326., 506., 54.], [172., 1119., 708., 76.])?;
        c.text("0.1.2", 916., 1010., 22., MUTED, false)?;
        self.badges(&c, BEFORE, 968.)?;
        c.text("0.2.0", 916., 1160., 22., ACCENT, true)?;
        self.badges(&c, AFTER, 1118.)?;
        c.text(
            "Same captured path. Thinner starts and ends.",
            64.,
            1240.,
            22.,
            MUTED,
            false,
        )?;
        c.text(
            "Centered numerals. Badges widen for multiple digits.",
            916.,
            1240.,
            22.,
            MUTED,
            false,
        )?;
        c.text("Native app captures on the same synthetic scene at 200% display scaling. Details enlarged equally.",64.,1330.,19.,MUTED,false)?;
        c.save(&self.output.join("01-before-after.png"))
    }
    fn components(&self) -> Result<()> {
        let c = Canvas::new(1800, 1600)?;
        self.header(
            &c,
            "Small controls. Clear feedback.",
            "Both interface themes, four drawing tools, and contextual guidance.",
            "02 / 03",
        )?;
        c.text("DARK", 64., 226., 22., MUTED, true)?;
        c.text("LIGHT", 650., 226., 22., MUTED, true)?;
        for (i, label) in [
            "Pen  /  1",
            "Arrow  /  2",
            "Rectangle  /  3",
            "Numbered mark  /  4",
        ]
        .into_iter()
        .enumerate()
        {
            let y = 276. + i as f32 * 116.;
            for (theme, x) in [("dark", 64.), ("light", 650.)] {
                self.crop(
                    &c,
                    &format!(
                        ".tmp/release-0.2.0/captures/after-{theme}/tool-{}.png",
                        i + 1
                    ),
                    [64., 794., 480., 88.],
                    [x, y, 480., 88.],
                )?;
            }
            c.text(label, 1230., y + 28., 26., INK, true)?;
        }
        c.text("Guidance when you need it", 64., 775., 28., INK, true)?;
        self.crop(&c, AFTER, [64., 898., 798., 64.], [64., 832., 798., 64.])?;
        self.crop(
            &c,
            ".tmp/release-0.2.0/captures/after-light/hover-pen.png",
            [64., 898., 468., 64.],
            [974., 832., 468., 64.],
        )?;
        c.text(
            "Copy or cancel without a keyboard reach.",
            64.,
            913.,
            21.,
            MUTED,
            false,
        )?;
        c.text(
            "Hover for the tool name and shortcut.",
            974.,
            913.,
            21.,
            MUTED,
            false,
        )?;
        c.fill(0xffdde1e9, 64., 972., 1672., 1.)?;
        c.text("Coral marks. Familiar tools.", 64., 1008., 28., INK, true)?;
        for (s, t) in [
            ([133., 326., 506., 54.], [64., 1076., 506., 54.]),
            ([580., 435., 140., 180.], [650., 1049., 140., 180.]),
            ([125., 255., 635., 65.], [867., 1075., 635., 65.]),
            ([820., 358., 90., 76.], [1515., 1070., 90., 76.]),
            ([1224., 436., 88., 76.], [1620., 1070., 88., 76.]),
        ] {
            self.crop(&c, AFTER, s, t)?;
        }
        for (label, x, y) in [
            ("Pen", 64., 1161.),
            ("Arrow", 650., 1240.),
            ("Rectangle", 867., 1161.),
            ("Numbers", 1515., 1161.),
        ] {
            c.text(label, x, y, 20., MUTED, false)?;
        }
        c.text("Pressure follows the pen.", 64., 1322., 28., INK, true)?;
        c.text(
            "Light, heavy, then increasing pressure.",
            64.,
            1376.,
            20.,
            MUTED,
            false,
        )?;
        c.text(
            "Synthetic Windows pen input.",
            64.,
            1410.,
            19.,
            MUTED,
            false,
        )?;
        self.crop(
            &c,
            ".tmp/stroke-eval/results/release-020-shapes-1/pressure.png",
            [80., 520., 980., 200.],
            [650., 1300., 980., 200.],
        )?;
        c.text("Actual 0.2.0 renderings. Themes affect controls; coral annotations stay consistent. Real stylus hardware remains unqualified.",64.,1535.,18.,MUTED,false)?;
        c.save(&self.output.join("02-components.png"))
    }
    fn stress(&self, before: [f64; 4], after: [f64; 4]) -> Result<()> {
        let c = Canvas::new(1800, 1640)?;
        self.header(
            &c,
            "More ink. The same drawing pace.",
            "A 240 Hz curve replayed over 250 completed strokes. Actual test captures below.",
            "03 / 03",
        )?;
        c.text("0.1.2  /  baseline", 64., 224., 24., MUTED, true)?;
        c.text("0.2.0  /  Natural", 916., 224., 24., ACCENT, true)?;
        self.crop(
            &c,
            ".tmp/stroke-eval/results/release-020-baseline-1/curve-250.png",
            [0., 0., 1400., 850.],
            [64., 272., 820., 498.],
        )?;
        self.crop(
            &c,
            ".tmp/stroke-eval/results/release-020-natural-1/curve-250.png",
            [0., 0., 1400., 850.],
            [916., 272., 820., 498.],
        )?;
        c.text(
            "Fast replay loses curve detail at this density.",
            64.,
            788.,
            21.,
            MUTED,
            false,
        )?;
        c.text(
            "Captured curve stays within the coverage check.",
            916.,
            788.,
            21.,
            MUTED,
            false,
        )?;
        c.fill(0xffdde1e9, 64., 844., 1672., 1.)?;
        // Native GDI+ bars replace the managed Chart control, using the same values and 0..150 ms scale.
        chart(&c, before, after)?;
        c.text(
            &format!("{:.1} ms", after[3]),
            1220.,
            934.,
            58.,
            ACCENT,
            true,
        )?;
        c.text("p95 paint at 250 strokes", 1224., 1013., 24., INK, true)?;
        c.text(
            &format!("Baseline: {:.1} ms", before[3]),
            1224.,
            1061.,
            23.,
            MUTED,
            false,
        )?;
        c.text(
            "5 alternating runs per build",
            1224.,
            1130.,
            22.,
            MUTED,
            false,
        )?;
        c.text(
            "120 observations per density",
            1224.,
            1168.,
            22.,
            MUTED,
            false,
        )?;
        c.text(
            "2 seconds per curve at 240 Hz",
            1224.,
            1206.,
            22.,
            MUTED,
            false,
        )?;
        c.text("0.2.0: 0 of 9,200 checked reference positions more than 3 px from ink across all four densities.",64.,1360.,23.,INK,true)?;
        c.text(
            "60-second sustained replay  /  240 Hz  /  25 historical strokes",
            64.,
            1415.,
            25.,
            INK,
            true,
        )?;
        c.text("Packaged executable: all 14,380 checked reference positions within 3 px of ink; max injection lateness 0.62 ms.",64.,1459.,20.,MUTED,false)?;
        c.text("1400 x 850 physical-pixel canvas; 200% display scaling. Forced-paint wall time, not input-to-display latency.",64.,1532.,19.,MUTED,false)?;
        c.text("Single-machine measurements. Coverage is an ink-mask diagnostic, not proof of exact sample retention or unlimited capacity.",64.,1571.,18.,MUTED,false)?;
        c.save(&self.output.join("03-stress-test.png"))
    }
}
fn chart(c: &Canvas, before: [f64; 4], after: [f64; 4]) -> Result<()> {
    c.text("p95 paint time (ms)", 100., 897., 18., INK, false)?;
    for (x, label, color) in [(500., "0.1.2", 0xff929bab), (675., "0.2.0", ACCENT)] {
        c.fill(color, x, 897., 20., 16.)?;
        c.text(label, x + 30., 893., 18., INK, false)?;
    }
    for tick in [0, 50, 100, 150] {
        let y = 1240. - tick as f32 * 1.8;
        c.fill(0xffdde1e9, 160., y, 930., 1.)?;
        c.text(&tick.to_string(), 110., y - 10., 18., MUTED, false)?;
    }
    for (i, label) in ["0", "25", "100", "250"].into_iter().enumerate() {
        let x = 220. + i as f32 * 225.;
        for (value, dx, color) in [(before[i], 0., 0xff929bab), (after[i], 60., ACCENT)] {
            if !value.is_finite() || !(0.0..=150.0).contains(&value) {
                return Err("Chart value outside 0..150 ms".into());
            }
            let h = value as f32 * 1.8;
            c.fill(color, x + dx, 1240. - h, 50., h)?;
            c.text(
                &format!("{value:.1}"),
                x + dx - 2.,
                1210. - h,
                18.,
                INK,
                true,
            )?;
        }
        c.text(label, x + 30., 1253., 18., INK, false)?;
    }
    c.text("Completed long strokes", 460., 1295., 20., INK, false)
}
const BEFORE: &str = ".tmp/release-0.2.0/captures/before/scene.png";
const AFTER: &str = ".tmp/release-0.2.0/captures/after-dark/scene.png";
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.is_empty() || args.len() > 2 {
        return Err("Usage: release-showcase ROOT [OUTPUT_DIRECTORY]".into());
    }
    let root = PathBuf::from(&args[0]);
    let output = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("docs/releases/0.2.0"));
    let summary: serde_json::Value = serde_json::from_slice(&std::fs::read(
        root.join("docs/releases/0.2.0/performance.json"),
    )?)?;
    let values = |build: &str| -> Result<[f64; 4]> {
        let mut result = [0.; 4];
        for (i, density) in ["0", "25", "100", "250"].into_iter().enumerate() {
            result[i] = summary["builds"][build]["densities"][density]["paintP95"]
                .as_f64()
                .ok_or("Missing paintP95")?;
        }
        Ok(result)
    };
    let before = values("baseline")?;
    let after = values("natural")?;
    let _runtime = Runtime::new()?;
    std::fs::create_dir_all(&output)?;
    let showcase = Showcase { root, output };
    showcase.before_after()?;
    showcase.components()?;
    showcase.stress(before, after)
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
