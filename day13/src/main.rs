use anyhow::{self, bail, format_err, Error, Result};
use intcode::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
// use cursive::{
//     self,
//     event::{self, EventResult},
//     view::View,
//     Cursive, Printer, Vec2,
// };
use std::collections::HashMap;
use std::collections::VecDeque;
use std::convert::{TryFrom, TryInto};

#[derive(FromPrimitive, PartialEq, Debug)]
enum Tile {
    Empty = 0,
    Wall = 1,
    Block = 2,
    Paddle = 3,
    Ball = 4,
}

impl TryFrom<isize> for Tile {
    type Error = Error;

    fn try_from(val: isize) -> Result<Self, Self::Error> {
        Tile::from_usize(val as usize).ok_or_else(|| format_err!("Unknown tile type"))
    }
}

// impl From<Tile> for &'static str {
//     fn from(tile: Tile) -> &'static str {
//         use Tile::*;
//         match tile {
//             Empty => " ",
//             Wall => "|",
//             Block => "#",
//             Paddle => "=",
//             Ball => "o",
//         }
//     }
// }

// enum JoystickMovement {
//     Left = -1,
//     Neutral = 0,
//     Right = 1,
// }

// impl TryFrom<event::Event> for JoystickMovement {
//     type Error = ();
//     fn try_from(event: event::Event) -> Result<Self, Self::Error> {
//         use event::Event::*;
//         use event::Key::*;
//         match event {
//             Key(Right) => Ok(JoystickMovement::Right),
//             Key(Left) => Ok(JoystickMovement::Left),
//             Char(' ') => Ok(JoystickMovement::Neutral),
//             _ => Err(()),
//         }
//     }
// }

// struct Draw(isize, isize, Tile);

// enum GameEvent {
//     Draw(Draw),
//     GameOver,
//     RequestingInput,
// }

// struct Game {
//     cpu: IntcodeComputer,
//     score: isize,
//     // input_buffer: Option<JoystickMovement>,
//     draw_buffer: VecDeque<Draw>,
// }

// impl Game {
//     fn new(mut cpu: IntcodeComputer) -> Result<Game> {
//         let mut game = Game {
//             cpu,
//             score: 0,
//             // input_buffer: None,
//             draw_buffer: VecDeque::new(),
//         };

//         loop {
//             match game.input_execute(None)? {
//                 GameEvent::Draw(draw) => game.draw_buffer.push_back(draw),
//                 GameEvent::GameOver => bail!("unexpected game over"),
//                 GameEvent::RequestingInput => break Ok(game),
//             }
//         }
//     }

//     fn input_execute(&mut self, input: Option<isize>) -> Result<GameEvent> {
//         loop {
//             match self.execute(input)? {
//                 GameEvent::Draw(draw) => self.draw_buffer.push_back(draw),
//                 x => break Ok(x),
//             }
//         }
//     }

//     fn draw(&mut self, printer: &Printer) {
//         while let Some(Draw(x, y, tile)) = self.draw_buffer.pop_front() {
//             let x = x.try_into().unwrap();
//             let y = y.try_into().unwrap();

//             let coords = Vec2 { x, y };
//             let tile = tile.into();
//             printer.print(coords, tile);
//         }
//     }

//     fn execute(&mut self, input: Option<isize>) -> Result<GameEvent> {
//         use Event::*;
//         loop {
//             let x = match self.cpu.execute(input)? {
//                 HaveOutput(x) => x,
//                 RequestingInput => return Ok(GameEvent::RequestingInput),
//                 Halted => return Ok(GameEvent::GameOver),
//             };

//             match (self.cpu.execute(None)?, self.cpu.execute(None)?) {
//                 (HaveOutput(y), HaveOutput(tile)) => {
//                     if x == -1 && y == 0 {
//                         self.score = tile;
//                     } else {
//                         return Ok(GameEvent::Draw(Draw(x, y, Tile::try_from(tile)?)));
//                     }
//                 }
//                 _ => bail!("Unexpected output"),
//             }
//         }
//     }
// }

// impl View for Game {
//     fn draw(&self, printer: &Printer) {}

//     fn on_event(&mut self, event: event::Event) -> EventResult {
//         let input = if let Ok(movement) = JoystickMovement::try_from(event) {
//             movement
//         } else {
//             return EventResult::Ignored;
//         };

//         let _evt = self.input_execute(Some(input as isize)).unwrap();

//         EventResult::Consumed(None)
//     }
// }

fn hack_quarters(prog: &mut Vec<isize>) {
    prog[0] = 2;
}

fn none() -> Option<isize> {
    None
}

fn part_1(mut cpu: IntcodeComputer) -> Result<()> {
    use intcode::Event::*;
    let mut screen = HashMap::new();
    let mut score = 0;

    loop {
        let x = match cpu.execute(&mut none)? {
            HaveOutput(x) => x,
            Halted => break,
            _ => bail!("unexpected output"),
        };

        match (cpu.execute(&mut none)?, cpu.execute(&mut none)?) {
            (HaveOutput(y), HaveOutput(tile)) => {
                if x == -1 && y == 0 {
                    score = tile;
                } else {
                    screen.insert((x, y), Tile::try_from(tile)?);
                }
            }
            _ => bail!("Unexpected output"),
        }
    }

    let block_count = screen
        .into_iter()
        .filter(|(_, tile)| *tile == Tile::Block)
        .count();
    println!("{}", block_count);
    Ok(())
}

struct Game(IntcodeComputer);

enum GameEvent {
    UpdateScore(isize),
    BallPos(isize),
    PaddlePos(isize),
    Halted
}

impl Game {
    fn execute(&mut self, input: &mut dyn FnMut() -> Option<isize>) -> Result<GameEvent> {
        use Event::*;
        loop {
            let x = match self.0.execute(input)? {
                HaveOutput(x) => x,
                Halted => break Ok(GameEvent::Halted),
                _ => bail!("unexpected output"),
            };
    
            match (self.0.execute(input)?, self.0.execute(input)?) {
                (HaveOutput(y), HaveOutput(tile)) => {
                    if x == -1 && y == 0 {
                        break Ok(GameEvent::UpdateScore(tile))
                    } else {
                        match Tile::try_from(tile)? {
                            Tile::Ball => break Ok(GameEvent::BallPos(x)),
                            Tile::Paddle => break Ok(GameEvent::PaddlePos(x)),
                            _ => continue,
                        }
                    }
                }
                _ => bail!("Unexpected output"),
            }
        }
    }
}
fn part_2(mut cpu: IntcodeComputer) -> Result<()> {
    hack_quarters(&mut cpu.memory);
    let mut game = Game(cpu);
    
    let mut score = 0;
    let mut paddle_x: Option<isize> = None;
    let mut ball_x: Option<isize> = None;
    loop {
        let mut input = || { Some(if let (Some(paddle_x), Some(ball_x)) = (paddle_x, ball_x) {
            (paddle_x - ball_x).signum()
        } else {
            0
        }) };
        match game.execute(&mut input)? {
            GameEvent::BallPos(x) => {
                ball_x = Some(x);
            }
            GameEvent::PaddlePos(x) => {
                paddle_x = Some(x);
            }
            GameEvent::UpdateScore(x) => {
                score = x;
            }
            GameEvent::Halted => {
                println!("{}", score);
                break
            }
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let mut prog = first_arg_to_prog()?;
    let cpu = IntcodeComputer::new(prog);

    if !cfg!(feature = "part2") {
        part_1(cpu)?;
    } else {
        part_2(cpu)?;
    }
    Ok(())
}
