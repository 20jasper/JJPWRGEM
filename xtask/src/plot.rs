use std::cmp::Ordering;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow, bail};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use plotters::coord::Shift;
use plotters::prelude::*;
use plotters::style::{FontStyle, register_font};
use serde::Deserialize;

const CHART_WIDTH: u32 = 1280;
const CHART_HEIGHT: u32 = 720;
// Liberation Sans Narrow Regular is licensed under the SIL Open Font License 1.1.
const EMBEDDED_FONT_BASE64: &str =
    include_str!("../assets/fonts/liberation_sans_narrow_regular.ttf.base64");

static FONT_BYTES: OnceLock<&'static [u8]> = OnceLock::new();

#[derive(Deserialize)]
struct PrettyCanadaResults {
    results: Vec<BenchmarkResult>,
}

#[derive(Deserialize)]
struct BenchmarkResult {
    command: String,
    times: Vec<f64>,
}

#[derive(Clone, Copy)]
struct CandlestickStats {
    min: f64,
    q1: f64,
    median: f64,
    q3: f64,
    max: f64,
}

struct SeriesEntry {
    name: String,
    time: CandlestickStats,
}

struct BenchmarkDataset {
    stem: &'static str,
    action: BenchmarkAction,
    info: &'static DatasetInfo,
}

struct DatasetInfo {
    display_name: &'static str,
    description: &'static str,
}

#[derive(Clone, Copy)]
enum BenchmarkAction {
    Pretty,
    Minify,
}

impl BenchmarkDataset {
    fn title(&self) -> String {
        format!(
            "JSON {} {} Benchmark - Execution Time",
            self.action.title_label(),
            self.info.display_name
        )
    }

    fn subtitle(&self) -> String {
        format!(
            "{} a {}",
            self.action.subtitle_prefix(),
            self.info.description
        )
    }
}

impl BenchmarkAction {
    fn title_label(self) -> &'static str {
        match self {
            Self::Pretty => "pretty print",
            Self::Minify => "minify",
        }
    }

    fn subtitle_prefix(self) -> &'static str {
        match self {
            Self::Pretty => "Parsing and stringifying",
            Self::Minify => "Parsing and stringifying",
        }
    }
}

const CANADA_INFO: DatasetInfo = DatasetInfo {
    display_name: "Canada",
    description: "2.2MB JSON file with lots of lightly nested arrays",
};

const CITM_CATALOG_INFO: DatasetInfo = DatasetInfo {
    display_name: "CITM catalog",
    description: "1.7MB JSON file with lots of lightly nested, long objects",
};

const TWITTER_INFO: DatasetInfo = DatasetInfo {
    display_name: "Twitter",
    description: ".6MB JSON file with lots of lightly nested, short objects",
};

const BENCHMARK_DATASETS: &[BenchmarkDataset] = &[
    BenchmarkDataset {
        stem: "pretty-canada",
        action: BenchmarkAction::Pretty,
        info: &CANADA_INFO,
    },
    BenchmarkDataset {
        stem: "pretty-citm_catalog",
        action: BenchmarkAction::Pretty,
        info: &CITM_CATALOG_INFO,
    },
    BenchmarkDataset {
        stem: "pretty-twitter",
        action: BenchmarkAction::Pretty,
        info: &TWITTER_INFO,
    },
    BenchmarkDataset {
        stem: "ugly-canada",
        action: BenchmarkAction::Minify,
        info: &CANADA_INFO,
    },
    BenchmarkDataset {
        stem: "ugly-citm_catalog",
        action: BenchmarkAction::Minify,
        info: &CITM_CATALOG_INFO,
    },
    BenchmarkDataset {
        stem: "ugly-twitter",
        action: BenchmarkAction::Minify,
        info: &TWITTER_INFO,
    },
];

fn bench_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bench/output")
}

