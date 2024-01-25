use std::io::Result;

use ligma::game::LigmaInvaders;

fn main() -> Result<()> {
    let mut game = LigmaInvaders::new();
    match game.start() {
        Ok(_) => Ok(()),
        Err(_) => game.reset_screen(),
    }
}
