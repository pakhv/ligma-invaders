use std::{
    io::{stdout, Result, Stdout, Write},
    time::{Duration, SystemTime},
};

use crossterm::{
    cursor,
    event::{self, poll, read, Event, KeyCode, KeyEvent},
    execute, queue, style,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};

use crate::{
    input_controls::InputControls,
    state::{Coord, Object, State},
};

const MS_PER_UPDATE: u128 = 50;

#[derive(Debug)]
pub struct LigmaInvaders {
    last_update: SystemTime,
    std_out: Stdout,
    state: State,
}

impl LigmaInvaders {
    pub fn new() -> LigmaInvaders {
        LigmaInvaders {
            last_update: SystemTime::now(),
            std_out: stdout(),
            state: State {
                player: Object::create_player(Coord { x: 10, y: 10 }),
                enemies: vec![],
            },
        }
    }

    pub fn start(&mut self) -> Result<()> {
        self.prepare_screen()?;
        self.set_last_update();

        loop {
            let user_input = if poll(Duration::from_millis(MS_PER_UPDATE as u64))? {
                handle_input()?
            } else {
                None
            };
            dbg!(&user_input);

            if user_input
                .as_ref()
                .is_some_and(|i| i == &InputControls::Quit)
            {
                break;
            }

            self.update_and_render(user_input)?;
        }

        self.reset_screen()
    }

    fn update_and_render(&mut self, user_input: Option<InputControls>) -> Result<()> {
        let mut lag = self.get_elapsed().as_millis();

        while lag >= MS_PER_UPDATE {
            self.set_last_update();
            lag -= MS_PER_UPDATE;

            match &user_input {
                Some(input) => match input {
                    InputControls::MoveLeft => self.state.player.go_left(),
                    InputControls::MoveRight => self.state.player.go_right(),
                    _ => (),
                },
                None => (),
            };
            self.render()?;
        }

        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        queue!(self.std_out, terminal::Clear(terminal::ClearType::All))?;

        for Coord { x, y } in &self.state.player.position {
            queue!(self.std_out, cursor::MoveTo(*x, *y), style::Print("█"))?;
        }

        self.std_out.flush()
    }

    fn set_last_update(&mut self) {
        self.last_update = SystemTime::now();
    }

    fn get_elapsed(&self) -> Duration {
        self.last_update.elapsed().unwrap()
    }

    fn prepare_screen(&mut self) -> Result<()> {
        enable_raw_mode()?;
        execute!(self.std_out, cursor::Hide, terminal::EnterAlternateScreen)
    }

    pub fn reset_screen(&mut self) -> Result<()> {
        execute!(
            self.std_out,
            style::ResetColor,
            cursor::Show,
            event::DisableMouseCapture,
            terminal::LeaveAlternateScreen,
        )?;
        disable_raw_mode()
    }
}

fn handle_input() -> Result<Option<InputControls>> {
    match read()? {
        Event::Key(KeyEvent {
            code: KeyCode::Left,
            ..
        }) => Ok(Some(InputControls::MoveLeft)),
        Event::Key(KeyEvent {
            code: KeyCode::Right,
            ..
        }) => Ok(Some(InputControls::MoveRight)),
        Event::Key(KeyEvent {
            code: KeyCode::Up, ..
        }) => Ok(Some(InputControls::Shoot)),
        Event::Key(KeyEvent {
            code: KeyCode::Char(ch),
            ..
        }) => match ch {
            'q' => Ok(Some(InputControls::Quit)),
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}
