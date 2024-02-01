use crate::{
    game::{MS_PER_UPDATE, VIEWPORT_MAX_X, VIEWPORT_MAX_Y, VIEWPORT_MIN_X, VIEWPORT_MIN_Y},
    ligma_result::LigmaResult,
};
use rand::Rng;
use std::time::{Duration, SystemTime};

#[derive(Debug)]
pub struct State {
    pub player: Player,
    pub aliens: Aliens,
    pub bunkers: Bunkers,
    prototypes: Prototypes,
    player_color: RgbColor,
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
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

trait Row {
    fn generate(position: Vec<Coord>) -> Self;
}

#[derive(Debug)]
pub struct Alien {
    pub position: Vec<Coord>,
}

impl Row for Alien {
    fn generate(position: Vec<Coord>) -> Alien {
        Alien { position }
    }
}

#[derive(Debug)]
pub struct Bunker {
    pub position: Vec<Coord>,
}

impl Row for Bunker {
    fn generate(position: Vec<Coord>) -> Bunker {
        Bunker { position }
    }
}

#[derive(Debug)]
pub struct Bunkers {
    pub positions: Vec<Bunker>,
}

#[derive(Debug)]
pub struct AliensRow {
    pub aliens: Vec<Alien>,
    last_update: SystemTime,
}

#[derive(Debug)]
pub struct Aliens {
    pub aliens_rows: Vec<AliensRow>,
    pub lasers: Vec<Laser>,
    times_slower_than_cycle: u128,
    direction: AlienDirection,
    last_shot: SystemTime,
}

#[derive(Debug, Clone)]
pub struct Laser {
    pub position: Vec<Coord>,
    last_update: SystemTime,
    times_slower_than_cycle: u128,
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
            bunkers: Bunkers::init()?,
            prototypes: Prototypes {
                laser: laser_prototype,
            },
            player_color: RgbColor {
                r: 32,
                g: 255,
                b: 32,
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

        if laser
            .position
            .iter()
            .find(|p| p.y as i16 - Player::LASER_SPEED < VIEWPORT_MIN_Y as i16)
            .is_some()
        {
            self.player.laser = None;
            return;
        }

        laser.position.iter_mut().for_each(|p| {
            p.y = (p.y as i16 - Player::LASER_SPEED) as u16;
        });
    }

    pub fn update_aliens(&mut self) {
        self.aliens.update();
    }

    pub fn apply_collisions(&mut self) {
        if self.player.laser.is_none() {
            return;
        }

        let laser = self.player.laser.clone().unwrap().position;

        for bunker in self.bunkers.positions.iter_mut() {
            match bunker.remove_shot_positions(&laser) {
                true => {
                    self.player.laser = None;
                    return;
                }
                false => continue,
            }
        }

        for aliens_row in self.aliens.aliens_rows.iter_mut().rev() {
            for (idx, alien) in aliens_row.aliens.iter().enumerate() {
                let killed_alien = alien
                    .position
                    .iter()
                    .find(|&p| collides_with_laser(&laser, p));

                if let Some(_) = killed_alien {
                    aliens_row.aliens.remove(idx);
                    self.player.laser = None;
                    return;
                }
            }
        }
    }

    pub fn get_player_color(&self) -> &RgbColor {
        &self.player_color
    }

    pub fn update_aliens_lasers(&mut self) {
        self.aliens.update_existing_aliens_lasers();
        self.aliens
            .shoot(&self.player.position, &self.prototypes.laser);
    }
}

impl Player {
    const SPEED: i16 = 2;
    const LASER_SPEED: i16 = 1;
    const HEALTH: usize = 3;
    const INITIAL_X: u16 = 1;
    const INITIAL_Y: u16 = 40;
    const LASER_SLOWER_THAN_CYCLE: u128 = 1;

