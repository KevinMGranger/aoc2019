use anyhow::{self, format_err, Result};
use intcode::*;
use std::collections::HashSet;
use Event::*;

fn compute_chain(phases: impl IntoIterator<Item = isize>, program: &Vec<isize>) -> Result<isize> {
    let mut signal = 0;
    for phase_setting in phases {
        let mut cpu = IntcodeComputer::new(program.clone());
        dbg!(cpu.execute(Some(phase_setting))?);
        if let HaveOutput(x) = dbg!(cpu.execute(Some(signal))?) {
            signal = x;
        }
        dbg!(cpu.execute(None)?);
    }

    Ok(signal)
}

struct FeedbackLoopAmp {
    cpu: IntcodeComputer,
}

impl FeedbackLoopAmp {
    fn new(phase_signal: isize, mut cpu: IntcodeComputer) -> Result<FeedbackLoopAmp> {
        if let RequestingInput = cpu.execute(Some(phase_signal))? {
            Ok(FeedbackLoopAmp { cpu })
        } else {
            Err(format_err!("Bad event"))
        }
    }
    fn call(&mut self, signal: isize) -> Result<Option<isize>> {
        match self.cpu.execute(Some(signal))? {
            HaveOutput(x) => Ok(Some(x)),
            Halted => Ok(None),
            _ => Err(format_err!("Bad event")),
        }
    }
}

fn feedback_loop(phases: impl IntoIterator<Item = isize>, program: &Vec<isize>) -> Result<isize> {
    let mut signal = 0;
    let mut idx = 0;

    let mut cpus = phases
        .into_iter()
        .map(|setting| FeedbackLoopAmp::new(setting, IntcodeComputer::new(program.clone())))
        .collect::<Result<Vec<FeedbackLoopAmp>>>()?;

    loop {
        if let Some(new_signal) = cpus[idx].call(signal)? {
            signal = new_signal;
            idx = (idx + 1) % cpus.len();
        } else {
            break Ok(signal);
        }
    }
}

fn main() -> Result<()> {
    let program = first_arg_to_prog()?;
    let mut max_output = 0;

    let phase_range = if cfg!(feature = "part2") { 5..=9 } else { 0..=4 };

    'p1: for phase1 in phase_range.clone() {
        'p2: for phase2 in phase_range.clone() {
            'p3: for phase3 in phase_range.clone() {
                'p4: for phase4 in phase_range.clone() {
                    'p5: for phase5 in phase_range.clone() {
                        let sequence = [phase1, phase2, phase3, phase4, phase5];
                        let mut hash = HashSet::new();
                        for i in sequence.iter() {
                            if !hash.insert(*i) {
                                continue 'p5;
                            }
                        }
                        let computed_putput = if cfg!(feature="part2") {
                            feedback_loop(sequence.iter().map(|x| *x), &program)?
                        } else {
                            compute_chain(sequence.iter().map(|x| *x), &program)?
                        };
                        println!("{:?} = {}", sequence, computed_putput);
                        if computed_putput > max_output {
                            max_output = computed_putput;
                        }
                    }
                }
            }
        }
    }

    println!("{}", max_output);
    Ok(())
}
