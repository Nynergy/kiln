use clap::{
    Args,
    Parser,
    Subcommand,
};
use std::path::PathBuf;

use crate::types::id3::TagId;

#[derive(Parser)]
#[command(name = "kiln")]
#[command(author = "Ben Buchanan")]
#[command(version = "0.1.0")]
#[command(about = "An id3 tag utility for the command line")]
pub struct KilnArgs {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List tags for all selected files
    List(ListArgs),

    /// Set tags given an input file
    Set(SetArgs),
}

#[derive(Args)]
pub struct ListArgs {
    /// Glob string to select files/directories
    #[arg(default_value_t = String::from("./*"))]
    pub glob: String,

    /// Turn off comments in the output
    #[arg(short = 'c', long)]
    pub no_comments: bool,

    /// Force listing files with no tags
    #[arg(short, long)]
    pub force_empty: bool,
}

#[derive(Args)]
pub struct SetArgs {
    /// Input file to read tags from
    pub input_file: PathBuf,

    /// Ask for user confirmation before writing tags to files
    #[arg(short, long)]
    pub ask: bool,

    /// Specify a list of tags to preserve (will not be deleted)
    #[arg(short, long = "preserve", use_value_delimiter = true, value_delimiter = ',')]
    pub preserved_tags: Vec<TagId>,
}
