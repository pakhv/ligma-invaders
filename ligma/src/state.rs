use crate::game::{MAX_X, MAX_Y, MIN_X, MIN_Y, MS_PER_UPDATE};
use std::{cmp, time::SystemTime};

#[derive(Debug)]
pub struct State {
    pub player: Player,
    prototypes: Prototypes,
}

#[derive(Debug)]
struct Prototypes {
    lazer: Vec<Coord>,
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

impl State {
    pub fn new() -> State {
        let player_model = include_str!("./assets/player.txt");
        let lazer_model = include_str!("./assets/lazer.txt");

        let player_prototype = parse_prototype(player_model);
        let lazer_prototype = parse_prototype(lazer_model);

        State {
            player: Player {
                health: 3,
                position: shift_prototype(&player_prototype, 1, 40),
                lazer: None,
            },
            prototypes: Prototypes {
                lazer: lazer_prototype,
            },
        }
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
}

fn parse_prototype(content: &str) -> Vec<Coord> {
    content
        .lines()
        .map(|l| {
            let mut parts = l.split_whitespace();
            let x = u16::from_str_radix(parts.next().expect("error parsing assets content"), 10)
                .expect("error converting to u16");
            let y = u16::from_str_radix(parts.next().expect("error parsing assets content"), 10)
                .expect("error converting to u16");
            let chars = parts
                .next()
                .expect("error parsing assets content")
                .chars()
                .collect::<Vec<char>>();

            let ch = chars.get(0).expect("");

            Coord { x, y, ch: *ch }
        })
        .collect()
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
