use {getch::Getch, std::sync::mpsc::Sender};

#[derive(Debug, PartialEq)]
pub enum Axis {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone)]
pub enum Direction {
    Up,
    Right,
    Left,
    Down,
}

impl Direction {
    pub fn axis(&self) -> Axis {
        use Axis::*;
        use Direction::*;
        match self {
            Up | Down => Vertical,
            Left | Right => Horizontal,
        }
    }
}

pub enum Key {
    Arrow(Direction),
    Enter,
    Other,
}

fn read_key(g: &Getch) -> Key {
    use Direction::*;
    match g.getch() {
        Ok(87) | Ok(119) => Key::Arrow(Up),
        Ok(68) | Ok(100) => Key::Arrow(Right),
        Ok(83) | Ok(115) => Key::Arrow(Down),
        Ok(65) | Ok(97) => Key::Arrow(Left),
        Ok(10) => Key::Enter,
        Ok(32) => Key::Enter,
        Ok(27) => match g.getch() {
            Ok(91) => match g.getch() {
                Ok(65) => Key::Arrow(Up),
                Ok(67) => Key::Arrow(Right),
                Ok(66) => Key::Arrow(Down),
                Ok(68) => Key::Arrow(Left),
                _ => Key::Other,
            },
            _ => Key::Other,
        },
        _ => Key::Other,
    }
}

pub fn listen_for_keys(tx: Sender<Key>) {
    use Direction::*;
    let g = Getch::new();
    let mut direction = Down;
    tx.send(Key::Arrow(direction.clone())).unwrap();
    loop {
        match read_key(&g) {
            Key::Arrow(arrow) => {
                if direction.axis() != arrow.axis() {
                    direction = arrow;
                    match tx.send(Key::Arrow(direction.clone())) {
                        Ok(_) => {}
                        Err(_) => return,
                    }
                }
            }
            Key::Enter => {
                let _ = tx.send(Key::Enter);
                return;
            }
            _ => {}
        }
    }
}
