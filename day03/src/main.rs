#[macro_use]
extern crate bitflags;

use anyhow;
use std::collections::HashMap;
use std::io::{self, BufRead};
use std::str::FromStr;

bitflags! {
    struct Wire: u8 {
        const WIRE1 = 0b01;
        const WIRE2 = 0b10;
    }
}

impl Wire {
    fn index(&self) -> usize {
        (self.bits - 1) as usize
    }
}

#[derive(Debug)]
enum Direction {
    U(usize),
    D(usize),
    L(usize),
    R(usize),
}

impl Direction {
    fn step_coordinate(&self, coordinate: &mut (isize, isize)) {
        use Direction::*;
        match self {
            U(_) => coordinate.1 += 1,
            D(_) => coordinate.1 -= 1,
            L(_) => coordinate.0 -= 1,
            R(_) => coordinate.0 += 1,
        }
    }

    // Pop pop!
    fn magnitude(&self) -> usize {
        use Direction::*;
        *match self {
            U(x) => x,
            D(x) => x,
            L(x) => x,
            R(x) => x,
        }
    }
}

impl FromStr for Direction {
    type Err = anyhow::Error;
    fn from_str(string: &str) -> anyhow::Result<Direction> {
        use Direction::*;
        let num = usize::from_str(&string[1..])?;
        Ok(match &string[0..1] {
            "U" => U(num),
            "D" => D(num),
            "L" => L(num),
            "R" => R(num),
            dir => anyhow::bail!("Unknown direction {}", dir),
        })
    }
}

/// Keeps track of which wires have visited a coordinate
/// and how many steps it took each one to get there.
struct WireStatus {
    visits: Wire,
    steps: [usize; 2],
}

impl WireStatus {
    /// Create a new WireStatus from a given wire,
    /// storing its step count.
    fn new_from(wire: Wire, step_count: usize) -> WireStatus {
        let mut steps = [usize::max_value(); 2];
        steps[wire.index()] = step_count;
        WireStatus {
            visits: wire,
            steps,
        }
    }
    /// Mark that the given wire has visited.
    /// If this wire has been to this point before, then it resets the
    /// step count of the current run.
    /// Returns true if both wires have now visited this point.
    fn visit_from(&mut self, wire: Wire, new_steps: &mut usize) {
        use std::cmp::Ordering;
        self.visits |= wire;

        let current_steps = &mut self.steps[wire.bits as usize - 1];
        match current_steps.cmp(&new_steps) {
            Ordering::Less => *new_steps = *current_steps,
            // this branch should only happen once, when it's been visited the first time.
            Ordering::Greater => *current_steps = *new_steps,
            _ => {}
        }
    }

    fn is_crossed(&self) -> bool {
        self.visits.is_all()
    }

    // The combined length of the wire paths that visited this crossed point.
    // Asserts that it has indeed been crossed.
    fn total_length(&self) -> usize {
        assert!(self.visits.is_all());
        self.steps[0] + self.steps[1]
    }
}

/// The wiring itself.
struct Wiring {
    /// Points that have been visited.
    wiring: HashMap<(isize, isize), WireStatus>,
    /// The currrent closest crossing via manhattan distance.
    closest_crossing: (isize, isize),
    /// The distance to that crossing via manhattan distance.
    dist: usize,
    /// The current shortest length to a wire crossing.
    length: usize,
}

impl Wiring {
    fn new() -> Wiring {
        Wiring {
            wiring: HashMap::new(),
            closest_crossing: (isize::max_value() / 2, isize::max_value() / 2),
            dist: usize::max_value(),
            length: usize::max_value(),
        }
    }

    fn set_wire(&mut self, coord: (isize, isize), wire_number: Wire, steps: &mut usize) {
        let wire_status = self
            .wiring
            .entry(coord)
            .or_insert_with(|| WireStatus::new_from(wire_number, *steps));

        wire_status.visit_from(wire_number, steps);

        if wire_status.is_crossed() {
            let dist = (coord.0.abs() + coord.1.abs()) as usize;
            if dist < self.dist {
                self.closest_crossing = coord;
                self.dist = dist;
            }
            let length = wire_status.total_length();
            if length < self.length {
                self.length = length;
            }
        }
    }

    fn run_wire(&mut self, wire_number: Wire, wire: impl IntoIterator<Item = Direction>) {
        let mut current_coordinate = (0, 0);
        let mut steps = 0;

        for dir in wire {
            for _ in 0..dir.magnitude() {
                dir.step_coordinate(&mut current_coordinate);
                steps += 1;
                self.set_wire(current_coordinate, wire_number, &mut steps);
            }
        }
    }
}

fn line_to_directions(line: &str) -> anyhow::Result<Vec<Direction>> {
    line.split(',').map(Direction::from_str).collect()
}

fn main() -> anyhow::Result<()> {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let one = lines.next().unwrap()?;
    let two = lines.next().unwrap()?;

    let path1 = line_to_directions(&one)?;
    let path2 = line_to_directions(&two)?;

    let mut wiring = Wiring::new();

    wiring.run_wire(Wire::WIRE1, path1);
    wiring.run_wire(Wire::WIRE2, path2);

    println!("Distance: {}\nLength: {}", wiring.dist, wiring.length);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test {
        ($name:ident ($wire1:expr, $wire2:expr) -> $dist:expr $(, $length:expr)? ) => {
            #[test]
            fn $name() {
                let wire1_path = line_to_directions($wire1).unwrap();
                let wire2_path = line_to_directions($wire2).unwrap();

                let mut wiring = Wiring::new();

                wiring.run_wire(Wire::WIRE1, wire1_path);
                wiring.run_wire(Wire::WIRE2, wire2_path);

                assert_eq!(wiring.dist, $dist);
                $( assert_eq!(wiring.length, $length) )?
            }
        };
    }

    test!(main_example ("R8,U5,L5,D3", "U7,R6,D4,L4") -> 6);
    test!(smol_1 ("R75,D30,R83,U83,L12,D49,R71,U7,L72", "U62,R66,U55,R34,D71,R55,D58,R83") -> 159, 610);
    test!(smol_2 ("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51", "U98,R91,D20,R16,D67,R40,U7,R15,U6,R7") -> 135, 410);
}
