use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};

use anyhow::{Context, Error, Result, anyhow, bail};
use jjpwrgem_parse::ast::{self, Document, Value};
use plotters::prelude::*;

const CHART_WIDTH: u32 = 1280;
const CHART_HEIGHT: u32 = 720;

struct BenchmarkResult {
    command: String,
    times: Vec<f64>,
}

impl TryFrom<(&Document<&str>, &Value)> for BenchmarkResult {
    type Error = Error;

    fn try_from((doc, item): (&Document<&str>, &Value)) -> std::result::Result<Self, Self::Error> {
        let ast::Value::Object(_, entry) = item else {
            bail!("must be a JSON object");
        };

        let command_value = doc
            .get_object_value(entry, "command")
            .with_context(|| "missing 'command' field")?;
        let command = doc
            .as_str(command_value)
            .with_context(|| "command must be a string")?;
        // `name` holds the `--command-name` label, `command` the full command line
        let command = doc
            .get_object_value(entry, "name")
            .and_then(|name| doc.as_str(name))
            .unwrap_or(command)
            .to_owned();

        let measurements_value = doc
            .get_object_value(entry, "measurements")
            .with_context(|| "missing 'measurements' field")?;
        let ast::Value::Array(_, measurements) = measurements_value else {
            bail!("'measurements' must be an array");
        };

        let times = measurements
            .iter()
            .enumerate()
            .map(|(index, measurement)| -> Result<f64> {
                let ast::Value::Object(_, fields) = measurement else {
                    bail!("measurements[{index}] must be an object");
                };
                let wall_clock = doc
                    .get_object_value(fields, "time_wall_clock")
                    .with_context(|| format!("measurements[{index}] missing 'time_wall_clock'"))?;
                let ast::Value::Object(_, wall_clock_fields) = wall_clock else {
                    bail!("measurements[{index}].time_wall_clock must be an object");
                };
                let value = doc
                    .get_object_value(wall_clock_fields, "value")
                    .with_context(|| {
                        format!("measurements[{index}].time_wall_clock missing 'value'")
                    })?;
                doc.parse_f64(value).with_context(|| {
                    format!("measurements[{index}].time_wall_clock.value must be a number")
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(BenchmarkResult { command, times })
    }
}

fn parse_benchmark_results(raw: &str) -> Result<Vec<BenchmarkResult>> {
    let doc = Document::parse(raw)
        .map_err(|err| anyhow!("failed to parse JSON with jjpwrgem parser: {err}"))?;
    let ast::Value::Object(_, entries) = doc.root() else {
        bail!("benchmark data must be a JSON object");
    };

    let results_value = doc
        .get_object_value(entries, "results")
        .context("benchmark data missing 'results' field")?;
    let ast::Value::Array(_, items) = results_value else {
        bail!("'results' field must be an array");
    };

    items
        .iter()
        .enumerate()
        .map(|(index, item)| -> Result<BenchmarkResult> {
            (&doc, item)
                .try_into()
                .with_context(|| format!("index: {index}"))
        })
        .collect()
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
        let action = match self.action {
            BenchmarkAction::Pretty => "pretty print",
            BenchmarkAction::Minify => "minify",
        };
        format!(
            "JSON {action} {} Benchmark - Execution Time",
            self.info.display_name
        )
    }

    fn subtitle(&self) -> String {
        format!("Parsing and stringifying a {}", self.info.description)
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
    PathBuf::from(repo_path!("benches/docker/output"))
}

pub fn plot_all_benchmarks() -> Result<()> {
    let output_dir = bench_output_dir();
    let mut failures: Vec<(&'static str, anyhow::Error)> = Vec::new();

    for dataset in BENCHMARK_DATASETS {
        let input = output_dir.join(format!("{}.json", dataset.stem));
        let output = output_dir.join(format!("{}.svg", dataset.stem));
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

pub fn plot_benchmark_candlesticks(
    input: &Path,
    output: &Path,
    title: &str,
    subtitle: &str,
) -> Result<()> {
    let raw = std::fs::read_to_string(input)
        .with_context(|| format!("failed to read input file {}", input.display()))?;
    let results = parse_benchmark_results(&raw)
        .with_context(|| format!("failed to parse benchmark data in {}", input.display()))?;

    let mut entries = Vec::new();
    for result in results {
        let seconds_to_ms: Vec<f64> = result.times.iter().map(|time| time * 1_000.0).collect();
        let Some(time) = compute_stats(&seconds_to_ms) else {
            bail!("benchmark '{}' is missing time samples", result.command);
        };
        entries.push(SeriesEntry {
            name: result.command,
            time,
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

    let root = SVGBackend::new(output, (CHART_WIDTH, CHART_HEIGHT)).into_drawing_area();
    root.fill(&WHITE).context("failed to clear drawing area")?;
    let chart_area = root
        .titled(title, ("sans-serif", 28).into_font())
        .context("failed to render chart title")?
        .titled(subtitle, ("sans-serif", 22).into_font())
        .context("failed to render chart subtitle")?;

    draw_candlesticks(&chart_area, &entries)?;

    root.present().context("failed to write chart image")?;
    Ok(())
}

fn draw_candlesticks(
    area: &DrawingArea<SVGBackend, plotters::coord::Shift>,
    entries: &[SeriesEntry],
) -> Result<()> {
    let color = RGBColor(33, 150, 243);
    let grid_color = RGBColor(210, 210, 210);
    let label_color = RGBColor(30, 30, 30);
    let font = ("sans-serif", 20).into_font();
    let max = entries
        .iter()
        .map(|entry| entry.time.max)
        .fold(f64::NEG_INFINITY, f64::max);

    let mut chart = ChartBuilder::on(area)
        .margin(25)
        .set_label_area_size(LabelAreaPosition::Left, 70)
        .set_label_area_size(LabelAreaPosition::Bottom, 90)
        .build_cartesian_2d(0..entries.len() as i32, linear_range(max))
        .context("failed to build candlestick chart")?;

    chart
        .configure_mesh()
        .light_line_style(ShapeStyle::from(grid_color).stroke_width(1))
        .bold_line_style(ShapeStyle::from(grid_color).stroke_width(2))
        .x_desc("Command")
        .y_desc("Milliseconds")
        .axis_desc_style(font.clone())
        .label_style(font.clone())
        .x_labels(entries.len())
        .x_label_formatter(&|value| {
            usize::try_from(*value)
                .ok()
                .and_then(|index| entries.get(index))
                .map(|entry| entry.name.clone())
                .unwrap_or_default()
        })
        .y_labels(8)
        .y_label_formatter(&|value| format!("{value:.0}"))
        .draw()
        .context("failed to configure chart mesh")?;

    let candle_style = ShapeStyle::from(color).filled().stroke_width(3);
    let whisker_style = ShapeStyle::from(color.mix(0.7)).stroke_width(10);

    chart
        .draw_series(entries.iter().enumerate().map(|(index, entry)| {
            let stats = entry.time;
            CandleStick::new(
                index as i32,
                stats.q1,
                stats.max,
                stats.min,
                stats.q3,
                candle_style,
                whisker_style,
                28,
            )
        }))
        .context("failed to draw candlestick series")?;

    chart
        .draw_series(entries.iter().enumerate().map(|(index, entry)| {
            EmptyElement::at((index as i32, entry.time.max))
                + Text::new(
                    format!("{:.1} ms", entry.time.median),
                    (0, -18),
                    font.clone().color(&label_color),
                )
        }))
        .context("failed to draw candlestick labels")?;

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

#[cfg(test)]
mod tests {
    use super::*;

    // trimmed from `hyperfine 2.0.0 --export-json`
    const HYPERFINE_V2: &str = r#"{
  "schema_version": 2,
  "primary_metric": "time_wall_clock",
  "results": [
    {
      "command": "jjp format < data.json",
      "name": "jjp",
      "measurements": [
        { "time_wall_clock": { "value": 0.005, "unit": "second" }, "exit_code": 0 },
        { "time_wall_clock": { "value": 0.006, "unit": "second" }, "exit_code": 0 }
      ]
    },
    {
      "command": "jq < data.json",
      "measurements": [
        { "time_wall_clock": { "value": 0.017, "unit": "second" }, "exit_code": 0 },
        { "time_wall_clock": { "value": 0.018, "unit": "second" }, "exit_code": 0 }
      ]
    }
  ]
}"#;

    #[test]
    fn parses_hyperfine_v2_results() {
        let results = parse_benchmark_results(HYPERFINE_V2).unwrap();

        let summary: Vec<_> = results
            .iter()
            .map(|result| (result.command.as_str(), result.times.as_slice()))
            .collect();
        assert_eq!(
            summary,
            [
                ("jjp", [0.005, 0.006].as_slice()),
                ("jq < data.json", [0.017, 0.018].as_slice()),
            ]
        );
    }

    #[test]
    fn rejects_hyperfine_v1_results() {
        let v1 = r#"{ "results": [{ "command": "jjp", "times": [0.005] }] }"#;

        assert!(parse_benchmark_results(v1).is_err());
    }

    #[test]
    fn plots_hyperfine_v2_results() {
        let dir = std::env::temp_dir().join(format!("xtask-plot-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let input = dir.join("bench.json");
        let output = dir.join("bench.svg");
        std::fs::write(&input, HYPERFINE_V2).unwrap();

        plot_benchmark_candlesticks(&input, &output, "title", "subtitle").unwrap();

        assert!(std::fs::metadata(&output).unwrap().len() > 0);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
