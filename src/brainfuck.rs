// https://eli.thegreenplace.net/2017/adventures-in-jit-compilation-part-1-an-interpreter.html
// https://esolangs.org/wiki/Brainfuck
use failure;
use log::*;
use std::io::Read;
use std::io::Write;

use mmap;

type Result<T> = std::result::Result<T, failure::Error>;

pub trait Brainfuck {
    fn new(s: &[u8]) -> Self;
    // fn run(&self, read: impl Read, mut write: impl Write) -> Result<()>;
    fn run(&self, read: impl Read, write: impl Write) -> Result<()>;
}

fn filter_instructions(s: &[u8]) -> Vec<u8> {
    s.into_iter()
        .filter(|c| match c {
            b'>' | b'<' | b'+' | b'-' | b'.' | b',' | b'[' | b']' => true,
            _ => false,
        })
        .cloned()
        .collect()
}

struct Interpreter1 {
    instructions: Vec<u8>,
}

impl Interpreter1 {
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
}

impl Brainfuck for Interpreter1 {
    fn new(s: &[u8]) -> Interpreter1 {
        Interpreter1 {
            instructions: filter_instructions(s),
        }
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
                b',' => {
                    memory[data_ptr] = input
                        .next()
                        .ok_or_else(|| failure::err_msg("input unavailable"))?
                        .expect("read error");
                }
                b'.' => {
                    write.write_all(&memory[data_ptr..(data_ptr + 1)])?;
                    write.flush().unwrap();
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

// Optimized interpreter Part 1 - take 2
enum Op {
    // For take 2
    IncPtr(usize),
    DecPtr(usize),
    IncData(usize),
    DecData(usize),
    ReadStdin(usize),
    WriteStdout(usize),
    JumpIfDataIsZero(usize),
    JumpIfDataIsNotZero(usize),
    // For take 3
    LoopSetToZero,
    LoopMovePtr(isize),
    LoopMoveData(isize),
}

fn translate_program(instructions: &[u8], do_optimize_loop: bool) -> Vec<Op> {
    let mut ops: Vec<Op> = Vec::new();

    let mut open_bracket_stack: Vec<usize> = Vec::new();

    let mut pc = 0;
    let instructions_size = instructions.len();

    while pc < instructions_size {
        match instructions[pc] {
            b'[' => {
                open_bracket_stack.push(ops.len());
                ops.push(Op::JumpIfDataIsZero(0));
                pc += 1;
            }
            b']' => {
                assert!(!open_bracket_stack.is_empty());
                let open_bracket_offset = open_bracket_stack.pop().unwrap();
                let ops_len = ops.len();

                // For part 1 - take 3
                let loop_optimized = if do_optimize_loop {
                    if let Some(op) = optimize_loop(&ops, open_bracket_offset) {
                        // Replace this whole loop with optimized_loop.
                        ops.truncate(open_bracket_offset);
                        ops.push(op);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
                if !loop_optimized {
                    if let Op::JumpIfDataIsZero(ref mut off) = &mut ops[open_bracket_offset] {
                        *off = ops_len;
                    } else {
                        unreachable!();
                    }
                    ops.push(Op::JumpIfDataIsNotZero(open_bracket_offset));
                }
                pc += 1;;
            }
            x => {
                let start = pc;
                pc += 1;
                while pc < instructions_size && instructions[pc] == x {
                    pc += 1;
                }
                let repeats = pc - start;
                match x {
                    b'>' => ops.push(Op::IncPtr(repeats)),
                    b'<' => ops.push(Op::DecPtr(repeats)),
                    b'+' => ops.push(Op::IncData(repeats)),
                    b'-' => ops.push(Op::DecData(repeats)),
                    b',' => ops.push(Op::ReadStdin(repeats)),
                    b'.' => ops.push(Op::WriteStdout(repeats)),
                    _ => unreachable!(),
                }
            }
        }
    }
    ops
}

struct Interpreter2 {
    ops: Vec<Op>,
}

impl Brainfuck for Interpreter2 {
    fn new(s: &[u8]) -> Interpreter2 {
        let instructions = filter_instructions(s);
        let ops = translate_program(&instructions, false);
        Interpreter2 { ops }
    }
    fn run(&self, read: impl Read, mut write: impl Write) -> Result<()> {
        let mut memory: Vec<u8> = vec![0; 30000];
        let mut pc = 0;
        let mut data_ptr: usize = 0;
        let mut input = read.bytes();
        while pc < self.ops.len() {
            match self.ops[pc] {
                Op::IncPtr(n) => data_ptr += n,
                Op::DecPtr(n) => data_ptr -= n,
                Op::IncData(n) => memory[data_ptr] = memory[data_ptr].wrapping_add(n as u8),
                Op::DecData(n) => memory[data_ptr] = memory[data_ptr].wrapping_sub(n as u8),
                Op::ReadStdin(n) => {
                    for _ in 0..n {
                        memory[data_ptr] = input
                            .next()
                            .ok_or_else(|| failure::err_msg("input unavailable"))?
                            .expect("read error");
                    }
                }
                Op::WriteStdout(n) => {
                    for _ in 0..n {
                        write.write_all(&memory[data_ptr..(data_ptr + 1)])?;
                    }
                    write.flush().unwrap();
                }
                Op::JumpIfDataIsZero(n) => {
                    if memory[data_ptr] == 0 {
                        pc = n;
                    }
                }
                Op::JumpIfDataIsNotZero(n) => {
                    if memory[data_ptr] != 0 {
                        pc = n;
                    }
                }
                _ => unreachable!(),
            }
            pc += 1;
        }
        Ok(())
    }
}

// Optimized interpreter Part 1 - take 3

// See https://github.com/eliben/code-for-blog/blob/master/2017/bfjit/optinterp3.cpp
fn optimize_loop(ops: &[Op], loop_start: usize) -> Option<Op> {
    // ...[....]
    //    ^
    //    loop-start

    if ops.len() - loop_start == 2 {
        // [x]
        match ops[loop_start + 1] {
            Op::IncData(_) | Op::DecData(_) => Some(Op::LoopSetToZero),
            Op::IncPtr(n) => Some(Op::LoopMovePtr(n as isize)),
            Op::DecPtr(n) => Some(Op::LoopMovePtr(-(n as isize))),
            _ => None,
        }
    } else if ops.len() - loop_start == 5 {
        // Detect patterns: -<+> and ->+<
        match (&ops[loop_start + 1], &ops[loop_start + 3]) {
            (Op::DecData(1), Op::IncData(1)) => {
                match (&ops[loop_start + 2], &ops[loop_start + 4]) {
                    (Op::IncPtr(m), Op::DecPtr(n)) if m == n => Some(Op::LoopMoveData(*m as isize)),
                    (Op::DecPtr(m), Op::IncPtr(n)) if m == n => {
                        Some(Op::LoopMoveData(-(*m as isize)))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    } else {
        None
    }
}

struct Interpreter3 {
    ops: Vec<Op>,
}

impl Brainfuck for Interpreter3 {
    fn new(s: &[u8]) -> Interpreter3 {
        let instructions = filter_instructions(s);
        let ops = translate_program(&instructions, true);
        Interpreter3 { ops }
    }
    fn run(&self, read: impl Read, mut write: impl Write) -> Result<()> {
        let mut memory: Vec<u8> = vec![0; 300000];
        let mut pc = 0;
        let mut data_ptr: usize = 0;
        let mut input = read.bytes();
        while pc < self.ops.len() {
            match self.ops[pc] {
                Op::IncPtr(n) => data_ptr += n,
                Op::DecPtr(n) => data_ptr -= n,
                Op::IncData(n) => memory[data_ptr] = memory[data_ptr].wrapping_add(n as u8),
                Op::DecData(n) => memory[data_ptr] = memory[data_ptr].wrapping_sub(n as u8),
                Op::ReadStdin(n) => {
                    for _ in 0..n {
                        memory[data_ptr] = input
                            .next()
                            .ok_or_else(|| failure::err_msg("input unavailable"))?
                            .expect("read error");
                    }
                }
                Op::WriteStdout(n) => {
                    for _ in 0..n {
                        write.write_all(&memory[data_ptr..(data_ptr + 1)])?;
                    }
                    write.flush().unwrap();
                }
                Op::JumpIfDataIsZero(n) => {
                    if memory[data_ptr] == 0 {
                        pc = n;
                    }
                }
                Op::JumpIfDataIsNotZero(n) => {
                    if memory[data_ptr] != 0 {
                        pc = n;
                    }
                }
                Op::LoopSetToZero => memory[data_ptr] = 0,
                Op::LoopMovePtr(n) => {
                    while memory[data_ptr] != 0 {
                        data_ptr = (data_ptr as isize + n) as usize
                    }
                }
                Op::LoopMoveData(n) => {
                    if memory[data_ptr] != 0 {
                        let move_to_ptr = ((data_ptr as isize) + n) as usize;
                        memory[move_to_ptr] = memory[move_to_ptr].wrapping_add(memory[data_ptr]);
                        memory[data_ptr] = 0;
                    }
                }
            }
            pc += 1;
        }
        Ok(())
    }
}

// Adventures in JIT compilation: Part 2 - an x64 JIT
// https://eli.thegreenplace.net/2017/adventures-in-jit-compilation-part-2-an-x64-jit/

// https://github.com/eliben/code-for-blog/blob/master/2017/bfjit/simplejit.cpp
// https://github.com/1uks/brainfuck-jit/blob/master/src/main.rs

struct CodeEmitter {
    code: Vec<u8>,
}

impl CodeEmitter {
    fn new() -> CodeEmitter {
        CodeEmitter { code: Vec::new() }
    }
    fn emit_byte(&mut self, byte: u8) {
        self.code.push(byte);
    }
    fn emit_bytes(&mut self, bytes: &[u8]) {
        self.code.extend(bytes);
    }
    fn emit_u32(&mut self, n: u32) {
        let bytes: [u8; 4] = unsafe { std::mem::transmute(n.to_le()) };
        self.emit_bytes(&bytes)
    }
    fn emit_u64(&mut self, n: u64) {
        self.emit_u32((n & 0xFFFFFFFF) as u32);
        self.emit_u32(((n >> 32) & 0xFFFFFFFF) as u32);
    }
    fn size(&self) -> usize {
        self.code.len()
    }

    fn replace_u32_at_offset(&mut self, offset: usize, n: u32) {
        let bytes: [u8; 4] = unsafe { std::mem::transmute(n.to_le()) };
        self.code[offset] = bytes[0];
        self.code[offset + 1] = bytes[1];
        self.code[offset + 2] = bytes[2];
        self.code[offset + 3] = bytes[3];
    }
}

fn compute_relative_32bit_offset(jump_from: usize, jump_to: usize) -> u32 {
    if jump_to > jump_from {
        let diff = jump_to - jump_from;
        assert!(diff < (1 << 31));
        diff as u32
    } else {
        let diff = jump_from - jump_to;
        assert!(diff - 1 < (1 << 31));
        // 2's complement
        !(diff as u32) + 1
    }
}

// fn simple_jit(instructions: &[u8], _read_fd: u8, _write_fd: u8) {
fn simple_jit(instructions: &[u8]) {
    // https://www.systutorials.com/240986/x86-64-calling-convention-by-gcc/
    // The calling convention of the System V AMD64 ABI is followed on GNU/Linux.
    // The registers RDI, RSI, RDX, RCX, R8, and R9 are used for integer
    // and memory address arguments and XMM0, XMM1, XMM2, XMM3, XMM4, XMM5, XMM6 and XMM7
    // are used for floating point arguments.

    let memory: Vec<u8> = vec![0; 30000];
    let mut open_bracket_stack = Vec::<usize>::new();
    let mut emitter = CodeEmitter::new();

    // movabs <address of memory.data>, %r13
    emitter.emit_bytes(&[0x49, 0xBD]);
    emitter.emit_u64(memory.as_ptr() as u64);

    for inst in instructions {
        match inst {
            // Inc %r13
            b'>' => emitter.emit_bytes(&[0x49, 0xFF, 0xC5]),
            // Dec %r13
            b'<' => emitter.emit_bytes(&[0x49, 0xFF, 0xCD]),
            b'+' => {
                // addb $1, 0(%r13)
                emitter.emit_bytes(&[0x41, 0x80, 0x45, 0x00, 0x01]);
            }
            b'-' => {
                // subb $1, 0(%r13)
                emitter.emit_bytes(&[0x41, 0x80, 0x6D, 0x00, 0x01]);
            }
            b'.' => {
                // To emit one byte to stdout, call the write syscall with fd=1 (for
                // stdout), buf=address of byte, count=1.
                //
                // mov $1, %rax
                // mov $1, %rdi
                // mov %r13, %rsi
                // mov $1, %rdx
                // syscall
                emitter.emit_bytes(&[0x48, 0xC7, 0xC0, 0x01, 0x00, 0x00, 0x00]);
                emitter.emit_bytes(&[0x48, 0xC7, 0xC7, 0x01, 0x00, 0x00, 0x00]);
                emitter.emit_bytes(&[0x4C, 0x89, 0xEE]);
                emitter.emit_bytes(&[0x48, 0xC7, 0xC2, 0x01, 0x00, 0x00, 0x00]);
                emitter.emit_bytes(&[0x0F, 0x05]);
            }
            b',' => {
                // To read one byte from stdin, call the read syscall with fd=0 (for
                // stdin),
                // buf=address of byte, count=1.
                emitter.emit_bytes(&[0x48, 0xC7, 0xC0, 0x00, 0x00, 0x00, 0x00]);
                emitter.emit_bytes(&[0x48, 0xC7, 0xC7, 0x00, 0x00, 0x00, 0x00]);
                emitter.emit_bytes(&[0x4C, 0x89, 0xEE]);
                emitter.emit_bytes(&[0x48, 0xC7, 0xC2, 0x01, 0x00, 0x00, 0x00]);
                emitter.emit_bytes(&[0x0F, 0x05]);
            }
            b'[' => {
                // For the jumps we always emit the instruciton for 32-bit pc-relative
                // jump, without worrying about potentially short jumps and relaxation.

                // cmpb $0, 0(%r13)
                emitter.emit_bytes(&[0x41, 0x80, 0x7d, 0x00, 0x00]);

                // Save the location in the stack, and emit JZ (with 32-bit relative
                // offset) with 4 placeholder zeroes that will be fixed up later.
                open_bracket_stack.push(emitter.size());
                emitter.emit_bytes(&[0x0F, 0x84]);
                emitter.emit_u32(0);
            }
            b']' => {
                assert!(!open_bracket_stack.is_empty());
                let open_bracket_offset = open_bracket_stack.pop().unwrap();
                // cmpb $0, 0(%r13)
                emitter.emit_bytes(&[0x41, 0x80, 0x7d, 0x00, 0x00]);

                let jump_back_from = emitter.size() + 6;
                let jump_back_to = open_bracket_offset + 6;
                let pcrel_offset_back = compute_relative_32bit_offset(jump_back_from, jump_back_to);

                // jnz <open_bracket_location>
                emitter.emit_bytes(&[0x0F, 0x85]);
                emitter.emit_u32(pcrel_offset_back);

                let jump_forward_from = open_bracket_offset + 6;
                let jump_forward_to = emitter.size();

                let pcrel_offset_forward =
                    compute_relative_32bit_offset(jump_forward_from, jump_forward_to);
                emitter.replace_u32_at_offset(open_bracket_offset + 2, pcrel_offset_forward);
            }
            _ => unreachable!(),
        }
    }

    // Emit a 'ret'
    emitter.emit_byte(0xC3);

    // JitProgram
    let rwx = &[
        mmap::MapOption::MapReadable,
        mmap::MapOption::MapWritable,
        mmap::MapOption::MapExecutable,
    ];

    let mapping = mmap::MemoryMap::new(emitter.size(), rwx).unwrap();
    unsafe {
        std::ptr::copy(emitter.code.as_ptr(), mapping.data(), emitter.size());
    }
    debug!("jit: size: {}", emitter.size());
    let func: fn() = unsafe { std::mem::transmute(mapping.data()) };
    func();
}

struct Jit1 {
    instructions: Vec<u8>,
}

impl Brainfuck for Jit1 {
    fn new(s: &[u8]) -> Jit1 {
        let instructions = filter_instructions(s);
        Jit1 { instructions }
    }
    fn run(&self, _: impl Read, _: impl Write) -> Result<()> {
        // TODO: Use read and write as stdin and stdout
        simple_jit(&self.instructions);
        Ok(())
    }
}

pub fn run_default<R: Read, W: Write>(s: &[u8], r: R, w: W) -> Result<()> {
    Interpreter1::new(s).run(r, w)
}

pub fn run<R: Read, W: Write>(
    s: &[u8],
    r: R,
    w: W,
    optimize: Option<u64>,
    jit: bool,
) -> Result<()> {
    if jit {
        Jit1::new(s).run(r, w)
    } else if let Some(o) = optimize {
        match o {
            1 => Interpreter1::new(s).run(r, w),
            2 => Interpreter2::new(s).run(r, w),
            3 => Interpreter3::new(s).run(r, w),
            _ => unimplemented!(),
        }
    } else {
        // TODO: Fix the default
        run_default(s, r, w)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use serde_derive::*;
    use std::path::{Path, PathBuf};

    #[test]
    fn simple_test() {
        let mut out = Vec::new();
        run_default(b"", &[] as &[u8], &mut out).unwrap();
        assert_eq!(out, b"");

        let s = br"
++++++++ ++++++++ ++++++++ ++++++++ ++++++++ ++++++++
>+++++
[<+.>-]
";
        let mut out = Vec::new();
        run_default(s, &[] as &[u8], &mut out).unwrap();
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

    fn assert_program_output<P: Brainfuck>(s: &[u8], stdin: &[u8], expected_output: &[u8]) {
        let mut out = Vec::new();
        P::new(s).run(stdin, &mut out).unwrap();
        assert_eq!(out, expected_output);
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
                let bf = bf.as_bytes();
                let test = std::fs::read_to_string(test).unwrap();
                let expected: Expected = serde_json::from_str(&test).unwrap();

                let stdin = expected.feed_in.as_bytes();
                let expected_output = expected.expect_out.as_bytes();

                assert_program_output::<Interpreter1>(bf, stdin, expected_output);
                assert_program_output::<Interpreter2>(bf, stdin, expected_output);
                assert_program_output::<Interpreter3>(bf, stdin, expected_output);
            })
            .count();
        assert!(
            num_brainfuck_program > 5,
            "the number of test cases should be more than 5?"
        );
    }

}
