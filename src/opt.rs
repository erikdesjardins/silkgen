use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Args {
    #[clap(
        short = 'v',
        long = "verbose",
        parse(from_occurrences),
        global = true,
        help = "Logging verbosity (-v info, -vv debug, -vvv trace)"
    )]
    pub verbose: u8,
}
