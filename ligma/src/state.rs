use crate::{
    game::{MAX_X, MAX_Y, MIN_X, MIN_Y, MS_PER_UPDATE},
    ligma_result::LigmaResult,
};
use std::{cmp, time::SystemTime};

#[derive(Debug)]
pub struct State {
    pub player: Player,
    pub aliens: Aliens,
    prototypes: Prototypes,
}

#[derive(Debug)]
struct Prototypes {
    lazer: Vec<Coord>,
    funny_alien: Vec<Coord>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Coord {
    pub x: u16,
    pub y: u16,
    pub ch: char,
}

#[derive(Debug)]
pub struct Player {
    pub health: usize,
    pub position: Vec<Coord>,
    pub lazer: Option<Lazer>,
}

#[derive(Debug)]
pub struct Alien {
    pub position: Vec<Coord>,
}

#[derive(Debug)]
pub struct Aliens {
    pub positions: Vec<Alien>,
    last_update: SystemTime,
    times_slower_than_cycle: u128,
    direction: AlienDirection,
}

#[derive(Debug, Clone)]
pub struct Lazer {
    pub position: Vec<Coord>,
    pub direction: LazerDirection,
    last_update: SystemTime,
    times_slower_than_cycle: u128,
}

#[derive(Debug, Clone)]
pub enum LazerDirection {
    Up,
    Down,
}

#[derive(Debug, Clone)]
pub enum AlienDirection {
    Left,
    Right,
}

const U16_CONVERT_ERROR: &str = "error converting coords to u16 while parsing assets";
const ASSETS_CHAR_ERROR: &str = "error getting char while parsing assets";
const ASSETS_PARSING_ERROR: &str = "error parsing assets content";

impl State {
    pub fn new() -> LigmaResult<State> {
        let player_model = include_str!("./assets/player.txt");
        let lazer_model = include_str!("./assets/lazer.txt");
        let funny_alien_model = include_str!("./assets/alien1.txt");

        let player_prototype = parse_prototype(player_model)?;
        let lazer_prototype = parse_prototype(lazer_model)?;
        let funny_alien_prototype = parse_prototype(funny_alien_model)?;

        let aliens_1 = generate_row_of_aliens(&funny_alien_prototype, 1, 10, 11);

        Ok(State {
            player: Player {
                health: 3,
                position: shift_prototype(&player_prototype, 1, 40),
                lazer: None,
            },
            aliens: Aliens {
                positions: aliens_1,
                last_update: SystemTime::now(),
                times_slower_than_cycle: 100,
                direction: AlienDirection::Left,
            },
            prototypes: Prototypes {
                lazer: lazer_prototype,
                funny_alien: funny_alien_prototype,
            },
        })
    }

    pub fn player_go_left(&mut self) {
        self.player.go_left();
    }

    pub fn player_go_right(&mut self) {
        self.player.go_right();
    }

    pub fn player_shoot(&mut self) {
        if self.player.lazer.is_some() {
            return;
        }

        self.player.shoot(&self.prototypes.lazer);
    }

    pub fn update_player_lazer(&mut self) {
        let lazer = self.player.lazer.as_mut();

        if lazer.is_none() {
            return;
        }

        let lazer = lazer.unwrap();

        if lazer.last_update.elapsed().unwrap().as_millis()
            < lazer.times_slower_than_cycle * MS_PER_UPDATE
        {
            return;
        }

        lazer.last_update = SystemTime::now();

        let shift: i16 = match lazer.direction {
            LazerDirection::Up => -1,
            LazerDirection::Down => 1,
        };

        if lazer
            .position
            .iter()
            .find(|p| p.y as i16 + shift < MIN_Y as i16 || p.y as i16 + shift > MAX_Y as i16)
            .is_some()
        {
            self.player.lazer = None;
            return;
        }

        lazer.position = lazer
            .position
            .iter()
            .map(|p| Coord {
                x: p.x,
                y: (p.y as i16 + shift) as u16,
                ch: p.ch,
            })
            .collect();
    }

    pub fn update_aliens(&mut self) {
        self.aliens.update();
    }
}

impl Player {
    fn shoot(&mut self, prototype: &Vec<Coord>) {
        let tip_position = self.position.first().unwrap();
        let position = shift_prototype(prototype, tip_position.x, tip_position.y - 2);

        self.lazer = Some(Lazer {
            position,
            direction: LazerDirection::Up,
            last_update: SystemTime::now(),
            times_slower_than_cycle: 2,
        })
    }

