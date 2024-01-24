use std::{
    io::{stdout, Result},
    thread,
    time::{Duration, SystemTime},
};

use crossterm::{
    cursor,
    event::{
        self, poll, read, Event, KeyCode, KeyEvent, KeyboardEnhancementFlags, ModifierKeyCode,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute, style,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};

use crate::input_controls::InputControls;

const MS_PER_UPDATE: u128 = 100;

#[derive(Debug)]
pub struct LigmaInvaders {}

impl LigmaInvaders {
    pub fn new() -> LigmaInvaders {
        LigmaInvaders {}
    }

    pub fn start(mut self) -> Result<()> {
        let mut stdout = stdout();

        enable_raw_mode()?;
        execute!(
            stdout,
            cursor::Hide,
            terminal::EnterAlternateScreen,
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES)
        )?;

        let mut previous = SystemTime::now();
        let mut lag: u128 = 0;

        loop {
            if poll(Duration::from_millis(MS_PER_UPDATE as u64))? {
                let input = handle_input()?;

                if input.is_some_and(|i| i == InputControls::Quit) {
                    break;
                }

                let current = SystemTime::now();
                let elapsed = previous.elapsed().unwrap();
                lag += elapsed.as_millis();

                self.update(lag);
                self.render();

                previous = current;
                //sleep(previous.elapsed().unwrap());
            }
        }

        execute!(
            stdout,
            style::ResetColor,
            cursor::Show,
            event::DisableMouseCapture,
            terminal::LeaveAlternateScreen,
            PopKeyboardEnhancementFlags,
        )?;
        disable_raw_mode()
    }

    fn update(&mut self, lag: u128) -> () {}

    fn render(&mut self) -> () {}
}

fn sleep(elapsed: Duration) {
    if elapsed.as_millis() < MS_PER_UPDATE {
        thread::sleep(Duration::from_millis(
            (MS_PER_UPDATE - elapsed.as_millis()) as u64,
        ));
    }
}

fn handle_input() -> Result<Option<InputControls>> {
    match read()? {
        Event::Key(KeyEvent {
            code: KeyCode::Left,
            modifiers: _,
            kind: _,
            state: _,
        }) => {
            println!("Left");
            Ok(Some(InputControls::MoveLeft))
        }
        Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: _,
            kind: _,
            state: _,
        }) => {
            println!("Right");
            Ok(Some(InputControls::MoveRight))
        }
        Event::Key(KeyEvent {
            code: KeyCode::Modifier(modifier),
            modifiers: _,
            kind: _,
            state: _,
        }) => match modifier {
            ModifierKeyCode::LeftShift => {
                println!("Shoot");
                Ok(Some(InputControls::Shoot))
            }
            _ => Ok(None),
        },
        Event::Key(KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: _,
            kind: _,
            state: _,
        }) => match ch {
            'q' => Ok(Some(InputControls::Quit)),
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}
