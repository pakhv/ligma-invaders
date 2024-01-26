use std::{cmp, time::SystemTime};

use crate::game::{MAX_X, MAX_Y, MIN_X, MIN_Y, MS_PER_UPDATE};

#[derive(Debug)]
pub struct State {
    pub player: Player,
    //pub enemies: Vec<Player>,
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

impl Player {
    pub fn create_player(x: u16, y: u16) -> Player {
        let position = vec![
            Coord {
                x: 4 + x,
                y: 0 + y,
                ch: '─',
            },
            Coord {
                x: 3 + x,
                y: 0 + y,
                ch: '┌',
            },
            Coord {
                x: 5 + x,
                y: 0 + y,
                ch: '─',
            },
            Coord {
                x: 6 + x,
                y: 0 + y,
                ch: '┐',
            },
            Coord {
                x: 0 + x,
                y: 1 + y,
                ch: '┌',
            },
            Coord {
                x: 1 + x,
                y: 1 + y,
                ch: '─',
            },
            Coord {
                x: 2 + x,
                y: 1 + y,
                ch: '─',
            },
            Coord {
                x: 3 + x,
                y: 1 + y,
                ch: '┘',
            },
            Coord {
                x: 4 + x,
                y: 1 + y,
                ch: ' ',
            },
            Coord {
                x: 5 + x,
                y: 1 + y,
                ch: ' ',
            },
            Coord {
                x: 6 + x,
                y: 1 + y,
                ch: '└',
            },
            Coord {
                x: 7 + x,
                y: 1 + y,
                ch: '─',
            },
            Coord {
                x: 8 + x,
                y: 1 + y,
                ch: '─',
            },
            Coord {
                x: 9 + x,
                y: 1 + y,
                ch: '┐',
            },
            Coord {
                x: 0 + x,
                y: 2 + y,
                ch: '┃',
            },
            Coord {
                x: 1 + x,
                y: 2 + y,
                ch: ' ',
            },
            Coord {
                x: 2 + x,
                y: 2 + y,
                ch: ' ',
            },
            Coord {
                x: 3 + x,
                y: 2 + y,
                ch: ' ',
            },
            Coord {
                x: 4 + x,
                y: 2 + y,
                ch: ' ',
            },
            Coord {
                x: 5 + x,
                y: 2 + y,
                ch: ' ',
            },
            Coord {
                x: 6 + x,
                y: 2 + y,
                ch: ' ',
            },
            Coord {
                x: 7 + x,
                y: 2 + y,
                ch: ' ',
            },
            Coord {
                x: 8 + x,
                y: 2 + y,
                ch: ' ',
            },
            Coord {
                x: 9 + x,
                y: 2 + y,
                ch: '┃',
            },
            Coord {
                x: 0 + x,
                y: 3 + y,
                ch: '└',
            },
            Coord {
                x: 1 + x,
                y: 3 + y,
                ch: '─',
            },
            Coord {
                x: 2 + x,
                y: 3 + y,
                ch: '─',
            },
            Coord {
                x: 3 + x,
                y: 3 + y,
                ch: '─',
            },
            Coord {
                x: 4 + x,
                y: 3 + y,
                ch: '─',
            },
            Coord {
                x: 5 + x,
                y: 3 + y,
                ch: '─',
            },
            Coord {
                x: 6 + x,
                y: 3 + y,
                ch: '─',
            },
            Coord {
                x: 7 + x,
                y: 3 + y,
                ch: '─',
            },
            Coord {
                x: 8 + x,
                y: 3 + y,
                ch: '─',
            },
            Coord {
                x: 9 + x,
                y: 3 + y,
                ch: '┘',
            },
        ];

        Player {
            health: 3,
            position,
            lazer: None,
        }
    }

    pub fn shoot(&mut self) {
        if self.lazer.is_some() {
            return;
        }

        let tip_position = self.position.first().unwrap();

        self.lazer = Some(Lazer {
            position: vec![
                Coord {
                    x: tip_position.x,
                    y: tip_position.y - 1,
                    ch: '┇',
                },
                Coord {
                    x: tip_position.x,
                    y: tip_position.y - 2,
                    ch: '┇',
                },
            ],
            direction: LazerDirection::Up,
            last_update: SystemTime::now(),
            times_slower_than_cycle: 2,
        })
    }

    pub fn go_left(&mut self) {
        if self.position.iter().find(|p| p.x <= MIN_X).is_some() {
            return;
        }

        self.shift_by(-3, 0);
    }

    pub fn go_right(&mut self) {
        if self.position.iter().find(|p| p.x >= MAX_X).is_some() {
            return;
        }

        self.shift_by(3, 0);
    }

    pub fn update_lazer(&mut self) {
        let lazer = self.lazer.as_mut();

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
            self.lazer = None;
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
