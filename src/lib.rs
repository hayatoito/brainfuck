// https://eli.thegreenplace.net/2017/adventures-in-jit-compilation-part-1-an-interpreter.html
// https://esolangs.org/wiki/Brainfuck
use failure;
use std::io::Read;
use std::io::Write;

type Result<T> = std::result::Result<T, failure::Error>;

struct Program {
    instructions: Vec<u8>,
}

impl Program {
    fn new(s: &[u8]) -> Program {
        Program {
            instructions: s
                .into_iter()
                .filter(|c| match c {
                    b'>' | b'<' | b'+' | b'-' | b'.' | b',' | b'[' | b']' => true,
                    _ => false,
                })
                .cloned()
                .collect(),
        }
    }

    fn create_jumptable(&self) -> Vec<usize> {
        let mut pc = 0;
        let program_size = self.instructions.len();

        let mut jumptable = vec![0; program_size];

        while pc < program_size {
            if self.instructions[pc] == b'[' {
                let mut bracket_nesting = 1;
                let mut seek = pc;
                while bracket_nesting > 0 && seek + 1 < program_size {
                    seek += 1;
                    match self.instructions[seek] {
                        b']' => bracket_nesting -= 1,
                        b'[' => bracket_nesting += 1,
                        _ => (),
                    }
                }
                assert_eq!(bracket_nesting, 0);
                jumptable[pc] = seek;
                jumptable[seek] = pc;
            }
            pc += 1;
        }
        jumptable
    }

    fn run(&self, read: impl Read, mut write: impl Write) -> Result<()> {
        let mut memory: Vec<u8> = vec![0; 30000];
        let jumptable = self.create_jumptable();

        let mut pc = 0;
        let mut data_ptr: usize = 0;

        let mut input = read.bytes();

        while pc < self.instructions.len() {
            match self.instructions[pc] {
                b'>' => data_ptr += 1,
                b'<' => data_ptr -= 1,
                b'+' => memory[data_ptr] = memory[data_ptr].wrapping_add(1),
                b'-' => memory[data_ptr] = memory[data_ptr].wrapping_sub(1),
                b'.' => {
                    write.write_all(&memory[data_ptr..(data_ptr + 1)])?;
                    write.flush().unwrap();
                }
                b',' => {
                    memory[data_ptr] = input
                        .next()
                        .ok_or_else(|| failure::err_msg("input unavailable"))?
                        .expect("read error");
                }
                b'[' => {
                    if memory[data_ptr] == 0 {
                        pc = jumptable[pc];
                    }
                }
                b']' => {
                    if memory[data_ptr] != 0 {
                        pc = jumptable[pc];
                    }
                }
                _ => unreachable!(),
            }
            pc += 1;
        }
        Ok(())
    }
}

pub fn run<R: Read, W: Write>(s: &[u8], r: R, w: W) -> Result<()> {
    Program::new(s).run(r, w)
}

#[cfg(test)]
mod test {

    use super::*;
    use serde_derive::*;
    use std::path::{Path, PathBuf};

    #[test]
    fn simple_test() {
        let mut out = Vec::new();
        run(b"", &[] as &[u8], &mut out).unwrap();
        assert_eq!(out, b"");

        let s = br"
++++++++ ++++++++ ++++++++ ++++++++ ++++++++ ++++++++
>+++++
[<+.>-]
";
        let mut out = Vec::new();
        run(s, &[] as &[u8], &mut out).unwrap();
        assert_eq!(out, b"12345");
    }

    fn path_from_project_root(path: impl AsRef<Path>) -> PathBuf {
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        root.push(path.as_ref());
        root
    }

    // Use code-for-blog/2017/bfjit/tests/testcases
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Expected {
        #[serde(rename = "feed-in")]
        feed_in: String,
        #[serde(rename = "expect-out")]
        expect_out: String,
    }

    #[test]
    fn assert_output() {
        let test_dir = path_from_project_root("src/testcases");
        let num_brainfuck_program = glob::glob(&format!("{}/{}", test_dir.display(), "*.bf"))
            .unwrap()
            .map(|bf| {
                let bf = bf.unwrap();
                eprintln!("testing: {}", bf.display());
                let test = PathBuf::from(&bf).with_extension("test");

                let bf = std::fs::read_to_string(bf).unwrap();
                let test = std::fs::read_to_string(test).unwrap();

                let expected: Expected = serde_json::from_str(&test).unwrap();
                let mut out = Vec::new();
                run(bf.as_bytes(), expected.feed_in.as_bytes(), &mut out).unwrap();
                assert_eq!(out, expected.expect_out.as_bytes());
            })
            .count();
        assert!(
            num_brainfuck_program > 5,
            "the number of test cases should be more than 5?"
        );
    }

}
