use ligma::game::LigmaInvaders;

fn main() {
    match LigmaInvaders::new() {
        Ok(mut game) => match game.start() {
            Ok(_) => (),
            Err(err) => {
                println!("{err}");
                game.reset_screen()
                    .expect("error while resetting the screen");
            }
        },
        Err(err) => println!("{err}"),
    }
}
