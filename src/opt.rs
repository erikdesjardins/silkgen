use crate::sizes::KicadDim;
use clap::{ArgAction, Args, Parser};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Arguments {
    /// Logging verbosity (-v info, -vv debug, -vvv trace)
    #[clap(short = 'v', long = "verbose", action = ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Input PNG file to be converted
    pub input: PathBuf,

    /// Output file (defaults to <input>.kicad_mod)
    #[clap(short = 'o', long = "output")]
    pub output: Option<PathBuf>,

    #[clap(flatten)]
    pub config: Config,
}

#[derive(Args, Debug)]
pub struct Config {
    /// Size of one pixel in the output footprint (e.g. 1mm or 0.05in)
    #[clap(short = 'p', long = "pixel-pitch")]
    pub pixel_pitch: KicadDim,

    /// Gap between silkscreen layers and copper layers (e.g. 0.1mm or 0.005in)
    ///
    /// This must be nonzero to avoid DRC violations.
    #[clap(short = 'c', long = "clearance")]
    pub clearance: KicadDim,

    /// Invert the image, treating dark pixels as silkscreen and light pixels as copper
    #[clap(long = "invert")]
    pub invert: bool,
}
