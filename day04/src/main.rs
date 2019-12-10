use std::io::{self, BufRead};
use std::str::FromStr;

fn stdin_range() -> (usize, usize) {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut line = String::new();
    reader.read_line(&mut line);

    let mut split = line.split('-');
    let lower = usize::from_str(split.next().unwrap()).unwrap();
    let higher = usize::from_str(split.next().unwrap()).unwrap();

    (lower, higher)
}



fn main() {
    println!("Hello, world!");
}