pub fn plot_all_benchmarks() -> Result<()> {
    let output_dir = bench_output_dir();
    let mut failures: Vec<(&'static str, anyhow::Error)> = Vec::new();

    for dataset in BENCHMARK_DATASETS {
        let input = output_dir.join(format!("{}.json", dataset.stem));
        let output = output_dir.join(format!("{}.png", dataset.stem));
        if let Err(err) =
            plot_benchmark_candlesticks(&input, &output, &dataset.title(), &dataset.subtitle())
        {
            failures.push((dataset.stem, err));
        }
    }

    if failures.is_empty() {
        Ok(())
    } else {
        let summary = failures
            .into_iter()
            .map(|(stem, err)| format!("{stem}: {err}"))
            .collect::<Vec<_>>()
            .join("\n");
        Err(anyhow!("failed to plot datasets:\n{summary}"))
    }
}

impl CandlestickStats {
    fn scaled(&self, factor: f64) -> Self {
        Self {
            min: self.min * factor,
            q1: self.q1 * factor,
            median: self.median * factor,
            q3: self.q3 * factor,
            max: self.max * factor,
        }
    }
}

pub fn plot_benchmark_candlesticks(
    input: &Path,
    output: &Path,
    title: &str,
    subtitle: &str,
) -> Result<()> {
    ensure_font_registered()?;

    let file = File::open(input)
        .with_context(|| format!("failed to open input file {}", input.display()))?;
    let reader = BufReader::new(file);
    let parsed: PrettyCanadaResults =
        serde_json::from_reader(reader).context("failed to parse benchmark data")?;

    let mut entries = Vec::new();
    for result in parsed.results {
        let Some(time_stats) = compute_stats(&result.times) else {
            bail!("benchmark '{}' is missing time samples", result.command);
        };

        entries.push(SeriesEntry {
            name: result.command,
            time: time_stats,
        });
    }

    if entries.is_empty() {
        bail!("no benchmark entries found");
    }

    entries.sort_by(|left, right| {
        left.time
            .median
            .partial_cmp(&right.time.median)
            .unwrap_or(Ordering::Equal)
    });

    if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }

    let root = BitMapBackend::new(output, (CHART_WIDTH, CHART_HEIGHT)).into_drawing_area();
    let background = RGBColor(255, 255, 255);
    root.fill(&background)
        .map_err(|err| anyhow!("failed to clear drawing area: {err}"))?;

    let names: Vec<String> = entries.iter().map(|entry| entry.name.clone()).collect();
    let raw_max = entries
        .iter()
        .map(|entry| entry.time.max * 1_000.0)
        .fold(f64::NEG_INFINITY, |acc, value| acc.max(value));
    let time_color = RGBColor(33, 150, 243);
    let chart_area = root
        .titled(title, ("sans-serif", 28).into_font())
        .map_err(|err| anyhow!("failed to render chart title: {err}"))?;
    let chart_area = chart_area
        .titled(subtitle, ("sans-serif", 22).into_font())
        .map_err(|err| anyhow!("failed to render chart subtitle: {err}"))?;

    let time_range = linear_range(raw_max);
    draw_candlesticks(
        &chart_area,
        "Milliseconds",
        &names,
        &entries,
        |entry| entry.time.scaled(1_000.0),
        time_range,
        time_color,
        |_entry, stats| format!("{:.1} ms", stats.median),
    )?;

    root.present()
        .map_err(|err| anyhow!("failed to write chart image: {err}"))?;
    Ok(())
}

fn ensure_font_registered() -> Result<()> {
    if FONT_BYTES.get().is_some() {
        return Ok(());
    }

    let sanitized: String = EMBEDDED_FONT_BASE64.split_whitespace().collect();
    let decoded = STANDARD
        .decode(sanitized)
        .context("failed to decode embedded font data")?;
    let leaked: &'static [u8] = Box::leak(decoded.into_boxed_slice());
    register_font("sans-serif", FontStyle::Normal, leaked)
        .map_err(|_| anyhow!("embedded font registration failed"))?;
    let _ = FONT_BYTES.set(leaked);
    Ok(())
}

type SeriesAccessor = fn(&SeriesEntry) -> CandlestickStats;
type LabelFormatter = fn(&SeriesEntry, CandlestickStats) -> String;

