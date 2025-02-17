use clap::Parser;
use docx2latex::Element;
use std::fs;
use std::io;
use std::path::PathBuf;

use xml::reader::{EventReader, XmlEvent};

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

    let _root = Element::read(&input)?;

    log::info!("Exiting 'main'");

    Ok(())
}
