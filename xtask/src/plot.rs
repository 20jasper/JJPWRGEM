use std::{
    cmp::Ordering,
    ops::Range,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow, bail};
use jjpwrgem_parse::{Array, Node, Object, SyntaxOptions, ast::Document};
use plotters::{
    coord::ranged1d::{DefaultFormatting, KeyPointHint, KeyPointWeight},
    element::DashedPathElement,
    prelude::*,
    style::text_anchor::{HPos, Pos, VPos},
};

mod config;

use config::{
    AXIS_HEADROOM, BASE, BENCHMARK_DATASETS, BOX_HEIGHT_PX, CAP_PX, CHART_HEIGHT, CHART_MARGIN,
    CHART_WIDTH, EXCLUDED_COMMANDS, GRID, HIGHLIGHT, HIGHLIGHT_COMMAND, LABEL_AREA_BOTTOM_PX,
    LABEL_AREA_LEFT_PX, MIN_TICK_SPACING_PX, MINOR_GRID, MINORS_PER_GAP, PLOT_WIDTH_PX, TEXT,
    TICK_CANDIDATES_MS,
};

struct BenchmarkResult {
    command: String,
    times: Vec<f64>,
}

type DocNode<'a> = Node<'a, &'a str>;

fn field<'a>(object: Object<'a, &'a str>, name: &str) -> Result<DocNode<'a>> {
    object
        .get(name)
        .ok_or_else(|| anyhow!("missing '{name}' field"))
}

fn as_object<'a>(node: DocNode<'a>, what: &str) -> Result<Object<'a, &'a str>> {
    node.as_object()
        .ok_or_else(|| anyhow!("{what} must be an object"))
}

fn as_array<'a>(node: DocNode<'a>, what: &str) -> Result<Array<'a, &'a str>> {
    node.as_array()
        .ok_or_else(|| anyhow!("{what} must be an array"))
}

fn as_f64(node: DocNode<'_>) -> Option<f64> {
    let number = node.as_number()?;
    let text = match number.exponent() {
        Some(exponent) => format!("{}e{exponent}", number.mantissa()),
        None => number.mantissa().to_owned(),
    };
    Some(text.parse().expect("valid json number text parses as f64"))
}

