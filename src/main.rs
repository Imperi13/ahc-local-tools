use clap::Parser;
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
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

fn main() {
    let args = Args::parse();
    match args.subcommand {
        Subcommands::TestAll {
            in_folder,
            out_folder,
        } => println!("{:?} {:?}", in_folder, out_folder),
    }
}
