use {
    crate::clear_screen,
    getch::Getch,
    std::{io::stdin, sync::mpsc::Sender},
};

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
    Space,
    Pause,
    Other,
}

fn read_key(g: &Getch) -> Key {
    use Direction::*;
    match g.getch() {
        Ok(87) | Ok(119) => Key::Arrow(Up),
        Ok(68) | Ok(100) => Key::Arrow(Right),
        Ok(83) | Ok(115) => Key::Arrow(Down),
        Ok(65) | Ok(97) => Key::Arrow(Left),
        // Space or enter (enter works only on Linux).
        Ok(10) | Ok(32) => Key::Space,
        Ok(80) | Ok(112) => Key::Pause,
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
    let mut g = Getch::new();
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
            Key::Space => {
                let _ = tx.send(Key::Space);
                return;
            }
            Key::Pause => {
                let _ = tx.send(Key::Pause);
                std::mem::drop(g);
                funny_pause_game::funny_pause_game();
                g = Getch::new();
                let _ = tx.send(Key::Pause);
            }
            _ => {}
        }
    }
}

mod funny_pause_game {
    use super::*;
    pub fn funny_pause_game() {
        clear_screen();
        print_help();
        let mut input = String::new();
        let mut numbers: Vec<f64> = Vec::new();
        loop {
            input.clear();
            stdin().read_line(&mut input).unwrap();
            println!("");
            input.make_ascii_lowercase();
            input.pop();
            input = input.trim().to_string();
            match input.as_str() {
                "riavvia" => {
                    clear_screen();
                    numbers.clear();
                    print_help();
                }
                "media" => {
                    if numbers.len() == 0 {
                        println!("Nessun numero inserito, perciò la media è 0.\n");
                    } else {
                        if numbers.len() == 1 {
                            println!("Hai inserito 1 numero:");
                        } else {
                            println!("Hai inserito {} numeri:", numbers.len());
                        }
                        for n in &numbers {
                            println!("{}", n);
                        }
                        println!(
                            "\nLa media è {}\n",
                            numbers.iter().sum::<f64>() / numbers.len() as f64
                        );
                    }
                }
                "aiuto" => print_help(),
                "esci" => return,
                "" => {}
                _ => {
                    match input.split(" ").map(|n| n.replace(",", ".").parse::<f64>()).collect::<Result<Vec<f64>, _>>() {
                        Ok(new_numbers) => {
                            for &number in &new_numbers {
                                numbers.push(number);
                            }
                            if new_numbers.len() == 1 {
                                println!("Inserito 1 nuovo numero.\n");
                            }
                            else {
                                println!("Inseriti {} nuovi numeri.\n", new_numbers.len());
                            }
                        }
                        Err(_) => println!("Inserisci numeri con lo spazio, per i numeri decimali non usare la virgola, ma il punto.")
                    }
                }
            }
        }
    }
    fn print_help() {
        println!(
"MEDIA DEI NUMERI:
Comandi:
\"riavvia\": Riavvia il programma ripartendo da zero.
\"media\": Leggi i numeri che hai inserito.
\"aiuto\": Mostra le istruzioni.
Inserendo numeri, anche più di uno sulla stessa linea, se separati da spazio, li aggiungi alla lista.
");
    }
}
