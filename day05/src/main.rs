use anyhow::{self, Result};
use intcode::{stdin_to_prog, IntcodeComputer};

fn main() -> Result<()> {
    let prog = stdin_to_prog()?;

    let mut cpu = IntcodeComputer::new_with_io(
        prog,
        || if cfg!(feature = "part2") { 5 } else { 1 },
        |i| println!("{}", i),
    );

    cpu.execute()?;
    Ok(())
}
