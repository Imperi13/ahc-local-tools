mod config;

use anyhow::Result;
use clap::Parser;
use clap::Subcommand;

use config::load_config;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    #[clap(long = "config-file")]
    config_file: Option<PathBuf>,
    #[clap(subcommand)]
    subcommand: Subcommands,
}

#[derive(Subcommand)]
enum Subcommands {
    /// test all case in in_folder & output to out_folder
    TestAll {
        in_folder: PathBuf,
        out_folder: PathBuf,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    let config = load_config(args.config_file)?;

    println!("{:?}", config);

    match args.subcommand {
        Subcommands::TestAll {
            in_folder,
            out_folder,
        } => println!("{:?} {:?}", in_folder, out_folder),
    };

    Ok(())
}
