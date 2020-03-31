use {getch::Getch, std::sync::mpsc::Sender};

pub enum Direction {
    Right,
    Left,
}

pub enum Key {
    Arrow(Direction),
    Space,
    DropCoin,
    Other,
}

pub fn read_key(g: &Getch) -> Key {
    use Direction::*;
    match g.getch() {
        Ok(68) | Ok(100) => Key::Arrow(Right),
        Ok(65) | Ok(97) => Key::Arrow(Left),
        Ok(83) | Ok(115) => Key::DropCoin,
        // Space or enter (enter works only on Linux).
        Ok(10) | Ok(32) => Key::Space,
        _ => Key::Other,
    }
}

pub fn listen_for_keys(tx: Sender<Key>) {
    let g = Getch::new();
    loop {
        let _ = tx.send(read_key(&g));
    }
}
