use std::path::PathBuf;
use clap::Parser;

/// A command line utility to convert docx files into latex templates.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input docx file to be processed.
    #[arg(short, long)]
    input: PathBuf,
    /// Output directory, where the resulting latex and media files will be placed.
    #[arg(short, long)]
    output: PathBuf
}

fn main() {
    pretty_env_logger::init();

    log::info!("Entered 'main'");
    let args = Args::parse();

    println!("{:?}", args.input);
    println!("{:?}", args.output);

    log::info!("Exiting 'main'");
}