    fn shoot(&mut self, prototype: &Vec<Coord>) {
        let tip_position = self.position.first().unwrap();
        let position = shift_prototype(
            prototype,
            tip_position.x,
            tip_position.y - Laser::MODEL_HEIGHT,
        );

        self.laser = Some(Laser {
            position,
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
    const INITIAL_Y: u16 = 1;
    const NUMBER: u16 = 11;
    const SLOWER_THAN_CYCLE: u128 = 100;
    const X_SHIFT_PER_UPDATE: i16 = 1;
    const Y_SHIFT_PER_UPDATE: i16 = 2;
    const ROWS_DELAY_SHIFT: u64 = 20;
    const ROWS_NUMBER: usize = 5;
    const STEP: u16 = 14;
    const MAX_LASERS_AT_A_TIME: usize = 3;
    const LASER_SPEED: u16 = 2;
    const SHOTS_MIN_INTERVAL: u128 = 200;
    const LASER_SLOWER_THAN_CYCLE: u128 = 25;

    fn init() -> LigmaResult<Aliens> {
        let step: usize = 5;

        let squid_model = include_str!("./assets/squid.txt");
        let squid_prototype = parse_prototype(squid_model)?;

        let crab_model = include_str!("./assets/crab.txt");
        let crab_prototype = parse_prototype(crab_model)?;

        let octopus_model = include_str!("./assets/octopus.txt");
        let octopus_prototype = parse_prototype(octopus_model)?;

        let rows = [
            &squid_prototype,
            &crab_prototype,
            &crab_prototype,
            &octopus_prototype,
            &octopus_prototype,
        ]
        .iter()
        .enumerate()
        .map(|(idx, &prototype)| {
            let row = generate_row_of_aliens(
                prototype,
                Aliens::INITIAL_X,
                Aliens::INITIAL_Y + (idx * step) as u16,
                Aliens::NUMBER,
                Aliens::STEP,
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
            lasers: vec![],
            last_shot: SystemTime::now(),
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

    fn update_existing_aliens_lasers(&mut self) {
        for laser in self.lasers.iter_mut() {
            if laser.last_update.elapsed().unwrap().as_millis()
                < laser.times_slower_than_cycle * MS_PER_UPDATE
            {
                continue;
            }

            laser.last_update = SystemTime::now();

            laser.position.iter_mut().for_each(|p| {
                p.y = p.y + Aliens::LASER_SPEED;
            });
        }

        self.lasers
            .retain(|l| l.position.iter().find(|p| p.y < VIEWPORT_MAX_Y).is_some());
    }

    fn shoot(&mut self, player: &Vec<Coord>, laser_prototype: &Vec<Coord>) {
        if self.last_shot.elapsed().unwrap().as_millis() <= Self::SHOTS_MIN_INTERVAL * MS_PER_UPDATE
        {
            return;
        }

        let mut rng = rand::thread_rng();

        let mut closest_aliens = self
            .aliens_rows
            .iter()
            .flat_map(|r| &r.aliens)
            .map(|a| {
                (
                    a.position.first().unwrap(),
                    get_distance_between_positions(
                        a.position.first().unwrap(),
                        player.first().unwrap(),
                    ),
                )
            })
            .collect::<Vec<_>>();

        closest_aliens.sort_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap());

        for (position, _) in closest_aliens.iter().take(Aliens::MAX_LASERS_AT_A_TIME) {
            if self.lasers.len() >= Aliens::MAX_LASERS_AT_A_TIME {
                return;
            }

            if rng.gen_range(0..5) == 1 {
                let position = shift_prototype(
                    laser_prototype,
                    position.x,
                    position.y + Laser::MODEL_HEIGHT,
                );

                self.lasers.push(Laser {
                    position,
                    last_update: SystemTime::now(),
                    times_slower_than_cycle: Aliens::LASER_SLOWER_THAN_CYCLE,
                });

                self.last_shot = SystemTime::now();
                return;
            }
        }
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
        match self.aliens.len() {
            0 => false,
            _ => {
                match direction {
                    AlienDirection::Left => {
                        let alien_in_question = self.aliens.first().unwrap();
                        alien_in_question.position.iter().any(|p| {
                            p.x as i16 - Aliens::X_SHIFT_PER_UPDATE <= VIEWPORT_MIN_X as i16
                        })
                    }
                    AlienDirection::Right => {
                        let alien_in_question = self.aliens.last().unwrap();
                        alien_in_question.position.iter().any(|p| {
                            p.x as i16 + Aliens::X_SHIFT_PER_UPDATE > VIEWPORT_MAX_X as i16
                        })
                    }
                }
            }
        }
    }
}

impl Laser {
    const MODEL_HEIGHT: u16 = 2;
}

impl Bunkers {
    const INITIAL_X: u16 = 4;
    const INITIAL_Y: u16 = 33;
    const NUMBER: u16 = 4;
    const STEP: u16 = 40;

    fn init() -> LigmaResult<Bunkers> {
        let bunker_model = include_str!("./assets/bunker.txt");
        let bunker_prototype = parse_prototype(bunker_model)?;

        Ok(Bunkers {
            positions: generate_row_of_aliens(
                &bunker_prototype,
                Self::INITIAL_X,
                Self::INITIAL_Y,
                Self::NUMBER,
                Self::STEP,
            ),
        })
    }
}

impl Bunker {
    fn remove_shot_positions(&mut self, laser: &Vec<Coord>) -> bool {
        let shot_position = self
            .position
            .iter()
            .find(|p| collides_with_laser(laser, p))
            .cloned();

        match shot_position {
            Some(shot_position) => {
                self.position.retain(|p| {
                    !((p.x == shot_position.x
                        || p.x == shot_position.x - 1
                        || p.x == shot_position.x + 1)
                        && p.y == shot_position.y)
                });

                true
            }
            None => false,
        }
    }
}

fn generate_row_of_aliens<T: Row>(
    alien_prototype: &Vec<Coord>,
    init_x: u16,
    init_y: u16,
    number: u16,
    step: u16,
) -> Vec<T> {
    (0..number)
        .map(|i| T::generate(shift_prototype(alien_prototype, init_x + i * step, init_y)))
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
        let chars = parts.next().unwrap_or(" ").chars().collect::<Vec<char>>();

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

fn collides_with_laser(laser: &Vec<Coord>, coord: &Coord) -> bool {
    coord.x == laser[0].x && coord.y == laser[0].y || coord.x == laser[1].x && coord.y == laser[1].y
}

fn get_distance_between_positions(first: &Coord, second: &Coord) -> f32 {
    f32::sqrt(
        f32::powi(first.x as f32 - second.x as f32, 2)
            + f32::powi(first.y as f32 - second.y as f32, 2),
    )
}
