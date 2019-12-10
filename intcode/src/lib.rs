use anyhow::{self, ensure, format_err, Error, Result};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::convert::{TryFrom, TryInto};
use std::fs::read_to_string;
use std::io::{self, BufRead};
use std::str::FromStr;

pub enum InstructionType {
    /// Three arguments
    A,
    /// One argument
    I,
    /// Two arguments
    J,
    /// Halt
    H,
}

#[derive(FromPrimitive, Debug, PartialEq, Eq)]
pub enum Opcode {
    // A-TYPE
    ADD = 1,
    MUL = 2,
    LT = 7,
    EQ = 8,
    // J-TYPE
    JIT = 5,
    JIF = 6,
    // I-TYPE
    STR = 3,
    OUT = 4,
    BAS = 9,
    // H-TYPE
    HLT = 99,
}

impl TryFrom<usize> for Opcode {
    type Error = Error;

    fn try_from(int: usize) -> Result<Self, Self::Error> {
        Opcode::from_usize(int).ok_or_else(|| anyhow::format_err!("Unknown opcode {}", int))
    }
}

impl Opcode {
    // fn param_count(&self) -> usize {
    //     use Opcode::*;
    //     match self {
    //         ADD | MUL => 3,
    //         STR | OUT => 1,
    //         HLT => 0,
    //     }
    // }
    fn instruction_length(&self) -> usize {
        use InstructionType::*;
        match self.instruction_type() {
            A => 4,
            J => 3,
            I => 2,
            H => 0,
        }
    }
    fn should_move(&self) -> usize {
        use InstructionType::*;
        match self.instruction_type() {
            A => 4,
            I => 2,
            J | H => 0,
        }
    }

    fn instruction_type(&self) -> InstructionType {
        use InstructionType::*;
        use Opcode::*;
        match self {
            ADD | MUL | LT | EQ => A,
            JIT | JIF => J,
            STR | OUT | BAS => I,
            HLT => H,
        }
    }
}

#[derive(FromPrimitive, Debug, PartialEq, Eq)]
pub enum Mode {
    Position = 0,
    Immediate = 1,
    Relative = 2,
}

impl TryFrom<usize> for Mode {
    type Error = Error;

    fn try_from(int: usize) -> Result<Self, Self::Error> {
        Mode::from_usize(int).ok_or_else(|| anyhow::format_err!("Unknown mode type {}", int))
    }
}

pub struct Operation {
    pub opcode: Opcode,
    pub mode1: Mode,
    pub mode2: Mode,
    pub mode3: Mode,
}

impl TryFrom<isize> for Operation {
    type Error = Error;

    fn try_from(int: isize) -> Result<Self, Self::Error> {
        anyhow::ensure!(
            int.is_positive(),
            "Int was negative when decoding operation"
        );
        let int = int as usize;

        let opcode = int % 100;
        let opcode: Opcode = opcode.try_into()?;
        let mode1 = ((int / 100) % 10).try_into()?;
        let mode2 = ((int / 1000) % 10).try_into()?;
        let mode3 = ((int / 10000) % 10).try_into()?;

        Ok(Operation {
            opcode,
            mode1,
            mode2,
            mode3,
        })
    }
}

pub struct IntcodeComputer {
    pub pc: usize,
    pub rel_base: isize,
    pub memory: Vec<isize>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    RequestingInput,
    HaveOutput(isize),
    Halted,
}

fn convert_addr(i: isize) -> Result<usize> {
    ensure!(i.signum() != -1, "Illegal address");
    Ok(i as usize)
}

impl IntcodeComputer {
    fn decode(&self) -> anyhow::Result<Operation> {
        self.memory[self.pc].try_into()
    }

    fn load_arg(&self, offset: usize, mode: Mode) -> Result<isize> {
        use Mode::*;
        match mode {
            Immediate => Ok(self.memory[self.pc + offset]),
            Position => {
                let addr = self.memory[self.pc + offset];
                self.get_value_from_addr(addr)
            }
            Relative => {
                let rel_base_augend = self.memory[self.pc + offset];
                let addr = self.rel_base + rel_base_augend;
                self.get_value_from_addr(addr)
            }
        }
    }

