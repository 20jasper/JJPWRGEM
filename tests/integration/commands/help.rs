use crate::common::cli;
use insta::assert_snapshot;
use rstest::rstest;

#[rstest]
#[case(&[] as &[&str], "no_args", false)]
#[case(&["-h"], "short", true)]
#[case(&["help"], "help_subcommand", true)]
#[case(&["--help"], "long", true)]
fn help_snapshot(#[case] args: &[&str], #[case] snapshot_name: &str, #[case] success: bool) {
    let mut cmd = cli();
    cmd.args(args);

    let output = cmd.output().expect("failed to capture help output");
    assert_eq!(output.status.success(), success);

    let stdout = String::from_utf8(output.stdout).expect("help text should be valid UTF-8");
    assert_snapshot!(snapshot_name, stdout);
}
