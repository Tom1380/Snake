mod grid;
mod key;
mod preparation;

use {
    colored::Colorize,
    grid::*,
    key::*,
    std::{
        sync::mpsc::{channel, Receiver},
        thread::{spawn},
    },
};

fn game_loop(rx: Receiver<Key>, griglia: &mut Grid) {
    loop {
        if let Ok(key) = rx.try_recv() {
            use Key::*;
            match key {
                DropCoin => &griglia.drop_coin(),
                Arrow(direction) => &griglia.move_cursor(direction),
                _ => todo!(),
            };
        }
        // griglia.win();
        griglia.print_grid();
    }
}

fn main() {
    let users = preparation::users();
    let mut griglia = Grid::new(users,  10, 10);
    griglia.print_grid();
    let (tx, rx) = channel();
    spawn(move || listen_for_keys(tx));
    game_loop(rx, &mut griglia);
}
