use plotters::prelude::RGBColor;

pub(super) const CHART_WIDTH: u32 = 1280;
pub(super) const CHART_HEIGHT: u32 = 720;
pub(super) const CHART_MARGIN: u32 = 25;
pub(super) const LABEL_AREA_LEFT_PX: u32 = 130;
pub(super) const LABEL_AREA_BOTTOM_PX: u32 = 100;
pub(super) const TICK_CANDIDATES_MS: [f64; 10] = [
    0.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0,
];
pub(super) const PLOT_WIDTH_PX: f64 = 1100.0;
pub(super) const MIN_TICK_SPACING_PX: f64 = 80.0;
pub(super) const MINORS_PER_GAP: usize = 3;
pub(super) const AXIS_HEADROOM: f64 = 1.15;
pub(super) const BOX_HEIGHT_PX: u32 = 24;
pub(super) const CAP_PX: i32 = 10;
pub(super) const BASE: RGBColor = RGBColor(33, 150, 243);
pub(super) const HIGHLIGHT: RGBColor = RGBColor(255, 112, 0);
pub(super) const GRID: RGBColor = RGBColor(210, 210, 210);
pub(super) const MINOR_GRID: RGBColor = RGBColor(235, 235, 235);
pub(super) const TEXT: RGBColor = RGBColor(30, 30, 30);
pub(super) const HIGHLIGHT_COMMAND: &str = "jjp";
pub(super) const EXCLUDED_COMMANDS: [&str; 1] = ["sjq"];

pub(super) struct BenchmarkDataset {
    pub(super) stem: &'static str,
    pub(super) action: BenchmarkAction,
    pub(super) info: &'static DatasetInfo,
}

pub(super) struct DatasetInfo {
    display_name: &'static str,
    description: &'static str,
}

#[derive(Clone, Copy)]
pub(super) enum BenchmarkAction {
    Pretty,
    Minify,
}

impl BenchmarkDataset {
    pub(super) fn title(&self) -> String {
        let action = match self.action {
            BenchmarkAction::Pretty => "pretty print",
            BenchmarkAction::Minify => "minify",
        };
        format!(
            "JSON {action} {} Benchmark - Execution Time",
            self.info.display_name
        )
    }

    pub(super) fn subtitle(&self) -> String {
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

pub(super) const BENCHMARK_DATASETS: &[BenchmarkDataset] = &[
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
