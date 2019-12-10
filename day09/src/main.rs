use anyhow::{self, bail, Result};
use intcode::*;
use Event::*;

fn main() -> Result<()> {
    let prog = first_arg_to_prog()?;

    let mut cpu = IntcodeComputer::new(prog);
    let mut input = Some(2);
    loop {
        match cpu.execute(input.take())? {
            HaveOutput(x) => {
                println!("{}", x);
            }
            Halted => break Ok(()),
            _ => bail!("Requesting input again"),
        }
    }
}
