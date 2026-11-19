use rstest::rstest;

use crate::common::assert_cli_snapshot;

#[rstest]
#[case(vec![], "no_args", false)]
#[case(vec!["-h"], "short_help", true)]
#[case(vec!["--help"], "long_help", true)]
fn check_help_snapshot(#[case] args: Vec<&str>, #[case] name: &str, #[case] success: bool) {
    assert_cli_snapshot!(name, &args, None, success);
}
