use anyhow::Result;
use std::io::{self, prelude::*};
use std::str::FromStr;

fn load_masses(reader: impl Read) -> impl Iterator<Item = Result<String>> {
    io::BufReader::new(reader)
        .lines()
        .map(|line| -> Result<String> { Ok(line?) })
}

fn convert_mass_str(mass: &str) -> Result<u64> {
    Ok(u64::from_str(mass)?)
}

fn convert_mass_to_fuel(mass: u64) -> u64 {
    (mass / 3).saturating_sub(2)
}

fn fuel_for_fuel(mut fuel: u64) -> u64 {
    let mut sum = fuel;

    loop {
        fuel = convert_mass_to_fuel(fuel);
        if fuel != 0 {
            sum += fuel;
        } else {
            break sum
        }
    }
}

fn main() -> Result<()> {
    let stdin = std::io::stdin();
    let masses = load_masses(stdin.lock());

    let total = masses
        .map(|mass| -> Result<u64> {
            let mass = convert_mass_str(&mass?)?;
            let fuel = convert_mass_to_fuel(mass);
            if cfg!(feature="part2") {
                Ok(fuel_for_fuel(fuel))
            } else {
                Ok(fuel)
            }
        })
        .sum::<Result<u64>>()?;

    println!("Total is {}", total);

    Ok(())
}