    fn go_left(&mut self) {
        if self.position.iter().find(|p| p.x <= MIN_X).is_some() {
            return;
        }

        self.shift_by(-3, 0);
    }

    fn go_right(&mut self) {
        if self.position.iter().find(|p| p.x >= MAX_X).is_some() {
            return;
        }

        self.shift_by(3, 0);
    }

    fn shift_by(&mut self, x_shift: i16, y_shift: i16) {
        self.position = self
            .position
            .iter()
            .map(|c| Coord {
                x: (cmp::max(c.x as i16 + x_shift, 0)) as u16,
                y: (cmp::max(c.y as i16 + y_shift, 0)) as u16,
                ch: c.ch,
            })
            .collect();
    }
}

impl Aliens {
    fn update(&mut self) {
        if self.last_update.elapsed().unwrap().as_millis()
            < self.times_slower_than_cycle * MS_PER_UPDATE
        {
            return;
        }

        self.last_update = SystemTime::now();

        let x_shift: i16 = 1;
        let y_shift: i16 = 2;

        match self.direction {
            AlienDirection::Left => {
                let change_direction = self.change_direction(-x_shift, MIN_X as i16);

                if change_direction {
                    self.direction = AlienDirection::Right;
                    self.shift_aliens(0, y_shift);

                    return;
                }

                self.shift_aliens(-x_shift, 0);
            }
            AlienDirection::Right => {
                let change_direction = self.change_direction(x_shift, MAX_X as i16);

                if change_direction {
                    self.direction = AlienDirection::Left;
                    self.shift_aliens(0, y_shift);

                    return;
                }

                self.shift_aliens(x_shift, 0);
            }
        };
    }

    fn shift_aliens(&mut self, x_shift: i16, y_shift: i16) {
        for alien in self.positions.iter_mut() {
            alien.position.iter_mut().for_each(|p| {
                p.x = (p.x as i16 + x_shift) as u16;
                p.y = (p.y as i16 + y_shift) as u16;
            });
        }
    }

    fn change_direction(&self, x_shift: i16, terminal_value: i16) -> bool {
        let comparer = if x_shift > 0 {
            |left: i16, right: i16| left > right
        } else {
            |left: i16, right: i16| left <= right
        };

        self.positions
            .iter()
            .map(|a| {
                if x_shift > 0 {
                    a.position.last()
                } else {
                    a.position.first()
                }
            })
            .filter(|a| a.is_some())
            .map(|a| a.unwrap())
            .any(|a| comparer(a.x as i16 + x_shift, terminal_value))
    }
}

fn generate_row_of_aliens(
    alien_prototype: &Vec<Coord>,
    init_x: u16,
    init_y: u16,
    number: u16,
) -> Vec<Alien> {
    let step: u16 = 9;

    (0..number - 1)
        .map(|i| {
            alien_prototype
                .iter()
                .map(|a| Coord {
                    x: a.x + init_x + i * step,
                    y: a.y + init_y,
                    ch: a.ch,
                })
                .collect::<Vec<_>>()
        })
        .map(|p| Alien { position: p })
        .collect::<Vec<_>>()
}

fn parse_prototype(content: &str) -> LigmaResult<Vec<Coord>> {
    let mut buffer = vec![];

    for line in content.lines() {
        let mut parts = line.split_whitespace();
        let x = u16::from_str_radix(parts.next().ok_or(ASSETS_PARSING_ERROR)?, 10)
            .map_err(|_| U16_CONVERT_ERROR)?;
        let y = u16::from_str_radix(parts.next().ok_or(ASSETS_PARSING_ERROR)?, 10)
            .map_err(|_| U16_CONVERT_ERROR)?;
        let chars = parts
            .next()
            .ok_or(ASSETS_PARSING_ERROR)?
            .chars()
            .collect::<Vec<char>>();

        let ch = chars.get(0).ok_or(ASSETS_CHAR_ERROR)?;

        buffer.push(Coord { x, y, ch: *ch })
    }

    Ok(buffer)
}

fn shift_prototype(prototype: &Vec<Coord>, x_shift: u16, y_shift: u16) -> Vec<Coord> {
    prototype
        .iter()
        .map(|c| Coord {
            x: c.x + x_shift,
            y: c.y + y_shift,
            ch: c.ch,
        })
        .collect()
}
