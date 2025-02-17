use clap::Parser;
use docx2latex::*;
use std::{io::Write, path::PathBuf};

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
    output: PathBuf,
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

    let mut input = args.input;

    input.push("word");
    input.push("media");
    let media_present;
    if input.exists() {
        log::info!("Found a media folder at {:?}", &input);
        media_present = true;
        output.push("media");
        if !output.exists() {
            log::info!("Creating directory {:?}", output);
            std::fs::create_dir(&output)?;
        }
        for entry in std::fs::read_dir(&input)? {
            if let Ok(file) = &entry {
                output.push(file.file_name());
                std::fs::copy(file.path(), &output)?;
                log::info!("Copied media file {:?}", file.file_name());
                output.pop();
            } else {
                log::error!("DirEntry Error: {:?}", &entry);
            }
        }
        output.pop();
    } else {
        log::info!("Did not find media folder at {:?}", &input);
        media_present = false;
    }
    input.pop();

    output.push("document.latex");
    log::info!("Creating file {:?}", output);
    let mut buf_writer = std::io::BufWriter::new(std::fs::File::create(&output)?);

    writeln!(&mut buf_writer, "\\documentclass{{article}}")?;
    writeln!(&mut buf_writer, "\\usepackage[T2A]{{fontenc}}")?;
    writeln!(&mut buf_writer, "\\usepackage[utf8]{{inputenc}}")?;
    writeln!(&mut buf_writer, "\\usepackage[fontsize=16pt]{{fontsize}}")?;
    writeln!(&mut buf_writer, "\\usepackage[left=2cm,right=2cm,bottom=2cm]{{geometry}}")?;
    writeln!(&mut buf_writer, "\\usepackage[english,ukrainian]{{babel}}")?;
    writeln!(&mut buf_writer, "\\usepackage{{amsmath}}")?;
    writeln!(&mut buf_writer, "\\usepackage{{amssymb}}")?;
    writeln!(&mut buf_writer, "\\usepackage{{dsfont}}")?;
    writeln!(&mut buf_writer, "\\usepackage{{hyperref}}")?;

    if media_present {
        writeln!(&mut buf_writer, "\\usepackage{{graphicx}}")?;
        writeln!(&mut buf_writer, "\\graphicspath{{ {{./media/}} }}")?;
    }

    writeln!(&mut buf_writer)?;
    writeln!(&mut buf_writer, "\\begin{{document}}")?;
    writeln!(&mut buf_writer)?;


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

    writeln!(&mut buf_writer, "\\end{{document}}")?;

    log::info!("Exiting 'main'");

    Ok(())
}