    fn store_arg(&mut self, offset: usize, mode: Mode, value: isize) -> Result<()> {
        use Mode::*;
        match mode {
            Immediate => return Err(format_err!("Can't store in an immediate")),
            Position => {
                let addr = self.memory[self.pc + offset];
                *self.get_ptr_from_addr(addr)? = value;
            }
            Relative => {
                let rel_base_augend = self.memory[self.pc + offset];
                let addr = self.rel_base + rel_base_augend;
                *self.get_ptr_from_addr(addr)? = value;
            }
        }
        Ok(())
    }

    fn get_value_from_addr(&self, addr: isize) -> Result<isize> {
        let idx = convert_addr(addr)?;
        if idx >= self.memory.len() {
            return Ok(0);
        }
        Ok(self.memory[idx])
    }

    fn get_ptr_from_addr(&mut self, addr: isize) -> Result<&mut isize> {
        let idx = convert_addr(addr)?;
        if idx >= self.memory.len() {
            self.memory.resize(idx + 1, 0);
        }
        Ok(&mut self.memory[idx])
    }

    pub fn new(program: Vec<isize>) -> IntcodeComputer {
        IntcodeComputer {
            pc: 0,
            rel_base: 0,
            memory: program,
        }
    }

    fn exec_operation(
        &mut self,
        operation: Operation,
        input: &mut dyn FnMut() -> Option<isize>,
    ) -> anyhow::Result<Option<Event>> {
        use Opcode::*;
        match operation.opcode {
            ADD => {
                let augend = self.load_arg(1, operation.mode1)?;
                let addend = self.load_arg(2, operation.mode2)?;
                let sum = augend + addend;
                self.store_arg(3, operation.mode3, sum)?;
            }
            MUL => {
                let multiplicand = self.load_arg(1, operation.mode1)?;
                let multiplier = self.load_arg(2, operation.mode2)?;
                let product = multiplicand * multiplier;
                self.store_arg(3, operation.mode3, product)?;
            }
            LT => {
                let left = self.load_arg(1, operation.mode1)?;
                let right = self.load_arg(2, operation.mode2)?;
                let result = left < right;
                let result = if result { 1 } else { 0 };
                self.store_arg(3, operation.mode3, result)?;
            }
            EQ => {
                let left = self.load_arg(1, operation.mode1)?;
                let right = self.load_arg(2, operation.mode2)?;
                let result = left == right;
                let result = if result { 1 } else { 0 };
                self.store_arg(3, operation.mode3, result)?;
            }
            JIT => {
                let test = self.load_arg(1, operation.mode1)?;
                let addr = self.load_arg(2, operation.mode2)?;
                let addr = convert_addr(addr)?;
                if test != 0 {
                    self.pc = addr;
                } else {
                    self.pc += operation.opcode.instruction_length();
                }
            }
            JIF => {
                let test = self.load_arg(1, operation.mode1)?;
                let addr = self.load_arg(2, operation.mode2)?;
                let addr = convert_addr(addr)?;
                if test == 0 {
                    self.pc = addr;
                } else {
                    self.pc += operation.opcode.instruction_length();
                }
            }
            STR => {
                if let Some(input) = (input)() {
                    self.store_arg(1, operation.mode1, input)?;
                } else {
                    return Ok(Some(Event::RequestingInput));
                }
            }
            OUT => {
                let output = self.load_arg(1, operation.mode1)?;
                self.pc += operation.opcode.should_move();
                return Ok(Some(Event::HaveOutput(output)));
            }
            BAS => {
                let augend = self.load_arg(1, operation.mode1)?;
                self.rel_base += augend;
            }
            HLT => return Ok(Some(Event::Halted)),
            // _ => unimplemented!(),
        }
        self.pc += operation.opcode.should_move();

        Ok(None)
    }

    fn exec_current(&mut self, input: &mut dyn FnMut() -> Option<isize>) -> Result<Option<Event>> {
        let operation = self.decode()?;
        self.exec_operation(operation, input)
    }

    pub fn execute(&mut self, input: &mut dyn FnMut() -> Option<isize>) -> Result<Event> {
        use Event::*;
        let mut result = self.exec_current(input)?;
        loop {
            match result {
                None => {
                    result = self.exec_current(input)?;
                }
                Some(RequestingInput) => break Ok(RequestingInput),
                Some(x) => break Ok(x),
            }
        }
    }
}

