use anyhow;
use intcode::*;

fn fix_1202(program: &mut Vec<usize>) {
    program[1] = 12;
    program[2] = 2;
}

fn find_ad_nauseum(initial_program: Vec<usize>, target: usize) -> (usize, usize) {
    for noun in 0..99 {
        for verb in 0..99 {
            let mut new_program = initial_program.clone();
            new_program[1] = noun;
            new_program[2] = verb;

            let mut cpu = IntcodeComputer::new(new_program);

            match cpu.execute() {
                Ok(()) if cpu.memory[0] == target => return (noun, verb),
                _ => continue,
            }
        }
    }
    panic!("no input works!");
}

fn main() -> anyhow::Result<()> {
    let mut program = stdin_to_prog()?;

    if !cfg!(feature = "part2") {
        fix_1202(&mut program);

        let mut cpu = IntcodeComputer::new(program);

        let _ = cpu.execute();

        println!("{}", cpu.memory[0]);
    } else {
        let (noun, verb) = find_ad_nauseum(program, 19690720);

        println!("{}", (100 * noun) + verb);
    }

    Ok(())
}
