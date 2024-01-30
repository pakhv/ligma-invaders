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
    ligma_result::LigmaResult,
    state::{Coord, State},
};

pub const MS_PER_UPDATE: u128 = 10;
pub const VIEWPORT_MIN_X: u16 = 1;
pub const VIEWPORT_MAX_X: u16 = 200;
pub const VIEWPORT_MIN_Y: u16 = 1;
pub const VIEWPORT_MAX_Y: u16 = 40;

#[derive(Debug)]
pub struct LigmaInvaders {
    last_update: SystemTime,
    std_out: Stdout,
    state: State,
}

impl LigmaInvaders {
    pub fn new() -> LigmaResult<LigmaInvaders> {
        Ok(LigmaInvaders {
            last_update: SystemTime::now(),
            std_out: stdout(),
            state: State::new()?,
        })
    }

    pub fn start(&mut self) -> LigmaResult<()> {
        self.prepare_screen()
            .map_err(|err| format!("error while preparing the screen, {err}"))?;
        self.set_last_update();

        loop {
            if poll(Duration::from_millis(MS_PER_UPDATE as u64))
                .map_err(|err| format!("error polling for user input, {err}"))?
            {
                let handle_result = self
                    .handle_user_input()
                    .map_err(|err| format!("error while handling user input, {err}"))?;

                match handle_result {
                    InputResult::Continue => (),
                    InputResult::Quit => break,
                }
            }

            self.update_and_render()?;
        }

        self.reset_screen()
            .map_err(|err| format!("error while resetting the screen, {err}"))
    }

    pub fn reset_screen(&mut self) -> Result<()> {
        execute!(
            self.std_out,
            style::ResetColor,
            cursor::Show,
            event::DisableMouseCapture,
            terminal::LeaveAlternateScreen
        )?;
        disable_raw_mode()
    }

    fn update_and_render(&mut self) -> LigmaResult<()> {
        let mut lag = self.get_elapsed().as_millis();

        while lag >= MS_PER_UPDATE {
            self.set_last_update();
            lag -= MS_PER_UPDATE;

            self.state.update_player_laser();
            self.state.update_aliens();
            self.state.apply_collisions();

            if lag < MS_PER_UPDATE {
                self.render()
                    .map_err(|err| format!("error while rendering game state, {err}"))?;
            }
        }

        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        queue!(self.std_out, terminal::Clear(terminal::ClearType::All))?;

        for Coord { x, y, ch } in &self.state.player.position {
            queue!(self.std_out, cursor::MoveTo(*x, *y), style::Print(ch))?;
        }

        if self.state.player.laser.is_some() {
            for Coord { x, y, ch } in &self.state.player.laser.as_ref().unwrap().position {
                queue!(self.std_out, cursor::MoveTo(*x, *y), style::Print(ch))?;
            }
        }

        for aliens_row in &self.state.aliens.aliens_rows {
            for alien in &aliens_row.aliens {
                for Coord { x, y, ch } in &alien.position {
                    queue!(self.std_out, cursor::MoveTo(*x, *y), style::Print(ch))?;
                }
            }
        }

        for bunker in &self.state.bunkers.positions {
            for Coord { x, y, ch } in &bunker.position {
                queue!(self.std_out, cursor::MoveTo(*x, *y), style::Print(ch))?;
            }
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

    fn handle_user_input(&mut self) -> Result<InputResult> {
        match read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                ..
            }) => {
                self.state.player_go_left();
                Ok(InputResult::Continue)
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                ..
            }) => {
                self.state.player_go_right();
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
                ' ' => {
                    self.state.player_shoot();
                    Ok(InputResult::Continue)
                }
                _ => Ok(InputResult::Continue),
            },
            _ => Ok(InputResult::Continue),
        }
    }
}
