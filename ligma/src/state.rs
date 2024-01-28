use crate::{
    game::{MS_PER_UPDATE, VIEWPORT_MAX_X, VIEWPORT_MAX_Y, VIEWPORT_MIN_X, VIEWPORT_MIN_Y},
    ligma_result::LigmaResult,
};
use std::time::{Duration, SystemTime};

#[derive(Debug)]
pub struct State {
    pub player: Player,
    pub aliens: Aliens,
    prototypes: Prototypes,
}

#[derive(Debug)]
struct Prototypes {
    laser: Vec<Coord>,
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
    pub laser: Option<Laser>,
}

#[derive(Debug)]
pub struct Alien {
    pub position: Vec<Coord>,
}

#[derive(Debug)]
pub struct AliensRow {
    pub aliens: Vec<Alien>,
    last_update: SystemTime,
}

#[derive(Debug)]
pub struct Aliens {
    pub aliens_rows: Vec<AliensRow>,
    times_slower_than_cycle: u128,
    direction: AlienDirection,
}

#[derive(Debug, Clone)]
pub struct Laser {
    pub position: Vec<Coord>,
    pub direction: LaserDirection,
    last_update: SystemTime,
    times_slower_than_cycle: u128,
}

#[derive(Debug, Clone)]
pub enum LaserDirection {
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
        let laser_model = include_str!("./assets/laser.txt");

        let player_prototype = parse_prototype(player_model)?;
        let laser_prototype = parse_prototype(laser_model)?;

