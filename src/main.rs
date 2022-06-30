use image::ImageError;
use thiserror::Error;

mod err;
mod opt;
mod sizes;

#[derive(Error, Debug)]
enum MainError {
    #[error("failed to load input: {0}")]
    FailedToLoadInput(ImageError),
}

fn main() -> Result<(), err::DisplayError> {
    let opt::Args {
        verbose,
        input,
        output,
        pixel_pitch,
    } = clap::Parser::parse();

    env_logger::Builder::new()
        .filter_level(match verbose {
            0 => log::LevelFilter::Warn,
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        })
        .init();

    let input_file = image::open(input).map_err(MainError::FailedToLoadInput)?;

    Ok(())
}
