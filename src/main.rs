use brainfuck;
use std::io::prelude::*;
use structopt::StructOpt;

type Result<T> = std::result::Result<T, failure::Error>;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(
        short = "v",
        long = "Verbose",
        parse(from_occurrences),
        help = "verbose level"
    )]
    verbose: u64,
    #[structopt(
        short = "o",
        long = "optimize",
        help = "Optimization level (1-3)",
        conflicts_with = "jit"
    )]
    optimize: Option<u64>,
    #[structopt(
        short = "j",
        long = "jit",
        help = "Use JIT (Just-in-time) compilation (linux x86-64 only)"
    )]
    jit: bool,
    program: String,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    loggerv::init_with_verbosity(opt.verbose).unwrap();
    let mut f = std::fs::File::open(opt.program)?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;
    let stdin = std::io::stdin();
    let stdin = stdin.lock();
    let stdout = std::io::stdout();
    brainfuck::run(&buffer, stdin, stdout, opt.optimize, opt.jit)
}
