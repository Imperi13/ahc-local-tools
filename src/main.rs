mod config;
mod test_exec;

use anyhow::Result;
use clap::Parser;
use clap::Subcommand;

use config::load_config;
use std::path::PathBuf;
use test_exec::test_all;

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
        #[clap(long = "best-result-file")]
        best_result_file: Option<PathBuf>,
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
            best_result_file,
        } => test_all(&config.exec_config, in_folder, out_folder, best_result_file)?,
    };

    Ok(())
}
