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
    /// Output directory, where the resulting latex and media files will be placed.
    #[arg(short, long)]
    output: PathBuf
}

fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    log::info!("Entered 'main'");

    let args = Args::parse();
    log::debug!("Input directory is {:?}", args.input);
    log::debug!("Output directory is {:?}", args.output);

    let mut output = args.output;
    if !output.exists() {
        log::info!("Creating directory {:?}", output);
        std::fs::create_dir(&output)?;
    }

    output.push("document.latex");
    log::info!("Creating file {:?}", output);
    let mut buf_writer = std::io::BufWriter::new(std::fs::File::create(&output)?);

    let mut input = args.input;
    input.push("word");
    input.push("_rels");
    input.push("document.xml.rels");

    log::debug!("Reading {:?}", &input);
    let mut parser = EventReader::new(std::io::BufReader::new(std::fs::File::open(&input)?));
    let rels = docx2latex::relationships(&mut parser)
        .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidData))?;

    input.pop();
    input.pop();
    input.push("document.xml");

    log::debug!("Reading {:?}", &input);
    let mut parser = EventReader::new(std::io::BufReader::new(std::fs::File::open(&input)?));

    let mut prysm = Prysm::new(rels);
    prysm.document(&mut parser, &mut buf_writer)?;

    log::info!("Exiting 'main'");

    Ok(())
}
