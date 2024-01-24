use std::io::Result;

use ligma::game::LigmaInvaders;

fn main() -> Result<()> {
    let game = LigmaInvaders::new();
    game.start()
}