        Ok(State {
            player: Player {
                health: Player::HEALTH,
                position: shift_prototype(&player_prototype, Player::INITIAL_X, Player::INITIAL_Y),
                laser: None,
            },
            aliens: Aliens::init()?,
            prototypes: Prototypes {
                laser: laser_prototype,
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
        if self.player.laser.is_some() {
            return;
        }

        self.player.shoot(&self.prototypes.laser);
    }

    pub fn update_player_laser(&mut self) {
        let laser = self.player.laser.as_mut();

        if laser.is_none() {
            return;
        }

        let laser = laser.unwrap();

        if laser.last_update.elapsed().unwrap().as_millis()
            < laser.times_slower_than_cycle * MS_PER_UPDATE
        {
            return;
        }

        laser.last_update = SystemTime::now();

        let shift: i16 = match laser.direction {
            LaserDirection::Up => -Player::LASER_SPEED,
            LaserDirection::Down => Player::LASER_SPEED,
        };

        if laser
            .position
            .iter()
            .find(|p| {
                p.y as i16 + shift < VIEWPORT_MIN_Y as i16
                    || p.y as i16 + shift > VIEWPORT_MAX_Y as i16
            })
            .is_some()
        {
            self.player.laser = None;
            return;
        }

        laser.position.iter_mut().for_each(|p| {
            p.y = (p.y as i16 + shift) as u16;
        });
    }

    pub fn update_aliens(&mut self) {
        self.aliens.update();
    }
}

impl Player {
    const SPEED: i16 = 3;
    const LASER_SPEED: i16 = 1;
    const HEALTH: usize = 3;
    const INITIAL_X: u16 = 1;
    const INITIAL_Y: u16 = 40;
    const LASER_SLOWER_THAN_CYCLE: u128 = 2;

    fn shoot(&mut self, prototype: &Vec<Coord>) {
        let tip_position = self.position.first().unwrap();
        let position = shift_prototype(
            prototype,
            tip_position.x,
            tip_position.y - Laser::MODEL_HEIGHT,
        );

        self.laser = Some(Laser {
            position,
            direction: LaserDirection::Up,
            last_update: SystemTime::now(),
            times_slower_than_cycle: Self::LASER_SLOWER_THAN_CYCLE,
        })
    }

    fn go_left(&mut self) {
        if self
            .position
            .iter()
            .find(|p| p.x <= VIEWPORT_MIN_X)
            .is_some()
        {
            return;
        }

        self.shift_by(-Self::SPEED);
    }

    fn go_right(&mut self) {
        if self
            .position
            .iter()
            .find(|p| p.x >= VIEWPORT_MAX_X)
            .is_some()
        {
            return;
        }

        self.shift_by(Self::SPEED);
    }

    fn shift_by(&mut self, x_shift: i16) {
        self.position.iter_mut().for_each(|c| {
            c.x = (c.x as i16 + x_shift) as u16;
        });
    }
}

impl Aliens {
    const INITIAL_X: u16 = 1;
    const INITIAL_Y: u16 = 10;
    const NUMBER: u16 = 11;
    const SLOWER_THAN_CYCLE: u128 = 10;
    const X_SHIFT_PER_UPDATE: i16 = 1;
    const Y_SHIFT_PER_UPDATE: i16 = 2;
    const ROWS_DELAY_SHIFT: u64 = 20;
    const ROWS_NUMBER: usize = 2;

    fn init() -> LigmaResult<Aliens> {
        let funny_alien_model = include_str!("./assets/alien1.txt");
        let funny_alien_prototype = parse_prototype(funny_alien_model)?;
        let step: usize = 4;

        let rows = (0..Self::ROWS_NUMBER)
            .map(|idx| {
                let row = generate_row_of_aliens(
                    &funny_alien_prototype,
                    Aliens::INITIAL_X,
                    Aliens::INITIAL_Y + (idx * step) as u16,
                    Aliens::NUMBER,
                );

                AliensRow {
                    aliens: row,
                    last_update: SystemTime::now()
                        + Duration::from_millis(
                            Aliens::ROWS_DELAY_SHIFT
                                * (Self::ROWS_NUMBER - 1 - idx) as u64
                                * MS_PER_UPDATE as u64,
                        ),
                }
            })
            .collect();

        Ok(Aliens {
            aliens_rows: rows,
            times_slower_than_cycle: Aliens::SLOWER_THAN_CYCLE,
            direction: AlienDirection::Right,
        })
    }

    fn update(&mut self) {
        for aliens_row in self.aliens_rows.iter_mut() {
            match aliens_row.last_update.elapsed() {
                Ok(elapsed) => {
                    if elapsed.as_millis() < self.times_slower_than_cycle * MS_PER_UPDATE {
                        continue;
                    }

                    if aliens_row.need_to_change_direction(self.direction.clone()) {
                        self.change_direction();
                        return;
                    }

                    let shift = match self.direction {
                        AlienDirection::Left => -Aliens::X_SHIFT_PER_UPDATE,
                        AlienDirection::Right => Aliens::X_SHIFT_PER_UPDATE,
                    };

                    aliens_row.shift_aliens(shift, 0);
                    aliens_row.last_update = SystemTime::now();
                }
                Err(_) => continue,
            };
        }
    }

    fn change_direction(&mut self) {
        self.direction = match self.direction {
            AlienDirection::Left => AlienDirection::Right,
            AlienDirection::Right => AlienDirection::Left,
        };

        self.aliens_rows
            .iter_mut()
            .enumerate()
            .for_each(|(idx, r)| {
                r.shift_aliens(0, Aliens::Y_SHIFT_PER_UPDATE);
                r.last_update = SystemTime::now()
                    + Duration::from_millis(
                        Self::ROWS_DELAY_SHIFT
                            * (Self::ROWS_NUMBER - 1 - idx) as u64
                            * MS_PER_UPDATE as u64,
                    );
            });
    }
}

impl AliensRow {
    fn shift_aliens(&mut self, x_shift: i16, y_shift: i16) {
        for alien in self.aliens.iter_mut() {
            alien.position.iter_mut().for_each(|p| {
                p.x = (p.x as i16 + x_shift) as u16;
                p.y = (p.y as i16 + y_shift) as u16;
            });
        }
    }

    fn need_to_change_direction(&self, direction: AlienDirection) -> bool {
        let (x_shift, terminal_value) = match direction {
            AlienDirection::Left => (-Aliens::X_SHIFT_PER_UPDATE, VIEWPORT_MIN_X as i16),
            AlienDirection::Right => (Aliens::X_SHIFT_PER_UPDATE, VIEWPORT_MAX_X as i16),
        };

        let comparer = if x_shift > 0 {
            |left: i16, right: i16| left > right
        } else {
            |left: i16, right: i16| left <= right
        };

        self.aliens
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

impl Laser {
    const MODEL_HEIGHT: u16 = 2;
}

fn generate_row_of_aliens(
    alien_prototype: &Vec<Coord>,
    init_x: u16,
    init_y: u16,
    number: u16,
) -> Vec<Alien> {
    let step: u16 = 9;

    (0..number)
        .map(|i| {
            alien_prototype
                .iter()
                .map(|a| Coord {
                    x: a.x + init_x + i * step,
                    y: a.y + init_y,
                    ch: a.ch,
                })
                .collect()
        })
        .map(|p| Alien { position: p })
        .collect()
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