#[allow(clippy::too_many_arguments)]
fn draw_candlesticks(
    area: &DrawingArea<BitMapBackend, Shift>,
    y_label: &str,
    names: &[String],
    entries: &[SeriesEntry],
    accessor: SeriesAccessor,
    y_range: std::ops::Range<f64>,
    color: RGBColor,
    label_formatter: LabelFormatter,
) -> Result<()> {
    let x_range = 0..(entries.len() as i32);
    let grid_color = RGBColor(210, 210, 210);
    let label_count = 8;
    let candle_width: u32 = 28;
    let mut chart = ChartBuilder::on(area)
        .margin(25)
        .set_label_area_size(LabelAreaPosition::Left, 70)
        .set_label_area_size(LabelAreaPosition::Bottom, 90)
        .build_cartesian_2d(x_range.clone(), y_range.clone())
        .map_err(|err| anyhow!("failed to build candlestick chart: {err}"))?;

    let axis_font = ("sans-serif", 20).into_font();
    chart
        .configure_mesh()
        .light_line_style(ShapeStyle::from(grid_color).stroke_width(1))
        .bold_line_style(ShapeStyle::from(grid_color).stroke_width(2))
        .x_desc("Command")
        .y_desc(y_label)
        .axis_desc_style(("sans-serif", 20).into_font())
        .label_style(axis_font.clone())
        .x_labels(names.len())
        .x_label_formatter(&|value| {
            if *value < 0 {
                return String::new();
            }
            names.get(*value as usize).cloned().unwrap_or_default()
        })
        .x_label_style(axis_font.clone())
        .y_labels(label_count)
        .y_label_formatter(&|value| format!("{value:.0}"))
        .draw()
        .map_err(|err| anyhow!("failed to configure chart mesh: {err}"))?;

    let candle_style = ShapeStyle::from(color).filled().stroke_width(3);
    let whisker_style = ShapeStyle::from(color.mix(0.7)).stroke_width(10);
    let label_font = ("sans-serif", 20).into_font();
    let label_color = RGBColor(30, 30, 30);

    chart
        .draw_series(entries.iter().enumerate().map(|(index, entry)| {
            let stats = accessor(entry);
            CandleStick::new(
                index as i32,
                stats.q1,
                stats.max,
                stats.min,
                stats.q3,
                candle_style,
                whisker_style,
                candle_width,
            )
        }))
        .map_err(|err| anyhow!("failed to draw candlestick series: {err}"))?;

    chart
        .draw_series(entries.iter().enumerate().map(|(index, entry)| {
            let stats = accessor(entry);
            let label = label_formatter(entry, stats);
            EmptyElement::at((index as i32, stats.max))
                + Text::new(label, (0, -18), label_font.clone().color(&label_color))
        }))
        .map_err(|err| anyhow!("failed to draw candlestick labels: {err}"))?;

    Ok(())
}

fn compute_stats(values: &[f64]) -> Option<CandlestickStats> {
    if values.is_empty() {
        return None;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));

    let min = *sorted.first()?;
    let max = *sorted.last()?;

    Some(CandlestickStats {
        min,
        q1: percentile(&sorted, 0.25),
        median: percentile(&sorted, 0.5),
        q3: percentile(&sorted, 0.75),
        max,
    })
}

fn percentile(sorted: &[f64], percentile: f64) -> f64 {
    debug_assert!(!sorted.is_empty());
    let clamped = percentile.clamp(0.0, 1.0);
    if sorted.len() == 1 {
        return sorted[0];
    }

    let index = clamped * (sorted.len() - 1) as f64;
    let lower = index.floor() as usize;
    let upper = index.ceil() as usize;

    if lower == upper {
        sorted[lower]
    } else {
        let weight = index - lower as f64;
        sorted[lower] * (1.0 - weight) + sorted[upper] * weight
    }
}

fn linear_range(max: f64) -> std::ops::Range<f64> {
    let start = 0.0;
    let safe_max = max.max(0.0);
    let padding = (safe_max * 0.1).max(10.0);
    let end = (safe_max + padding).max(start + 1.0);
    start..end
}
