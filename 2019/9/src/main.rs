use env_logger;
use log;
use std::fmt;
use std::io::Write;
use std::io::{stdin, stdout};

const INPUT_FILE: &str = "input.txt";

fn main() {
    env_logger::init();
    println!("running part 1");
    part1();
}

fn part1() {
    Cpu::new(None).run();
}

#[derive(Copy, Clone)]
enum Parameter {
    Position(i64),
    Immediate(i64),
    Relative(i64),
}

impl fmt::Debug for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Parameter::Position(v) => write!(f, "P({})", v),
            Parameter::Immediate(v) => write!(f, "I({})", v),
            Parameter::Relative(v) => write!(f, "R({})", v),
        }
    }
}

#[derive(Debug)]
enum Instruction {
    ADD(Vec<Parameter>),
    MUL(Vec<Parameter>),
    INPUT(Vec<Parameter>),
    OUTPUT(Vec<Parameter>),
    JUMP(bool, Vec<Parameter>),
    LESSTHAN(Vec<Parameter>),
    EQUALS(Vec<Parameter>),
    RELBASE(Vec<Parameter>),
    HALT,
}

struct Cpu {
    ip: usize,
    rbase: i64,
    pub memory: Vec<i64>,
}

impl Cpu {
    fn new(mem: Option<Vec<i64>>) -> Cpu {
        let mut memory = match mem {
            Some(m) => m,
            None => process_input(),
        };
        memory.resize(4096, 0);
        Cpu {
            ip: 0,
            rbase: 0,
            memory,
        }
    }

    // Build a vector of |cnt| parameters for the instruction based on
    // the flags in the opcode representing the parameter modes.
    fn pack_parameters(&mut self, cnt: usize) -> Vec<Parameter> {
        let mut vec = Vec::new();
        let mut flags = self.memory[self.ip - 1] / 100;
        print!(" {:03} ", flags);
        for i in 0..cnt {
            let val = self.memory[self.ip + i] as i64;
            let param = match flags % 10 {
                0 => Parameter::Position(val),
                1 => Parameter::Immediate(val),
                2 => Parameter::Relative(val),
                _ => panic!("invalid parameter mode"),
            };
            flags /= 10;
            vec.push(param);
        }
        self.ip += cnt;
        vec
    }

    fn unpack_parameter(&self, p: Parameter) -> i64 {
        let v = match p {
            Parameter::Immediate(x) => x,
            Parameter::Position(x) => self.memory[x as usize],
            Parameter::Relative(x) => self.memory[(self.rbase + x) as usize],
        };
        v
    }

    fn fetch_and_decode(&mut self) -> Instruction {
        self.ip += 1;
        let opcode = self.memory[self.ip - 1] % 100;
        print!("  {:02}  ", opcode);
        match opcode {
            1 => Instruction::ADD(self.pack_parameters(3)),
            2 => Instruction::MUL(self.pack_parameters(3)),
            3 => Instruction::INPUT(self.pack_parameters(1)),
            4 => Instruction::OUTPUT(self.pack_parameters(1)),
            5 => Instruction::JUMP(true, self.pack_parameters(2)),
            6 => Instruction::JUMP(false, self.pack_parameters(2)),
            7 => Instruction::LESSTHAN(self.pack_parameters(3)),
            8 => Instruction::EQUALS(self.pack_parameters(3)),
            9 => Instruction::RELBASE(self.pack_parameters(1)),
            99 => Instruction::HALT,
            _ => panic!("Invalid opcode: {} at position {}", opcode, self.ip - 1),
        }
    }

    fn run(mut self) -> Cpu {
        println!("  #    ip    op    f     instruction");
        println!(" ---  ----  ----  ---  ----------------");
        let mut cnt = 1;
        while self.ip < self.memory.len() {
            print!("{:3}:  {:04} ", cnt, self.ip);
            let instruction = self.fetch_and_decode();
            println!(" {:?}", instruction);
            cnt += 1;
            match instruction {
                Instruction::ADD(args) => self.op_add(args),
                Instruction::MUL(args) => self.op_mul(args),
                Instruction::INPUT(args) => self.op_input(args),
                Instruction::OUTPUT(args) => self.op_output(args),
                Instruction::JUMP(test, args) => self.op_jump(test, args),
                Instruction::LESSTHAN(args) => self.op_lessthan(args),
                Instruction::EQUALS(args) => self.op_equals(args),
                Instruction::RELBASE(args) => self.op_relbase(args),
                Instruction::HALT => break,
            }
        }
        self
    }

