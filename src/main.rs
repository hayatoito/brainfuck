use brainfuck;
use std::io::prelude::*;
use structopt::StructOpt;

type Result<T> = std::result::Result<T, failure::Error>;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short = "v", parse(from_occurrences))]
    verbose: u64,
    #[structopt(short = "i", long = "inteerpreter")]
    interpreter: Option<u64>,
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
    let stdout = stdout.lock();
    if let Some(i) = opt.interpreter {
        if i == 1 {
            brainfuck::run1(&buffer, stdin, stdout)?;
        } else if i == 2 {
            brainfuck::run2(&buffer, stdin, stdout)?;
        } else if i == 3 {
            brainfuck::run3(&buffer, stdin, stdout)?;
        } else if i == 4 {
            brainfuck::run_jit1(&buffer, stdin, stdout)?;
        } else {
            unreachable!()
        }
    } else {
        brainfuck::run(&buffer, stdin, stdout)?;
    }
    Ok(())
}
