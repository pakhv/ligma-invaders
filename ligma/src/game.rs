use std::{
    io::{stdout, Result, Stdout},
    time::{Duration, SystemTime},
};

use crossterm::{
    cursor,
    event::{self, poll, read, Event, KeyCode, KeyEvent},
    execute, style,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};

use crate::input_controls::InputControls;

const MS_PER_UPDATE: u128 = 100;

#[derive(Debug)]
pub struct LigmaInvaders {
    last_update: SystemTime,
    std_out: Stdout,
}

impl LigmaInvaders {
    pub fn new() -> LigmaInvaders {
        LigmaInvaders {
            last_update: SystemTime::now(),
            std_out: stdout(),
        }
    }

    pub fn start(mut self) -> Result<()> {
        enable_raw_mode()?;
        execute!(self.std_out, cursor::Hide, terminal::EnterAlternateScreen,)?;

        self.set_last_update();

        loop {
            if poll(Duration::from_millis(MS_PER_UPDATE as u64))? {
                let user_input = handle_input()?;

                if user_input
                    .as_ref()
                    .is_some_and(|i| i == &InputControls::Quit)
                {
                    break;
                }

                self.update(user_input);
                self.render();
            }
        }

        execute!(
            self.std_out,
            style::ResetColor,
            cursor::Show,
            event::DisableMouseCapture,
            terminal::LeaveAlternateScreen,
        )?;
        disable_raw_mode()
    }

    fn update(&mut self, _user_input: Option<InputControls>) {
        let mut lag = self.get_elapsed().as_millis();

        while lag >= MS_PER_UPDATE {
            self.set_last_update();
            lag -= MS_PER_UPDATE;
        }
    }

    fn render(&mut self) {}

    fn set_last_update(&mut self) {
        self.last_update = SystemTime::now();
    }

    fn get_elapsed(&self) -> Duration {
        self.last_update.elapsed().unwrap()
    }
}
//
// fn sleep(elapsed: Duration) {
//     if elapsed.as_millis() < MS_PER_UPDATE {
//         thread::sleep(Duration::from_millis(
//             (MS_PER_UPDATE - elapsed.as_millis()) as u64,
//         ));
//     }
// }

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
