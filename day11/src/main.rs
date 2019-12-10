use anyhow::{self, bail, format_err, Error, Result};
use intcode::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::cmp::{max, min};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use Event::*;

#[derive(FromPrimitive)]
enum Rotation {
    Left90 = 0,
    Right90 = 1,
}

impl TryFrom<isize> for Rotation {
    type Error = Error;

    fn try_from(val: isize) -> Result<Self, Self::Error> {
        Rotation::from_usize(val as usize).ok_or_else(|| format_err!("Bad rotation value"))
    }
}

enum Direction {
    N,
    S,
    E,
    W,
}

impl Direction {
    fn rotate(&mut self, rotation: Rotation) {
        use Direction::*;
        *self = match rotation {
            Rotation::Left90 => match self {
                N => W,
                W => S,
                S => E,
                E => N,
            },
            Rotation::Right90 => match self {
                N => E,
                E => S,
                S => W,
                W => N,
            },
        }
    }

    fn mov(&self, coords: &mut (isize, isize)) {
        use Direction::*;
        match self {
            N => coords.1 += 1,
            E => coords.0 += 1,
            S => coords.1 -= 1,
            W => coords.0 -= 1,
        }
    }
}

#[derive(FromPrimitive, Clone, Copy)]
enum PanelColor {
    Black = 0,
    White = 1,
}

impl TryFrom<isize> for PanelColor {
    type Error = Error;

    fn try_from(val: isize) -> Result<Self, Self::Error> {
        PanelColor::from_usize(val as usize).ok_or_else(|| format_err!("Bad panel color"))
    }
}

struct Robot {
    direction: Direction,
    coords: (isize, isize),
}

impl Robot {
    fn rotate_and_move(&mut self, rotation: Rotation) {
        self.direction.rotate(rotation);
        self.direction.mov(&mut self.coords);
    }
}

fn main() -> Result<()> {
    let prog = first_arg_to_prog()?;
    let mut cpu = IntcodeComputer::new(prog);

    let mut ship = HashMap::new();
    ship.insert((0, 0), PanelColor::White);
    let mut robot = Robot {
        coords: (0, 0),
        direction: Direction::N,
    };

    let mut min_x = isize::max_value();
    let mut min_y = isize::max_value();
    let mut max_x = isize::min_value();
    let mut max_y = isize::min_value();

    loop {
        let panel_color = ship.entry(robot.coords).or_insert(PanelColor::Black);
        let color = match cpu.execute(Some(*panel_color as isize))? {
            HaveOutput(x) => x.try_into()?,
            Halted => break,
            _ => bail!("Unexpected color"),
        };

        *panel_color = color;

        let rotation = if let HaveOutput(x) = cpu.execute(Some(*panel_color as isize))? {
            x.try_into()?
        } else {
            bail!("Unexpected rotation")
        };

        robot.rotate_and_move(rotation);

        min_x = min(min_x, robot.coords.0);
        max_x = max(max_x, robot.coords.0);
        min_y = min(min_y, robot.coords.1);
        max_y = max(max_y, robot.coords.1);
    }

    println!("{}", ship.len());
    println!("min ({}, {}) max ({}, {})", min_x, min_y, max_x, max_y);

    let height = max_y - min_y;
    let width = max_x - min_x;
    let mut screen = vec![".".repeat(width as usize + 1).into_bytes(); height as usize + 1];

    for (coord, color) in ship {
        screen[(coord.1 - min_y) as usize][(coord.0 - min_x) as usize] = match color {
            PanelColor::Black => b'.',
            PanelColor::White => b'#',
        };
    }

    for bytes in screen.into_iter().rev() {
        println!("{}", String::from_utf8(bytes)?);
    }
    Ok(())
}