pub fn stdin_to_prog() -> anyhow::Result<Vec<isize>> {
    let stdin = io::stdin();
    let stdin = stdin.lock();

    stdin
        .split(b',')
        .filter(|maybe_bytes| match maybe_bytes {
            Ok(ref bytes) => bytes.len() > 0 && bytes[0] != b'\n',
            _ => true,
        })
        .map(|maybe_bytes| -> anyhow::Result<isize> {
            let bytes = maybe_bytes?;
            let string = std::str::from_utf8(&bytes)?;
            let num = isize::from_str(string.trim())?;
            Ok(num)
        })
        .collect()
}

pub fn first_arg_to_prog() -> anyhow::Result<Vec<isize>> {
    // println!("{}", std::env::current_dir()?.display());
    let string = read_to_string("input")?;
    string
        .split(',')
        .map(|string| -> anyhow::Result<isize> {
            let num = isize::from_str(string.trim())?;
            Ok(num)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Event, IntcodeComputer};
    use Event::*;

    macro_rules! tests {
        ($($name:ident {
            prog: $prog:expr,
            final: $final:expr,
        });*) => {
            $(
                #[test]
                fn $name() {
                    let program = $prog;

                    let mut cpu = IntcodeComputer::new(program);

                    assert_eq!(cpu.execute(None).unwrap(), Event::Halted);

                    assert_eq!(cpu.memory, $final);
                }
            )*
        };
    }

    macro_rules! iotests {
        ($($name:ident {
            prog: $prog:expr,
            final: $final:expr,
            input: $input:expr,
            output: $output:expr,
        };)*) => {
            $(
                #[test]
                fn $name() {
                    let program = $prog;

                    let mut cpu = IntcodeComputer::new(program);

                    let mut input = None;

                    loop {
                        let res = cpu.execute(input).unwrap();
                        input = None;
                        match res {
                            Event::Halted => break,
                            Event::RequestingInput => { input = Some(($input)()); },
                            Event::HaveOutput(x) => { ($output)(x); }
                        }
                    }

                    assert_eq!(cpu.memory, $final);
                }
            )*
        };
    }

    tests! {
        day02_main_example {
            prog: vec![1,9,10,3,2,3,11,0,99,30,40,50],
            final: vec![3500,9,10,70, 2,3,11,0,99,30,40,50],
        };
        day02_smol_1 {
            prog: vec![1,0,0,0,99],
            final: vec![2,0,0,0,99],
        };
        day02_smol_2 {
            prog: vec![2,3,0,3,99],
            final: vec![2,3,0,6,99],
        };
        day02_smol_3 {
            prog: vec![2,4,4,5,99,0],
            final: vec![2,4,4,5,99,9801],
        };
        day02_smol_4 {
            prog: vec![1,1,1,4,99,5,6,0,99],
            final: vec![30,1,1,4,2,5,6,0,99],
        };
        day05_main_example {
            prog: vec![1002,4,3,4,33],
            final: vec![1002,4,3,4,99],
        };
        day05_some_notes {
            prog: vec![1101,100,-1,4,0],
            final: vec![1101,100,-1,4,99],
        }
    }

    iotests! {
        day05_custom_io {
            prog: vec![3, 5, 4, 5, 99, 0],
            final: vec![3, 5, 4, 5, 99, 6],
            input: || { 6 },
            output: |num| { assert_eq!(num, 6)},
        };
        output_immediate {
            prog: vec![104, 5, 99],
            final: vec![104, 5, 99],
            input: || { panic!("no input")},
            output: |num| { assert_eq!(num, 5)},
        };
    }

    fn io_halt(mut cpu: IntcodeComputer, pairs: impl IntoIterator<Item = (Option<isize>, Event)>) {
        for (input, event) in pairs {
            assert_eq!(cpu.execute(input).unwrap(), event);
        }
    }

    const END: (Option<isize>, Event) = (None, Event::Halted);

    #[test]
    fn cmp_pos_eq_8() {
        let program = vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8];

        let cpu = IntcodeComputer::new(program.clone());

        io_halt(cpu, vec![(Some(8), Event::HaveOutput(1)), END]);

        let cpu = IntcodeComputer::new(program.clone());

        io_halt(cpu, vec![(Some(9), Event::HaveOutput(0)), END]);
    }

    #[test]
    fn cmp_pos_lt_8() {
        let program = vec![3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8];

        let cpu = IntcodeComputer::new(program.clone()); // || 7, |i| assert_eq!(i, 1));

        io_halt(cpu, vec![(Some(7), HaveOutput(1)), END]);

        let cpu = IntcodeComputer::new(program.clone()); // || 9, |i| assert_eq!(i, 0));

        io_halt(cpu, vec![(Some(9), HaveOutput(0)), END]);
    }

    #[test]
    fn cmp_imm_eq_8() {
        let program = vec![3, 3, 1108, -1, 8, 3, 4, 3, 99];

        let cpu = IntcodeComputer::new(program.clone()); // || 8, |i| assert_eq!(i, 1));
        io_halt(cpu, vec![(Some(8), Event::HaveOutput(1)), END]);

        let cpu = IntcodeComputer::new(program.clone());
        io_halt(cpu, vec![(Some(9), Event::HaveOutput(0)), END]);
    }

    #[test]
    fn cmp_imm_lt_8() {
        let program = vec![3, 3, 1107, -1, 8, 3, 4, 3, 99];

        let cpu = IntcodeComputer::new(program.clone());
        io_halt(cpu, vec![(Some(7), HaveOutput(1)), END]);

        let cpu = IntcodeComputer::new(program.clone()); // || 9, |i| assert_eq!(i, 0));
        io_halt(cpu, vec![(Some(9), HaveOutput(0)), END]);
    }

    #[test]
    fn jmp_pos_zero() {
        let program = vec![3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9];

        let cpu = IntcodeComputer::new(program.clone()); // || 0, |i| assert_eq!(i, 0));
        io_halt(cpu, vec![(Some(0), HaveOutput(0)), END]);

        let cpu = IntcodeComputer::new(program.clone()); // || 2, |i| assert_eq!(i, 1));
        io_halt(cpu, vec![(Some(2), HaveOutput(1)), END]);
    }

    #[test]
    fn jmp_imm_zero() {
        let program = vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1];

        let cpu = IntcodeComputer::new(program.clone()); // || 0, |i| assert_eq!(i, 0));
        io_halt(cpu, vec![(Some(0), HaveOutput(0)), END]);

        let cpu = IntcodeComputer::new(program.clone()); // || 2, |i| assert_eq!(i, 1));
        io_halt(cpu, vec![(Some(2), HaveOutput(1)), END]);
    }

    #[test]
    fn larger_example() {
        let program = vec![
            3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36, 98, 0,
            0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000, 1, 20, 4,
            20, 1105, 1, 46, 98, 99,
        ];

        let cpu = IntcodeComputer::new(program.clone());
        io_halt(cpu, vec![(Some(7), HaveOutput(999)), END]);
        let cpu = IntcodeComputer::new(program.clone()); // || 8, |i| assert_eq!(i, 1000));
        io_halt(cpu, vec![(Some(8), HaveOutput(1000)), END]);

        let cpu = IntcodeComputer::new(program.clone());
        io_halt(cpu, vec![(Some(9), HaveOutput(1001)), END]);
    }

    #[test]
    fn rel_base_quine() {
        let program = vec![
            109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99,
        ];
        let cpu = IntcodeComputer::new(program.clone());

        let expect = program
            .into_iter()
            .map(|num| (None, HaveOutput(num)))
            .chain(std::iter::once(END));

        io_halt(cpu, expect);
    }

    #[test]
    fn rel_base_16_digits() {
        let program = vec![1102, 34915192, 34915192, 7, 4, 7, 99, 0];
        let cpu = IntcodeComputer::new(program);

        io_halt(cpu, vec![(None, HaveOutput(1219070632396864)), END]);
    }

    #[test]
    fn rel_base_lorge() {
        let program = vec![104, 1125899906842624, 99];
        let cpu = IntcodeComputer::new(program);

        io_halt(cpu, vec![(None, HaveOutput(1125899906842624)), END]);
    }
}
