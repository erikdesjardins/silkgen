use image::ImageError;
use std::fs::File;
use std::io;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use thiserror::Error;

mod analyze;
mod err;
mod generate;
mod opt;
mod sizes;

#[cfg(test)]
mod tests;

#[derive(Error, Debug)]
enum MainError {
    #[error("failed to load input file: {0}")]
    FailedToLoadInput(ImageError),
    #[error("input file name improperly formatted or no extension")]
    BadInputFileName,
    #[error("failed to open output file: {0}")]
    FailedToCreateOutput(io::Error),
    #[error("failed to write to output file: {0}")]
    FailedToWriteToOutput(io::Error),
}

fn main() -> Result<(), err::DisplayError> {
    let opt::Arguments {
        verbose,
        input,
        output,
        config,
    } = clap::Parser::parse();

    env_logger::Builder::new()
        .filter_level(match verbose {
            0 => log::LevelFilter::Warn,
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        })
        .init();

    let name = match input.file_stem() {
        Some(s) => s,
        None => Err(MainError::BadInputFileName)?,
    };

    let output = match output {
        Some(o) => o,
        None => {
            let mut out_dir = PathBuf::from(input.parent().unwrap_or_else(|| Path::new(".")));
            let mut out_name = name.to_owned();
            out_name.push(".kicad_mod");
            out_dir.push(out_name);
            out_dir
        }
    };

    let image = image::open(&input).map_err(MainError::FailedToLoadInput)?;

    let mut output_file = File::create(output).map_err(MainError::FailedToCreateOutput)?;

    generate::output_file(
        &name.to_string_lossy(),
        image,
        &config,
        BufWriter::new(&mut output_file),
    )
    .map_err(MainError::FailedToWriteToOutput)?;

    Ok(())
}