    // Instruction implementations
    fn op_add(&mut self, args: Vec<Parameter>) {
        assert_eq!(args.len(), 3);
        if let Parameter::Position(dest) = args[2] {
            self.memory[dest as usize] =
                self.unpack_parameter(args[0]) + self.unpack_parameter(args[1]);
        } else {
            panic!("Dest argument should never be immediate");
        }
    }

    fn op_mul(&mut self, args: Vec<Parameter>) {
        assert_eq!(args.len(), 3);
        if let Parameter::Position(dest) = args[2] {
            self.memory[dest as usize] =
                self.unpack_parameter(args[0]) * self.unpack_parameter(args[1]);
        } else {
            panic!("Dest argument should never be immediate");
        }
    }

    fn op_input(&mut self, args: Vec<Parameter>) {
        assert_eq!(args.len(), 1);

        print!("$ ");
        stdout().flush().unwrap();
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).unwrap();
        let dest = self.unpack_parameter(args[0]);
        self.memory[dest as usize] = buffer.trim().parse().unwrap();
        println!("\t[{}] = {}", dest, self.memory[dest as usize]);
    }

    fn op_output(&self, args: Vec<Parameter>) {
        assert_eq!(args.len(), 1);
        println!("> {}", self.unpack_parameter(args[0]));
    }

    fn op_jump(&mut self, test: bool, args: Vec<Parameter>) {
        assert_eq!(args.len(), 2);
        if (self.unpack_parameter(args[0]) != 0) == test {
            self.ip = self.unpack_parameter(args[1]) as usize;
        }
    }

    fn op_lessthan(&mut self, args: Vec<Parameter>) {
        assert_eq!(args.len(), 3);
        if let Parameter::Position(dest) = args[2] {
            self.memory[dest as usize] =
                (self.unpack_parameter(args[0]) < self.unpack_parameter(args[1])) as i64;
        } else {
            panic!("Dest argument should never be immediate");
        }
    }

    fn op_equals(&mut self, args: Vec<Parameter>) {
        assert_eq!(args.len(), 3);
        if let Parameter::Position(dest) = args[2] {
            self.memory[dest as usize] =
                (self.unpack_parameter(args[0]) == self.unpack_parameter(args[1])) as i64;
        } else {
            panic!("Dest argument should never be immediate");
        }
    }

    fn op_relbase(&mut self, args: Vec<Parameter>) {
        assert_eq!(args.len(), 1);
        let old_rbase = self.rbase;
        self.rbase += self.unpack_parameter(args[0]);
        println!("\trbase = {}", self.rbase);
    }
}

fn process_input() -> Vec<i64> {
    let mut v: Vec<i64> = std::fs::read_to_string(INPUT_FILE)
        .unwrap()
        .trim()
        .split(',')
        .map(|mass| mass.parse::<i64>().unwrap())
        .collect();
    v.resize(4096, 0);
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example1() {
        {
            let cpu = Cpu::new(Some(vec![1101, 100, -1, 4, 0])).run();
            assert_eq!(cpu.memory[4], 99);
        }
        {
            let cpu = Cpu::new(Some(vec![1002, 4, 3, 4, 33])).run();
            assert_eq!(cpu.memory[4], 99);
        }
    }

    #[test]
    #[ignore]
    fn manual_output_confirmation() {
        Cpu::new(Some(vec![
            109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99,
        ]))
        .run();
        Cpu::new(Some(vec![1102, 34915192, 34915192, 7, 4, 7, 99, 0])).run();
        Cpu::new(Some(vec![104, 1125899906842624, 99])).run();
    }
}
