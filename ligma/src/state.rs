use std::cmp;

#[derive(Debug)]
pub struct State {
    pub player: Object,
    pub enemies: Vec<Object>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Coord {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug)]
pub struct Object {
    pub health: usize,
    pub position: Vec<Coord>,
    pub obj_type: ObjectType,
}

impl Object {
    pub fn create_player(shift: Coord) -> Object {
        let position = vec![
            Coord {
                x: 0 + shift.x,
                y: 0 + shift.y,
            },
            Coord {
                x: 1 + shift.x,
                y: 0 + shift.y,
            },
            Coord {
                x: 0 + shift.x,
                y: 1 + shift.y,
            },
            Coord {
                x: 1 + shift.x,
                y: 1 + shift.y,
            },
        ];

        Object {
            health: 3,
            position,
            obj_type: ObjectType::Player,
        }
    }

    pub fn go_left(&mut self) {
        self.shift_by(-3, 0);
    }

    pub fn go_right(&mut self) {
        self.shift_by(3, 0);
    }

    fn shift_by(&mut self, x_shift: i16, y_shift: i16) {
        self.position = self
            .position
            .iter()
            .map(|c| Coord {
                x: (cmp::max(c.x as i16 + x_shift, 0)) as u16,
                y: (cmp::max(c.y as i16 + y_shift, 0)) as u16,
            })
            .collect();
    }
    //
    // pub fn create_alien() -> Object {
    //     Object {
    //         health: 1,
    //         position: vec![],
    //         obj_type: ObjectType::Alien,
    //     }
    // }
    //
    // pub fn is_hit(&self, lazer_coord: Coord) -> bool {
    //     self.position.contains(&lazer_coord)
    // }
}

#[derive(Debug)]
pub enum ObjectType {
    Player,
    Alien,
}
