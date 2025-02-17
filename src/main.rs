use clap::Parser;
use docx2latex::*;
use std::path::PathBuf;

use xml::reader::EventReader;

/// A command line utility to convert docx files into latex templates.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input directory containing Office Open XML package obtained by unzipping target `.docx` file.
    /// User is tasked with unzipping the file manually to provide finer control over the filesystem.
    #[arg(short, long)]
    input: PathBuf,
    // Output directory, where the resulting latex and media files will be placed.
    // #[arg(short, long)]
    // output: PathBuf
}

fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    log::info!("Entered 'main'");

    let args = Args::parse();
    log::debug!("Input directory is {:?}", args.input);
    // log::debug!("Output directory is {:?}", args.output);

    let mut input = args.input;

    input.push("word");
    input.push("document.xml");

    log::debug!("Reading {:?}", &input);

    // let _root = Element::read(&input)?;

    let mut prysm = Prysm::default();
    let mut parser = EventReader::new(std::io::BufReader::new(std::fs::File::open(&input)?));

    prysm.stream(&mut parser);

    log::info!("Exiting 'main'");

    Ok(())
}
