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
    input_result::InputResult,
    state::{Coord, Object, State},
};

const MS_PER_UPDATE: u128 = 10;

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
            if poll(Duration::from_millis(MS_PER_UPDATE as u64))? {
                let handle_result = self.handle_user_input()?;

                match handle_result {
                    InputResult::Continue => (),
                    InputResult::Quit => break,
                }
            }

            self.update_and_render()?;
        }

        self.reset_screen()
    }

    fn update_and_render(&mut self) -> Result<()> {
        let mut lag = self.get_elapsed().as_millis();

        while lag >= MS_PER_UPDATE {
            self.set_last_update();
            lag -= MS_PER_UPDATE;

            if lag < MS_PER_UPDATE {
                self.render()?;
            }
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

    fn handle_user_input(&mut self) -> Result<InputResult> {
        match read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                ..
            }) => {
                self.state.player.go_left();
                Ok(InputResult::Continue)
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                ..
            }) => {
                self.state.player.go_right();
                Ok(InputResult::Continue)
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up, ..
            }) => Ok(InputResult::Continue),
            Event::Key(KeyEvent {
                code: KeyCode::Char(ch),
                ..
            }) => match ch {
                'q' => Ok(InputResult::Quit),
                _ => Ok(InputResult::Continue),
            },
            _ => Ok(InputResult::Continue),
        }
    }
}
