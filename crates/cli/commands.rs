use clap::{Parser, Subcommand};

use crate::{
    docs::{indent, strip_front_matter},
    get_docs_snapshot,
};

#[derive(Parser)]
#[command(
    version = concat!(
        env!("CARGO_PKG_VERSION"), "\n", 
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/axolotl.txt")), "\n",
        "an axolotl riding a skateboard"
    ),
    about,
    disable_help_subcommand = true,
    help_expected = true,
    after_help = format!(
        "jjpwrgem is a tool for formatting and validating json inputs\n\nExamples:\n{}\n\n{}\n\nRun jjp <COMMAND> --help for information about specific commands",
        indent(strip_front_matter(get_docs_snapshot!("format_pretty"))),
        indent(strip_front_matter(get_docs_snapshot!("check_failure"))), 
    )
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Make your json look really good
    #[command(after_help = format!(
        "Examples:\n{}\n\n{}",
        indent(strip_front_matter(get_docs_snapshot!("format_pretty"))),
        indent(strip_front_matter(get_docs_snapshot!("format_uglify"))),
    ))]
    Format {
        /// Removes all insignificant whitespace instead of pretty printing,
        /// also known as minifying. Cannot be combined with --preferred-width
        #[arg(short, long, conflicts_with = "preferred_width")]
        uglify: bool,

        /// Preferred maximum line width. Note this is not a hard maximum width
        #[arg(long, default_value_t = 80, conflicts_with = "uglify")]
        preferred_width: usize,
    },
    #[command(after_help = format!(
        "Examples:\n{}\n\n{}",
        indent(strip_front_matter(get_docs_snapshot!("check_success"))),
        indent(strip_front_matter(get_docs_snapshot!("check_failure"))),
    ))]
    /// Validates json syntax
    Check,
}
