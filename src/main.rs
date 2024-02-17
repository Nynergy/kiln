use clap::Parser;

mod commands;
use commands::{
    list::list_tags,
    set::set_tags,
};

mod parse;

mod types;
use types::{
    args::{
        Commands,
        KilnArgs,
    },
};

fn main() {
    let args = KilnArgs::parse();

    let res = match args.command {
        Commands::List(args) => list_tags(args),
        Commands::Set(args) => set_tags(args),
    };

    if let Err(e) = res {
        eprintln!("{e}");
    }
}