fn parse_result(item: DocNode<'_>) -> Result<BenchmarkResult> {
    let object = as_object(item, "result")?;

    let command = field(object, "command")?
        .as_str()
        .ok_or_else(|| anyhow!("'command' must be a string"))?
        .raw();
    // `name` holds the `--command-name` label, `command` the full command line
    let command = object
        .get("name")
        .and_then(DocNode::as_str)
        .map_or(command, |name| name.raw())
        .to_owned();

    let times = as_array(field(object, "measurements")?, "'measurements'")?
        .iter()
        .enumerate()
        .map(|(index, measurement)| {
            let what = format!("measurements[{index}]");
            let fields = as_object(measurement, &what)?;
            let wall_clock = as_object(
                field(fields, "time_wall_clock").with_context(|| what.clone())?,
                &format!("{what}.time_wall_clock"),
            )?;
            as_f64(field(wall_clock, "value").with_context(|| what.clone())?)
                .ok_or_else(|| anyhow!("{what}.time_wall_clock.value must be a number"))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(BenchmarkResult { command, times })
}

fn parse_benchmark_results(raw: &str) -> Result<Vec<BenchmarkResult>> {
    let doc = Document::parse(raw, SyntaxOptions::JSON)
        .map_err(|err| anyhow!("failed to parse JSON with jjpwrgem parser: {err}"))?;
    let root = as_object(doc.root(), "benchmark data")?;

    as_array(field(root, "results")?, "'results'")?
        .iter()
        .enumerate()
        .map(|(index, item)| parse_result(item).with_context(|| format!("index: {index}")))
        .collect()
}

struct Series {
    name: String,
    min: f64,
    q1: f64,
    median: f64,
    q3: f64,
    max: f64,
}

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
    for result in results
        .into_iter()
        .filter(|result| !EXCLUDED_COMMANDS.contains(&result.command.as_str()))
    {
        let seconds_to_ms: Vec<f64> = result.times.iter().map(|time| time * 1_000.0).collect();
        let Some(series) = compute_stats(result.command.clone(), &seconds_to_ms) else {
            bail!("benchmark '{}' is missing time samples", result.command);
        };
        entries.push(series);
    }

    if entries.is_empty() {
        bail!("no benchmark entries found");
    }

    entries.sort_by(|left, right| {
        left.median
            .partial_cmp(&right.median)
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

type RowKey = SegmentValue<usize>;

struct SqrtRange {
    end_ms: f64,
}

impl SqrtRange {
    fn covering(max_ms: f64) -> Self {
        Self {
            end_ms: (max_ms * AXIS_HEADROOM * AXIS_HEADROOM).max(1.0),
        }
    }
}

impl Ranged for SqrtRange {
    type FormatOption = DefaultFormatting;
    type ValueType = f64;

    fn map(&self, value: &f64, limit: (i32, i32)) -> i32 {
        let fraction = value.max(0.0).sqrt() / self.end_ms.sqrt();
        limit.0 + (fraction * f64::from(limit.1 - limit.0)).round() as i32
    }

    fn key_points<Hint: KeyPointHint>(&self, hint: Hint) -> Vec<f64> {
        let majors = major_ticks(self.end_ms);
        match hint.weight() {
            KeyPointWeight::Bold => majors,
            KeyPointWeight::Any => {
                let minors = minor_ticks(&majors, self.end_ms);
                majors.into_iter().chain(minors).collect()
            }
        }
    }

    fn range(&self) -> Range<f64> {
        0.0..self.end_ms
    }
}

fn draw_candlesticks(
    area: &DrawingArea<SVGBackend, plotters::coord::Shift>,
    entries: &[Series],
) -> Result<()> {
    let font = ("sans-serif", 20).into_font();
    let max = entries.iter().map(|entry| entry.max).fold(0.0, f64::max);
    let rows = entries.len();

    let mut chart = ChartBuilder::on(area)
        .margin(CHART_MARGIN)
        .set_label_area_size(LabelAreaPosition::Left, LABEL_AREA_LEFT_PX)
        .set_label_area_size(LabelAreaPosition::Bottom, LABEL_AREA_BOTTOM_PX)
        .build_cartesian_2d(SqrtRange::covering(max), (0..rows - 1).into_segmented())
        .context("failed to build chart")?;

    chart
        .configure_mesh()
        .bold_line_style(ShapeStyle::from(GRID).stroke_width(1))
        .light_line_style(ShapeStyle::from(MINOR_GRID).stroke_width(1))
        .x_desc("Milliseconds (square-root scale)")
        .y_desc("Command")
        .axis_desc_style(font.clone())
        .x_label_style(font.clone().color(&TEXT))
        .x_label_formatter(&|ms| format!("{ms:.0}"))
        .disable_y_mesh()
        .disable_y_axis()
        .draw()
        .context("failed to draw axes")?;

    if let Some(jjp) = entries.iter().find(|entry| entry.name == HIGHLIGHT_COMMAND) {
        let median = jjp.median;
        chart
            .draw_series(std::iter::once(DashedPathElement::new(
                [(median, RowKey::Exact(0)), (median, RowKey::Last)],
                6,
                4,
                ShapeStyle::from(HIGHLIGHT.mix(0.8)).stroke_width(1),
            )))
            .context("failed to draw reference line")?;
    }

    for (row, entry) in entries.iter().rev().enumerate() {
        let Series {
            min,
            q1,
            median,
            q3,
            max,
            ..
        } = *entry;
        let highlighted = entry.name == HIGHLIGHT_COMMAND;
        let color = if highlighted { HIGHLIGHT } else { BASE };
        let y = RowKey::CenterOf(row);
        let whisker_style = ShapeStyle::from(color.mix(0.7)).stroke_width(2);
        let half_box = BOX_HEIGHT_PX / 2;

        chart
            .draw_series([
                PathElement::new(vec![(min, y.clone()), (max, y.clone())], whisker_style),
                PathElement::new(
                    vec![(q1, y.clone()), (q3, y.clone())],
                    ShapeStyle::from(color).stroke_width(BOX_HEIGHT_PX),
                ),
            ])
            .context("failed to draw whiskers and box")?;

        chart
            .draw_series([min, max].map(|x| {
                EmptyElement::at((x, y.clone()))
                    + PathElement::new(vec![(0, -CAP_PX), (0, CAP_PX)], whisker_style)
            }))
            .context("failed to draw whisker caps")?;

        chart
            .draw_series([EmptyElement::at((median, y.clone()))
                + PathElement::new(
                    vec![(0, -(half_box as i32)), (0, half_box as i32)],
                    ShapeStyle::from(WHITE).stroke_width(2),
                )])
            .context("failed to draw median marker")?;

        let median_label = font
            .clone()
            .color(&TEXT)
            .pos(Pos::new(HPos::Left, VPos::Center));
        let name_label = font
            .clone()
            .style(if highlighted {
                FontStyle::Bold
            } else {
                FontStyle::Normal
            })
            .color(if highlighted { &HIGHLIGHT } else { &TEXT })
            .pos(Pos::new(HPos::Right, VPos::Center));
        chart
            .draw_series([
                EmptyElement::at((max.max(q3), y.clone()))
                    + Text::new(format!("{median:.1} ms"), (10, 0), median_label),
                EmptyElement::at((0.0, y)) + Text::new(entry.name.clone(), (-10, 0), name_label),
            ])
            .context("failed to draw labels")?;
    }

    Ok(())
}

fn compute_stats(name: String, values: &[f64]) -> Option<Series> {
    if values.is_empty() {
        return None;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));

    let min = *sorted.first()?;
    let max = *sorted.last()?;

    Some(Series {
        name,
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

fn major_ticks(end_ms: f64) -> Vec<f64> {
    let mut kept: Vec<f64> = Vec::new();
    for ms in TICK_CANDIDATES_MS
        .into_iter()
        .take_while(|ms| *ms <= end_ms)
    {
        let far_enough = kept.last().is_none_or(|last| {
            (ms.sqrt() - last.sqrt()) / end_ms.sqrt() * PLOT_WIDTH_PX >= MIN_TICK_SPACING_PX
        });
        if far_enough {
            kept.push(ms);
        }
    }
    kept
}

fn minor_ticks(majors: &[f64], end_ms: f64) -> Vec<f64> {
    let next_candidate = majors
        .last()
        .and_then(|last| TICK_CANDIDATES_MS.into_iter().find(|ms| ms > last));
    let bounds: Vec<f64> = majors.iter().copied().chain(next_candidate).collect();
    bounds
        .windows(2)
        .flat_map(|pair| {
            let step = (pair[1] - pair[0]) / MINORS_PER_GAP as f64;
            (1..MINORS_PER_GAP).map(move |index| pair[0] + step * index as f64)
        })
        .filter(|ms| *ms <= end_ms)
        .collect()
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
    fn minor_ticks_split_each_gap_into_three() {
        let minors = minor_ticks(&[100.0, 250.0], 625.0);

        let expected = [150.0, 200.0, 1000.0 / 3.0, 1250.0 / 3.0];
        assert_eq!(minors.len(), expected.len());
        for (actual, expected) in minors.iter().zip(expected) {
            assert!((actual - expected).abs() < 1e-9);
        }
    }

    #[test]
    fn major_ticks_stay_within_range() {
        assert_eq!(
            major_ticks(625.0),
            [0.0, 5.0, 25.0, 50.0, 100.0, 250.0, 500.0]
        );
    }

    #[test]
    fn sqrt_range_maps_midpoint_of_root_axis() {
        let range = SqrtRange { end_ms: 100.0 };

        assert_eq!(range.map(&25.0, (0, 200)), 100);
        assert_eq!(range.map(&100.0, (0, 200)), 200);
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
