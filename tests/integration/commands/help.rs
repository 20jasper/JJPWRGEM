use rstest::rstest;

use crate::common::{cli, exec_cmd};

#[rstest]
#[case(vec![], "no_args")]
#[case(vec!["-h"], "short_help")]
#[case(vec!["--help"], "long_help")]
fn check_help_snapshot(#[case] args: Vec<&str>, #[case] name: &str) {
    let mut cmd = cli();
    cmd.args(&args);

    let output = exec_cmd(&mut cmd, None);

    crate::cli_snapshot!(name, output.snapshot_display());
}
