mod keys;

use {
    crate::{config::*, *},
    keys::*,
    rand::random,
    std::{
        collections::VecDeque,
        io::{stdout, Write},
        sync::mpsc::{channel, Receiver},
        thread::{sleep, spawn},
        time::Duration,
    },
};

const ROWS: u8 = 20;
const COLUMNS: u8 = 20;
const CELLS: u16 = ROWS as u16 * COLUMNS as u16;
// How many snacks should be dropped at a time?
const MAX_SNACKS_DROPPED: usize = CELLS as usize / 30;
// Seconds to cross a cell.
const SPEEDS: &'static [f32] = &[0.2, 0.1, 0.08, 0.05, 0.03];

#[derive(Debug, PartialEq, Clone)]
struct Cell {
    x: u8,
    y: u8,
}

impl Cell {
    fn neighbour(&self, direction: &Direction) -> Option<Cell> {
        use Direction::*;
        let mut neighbour = self.clone();
        match direction {
            Up => {
                if self.y > 0 {
                    neighbour.y -= 1;
                } else {
                    return None;
                }
            }
            Right => {
                if self.x < COLUMNS - 1 {
                    neighbour.x += 1;
                } else {
                    return None;
                }
            }
            Down => {
                if self.y < ROWS - 1 {
                    neighbour.y += 1;
                } else {
                    return None;
                }
            }
            Left => {
                if self.x > 0 {
                    neighbour.x -= 1;
                } else {
                    return None;
                }
            }
        }
        Some(neighbour)
    }
}

struct OutputBuffer {
    buffer: String,
}

impl OutputBuffer {
    fn with_capacity(capacity: usize) -> OutputBuffer {
        OutputBuffer {
            buffer: String::with_capacity(capacity),
        }
    }

    fn append(&mut self, s: &str) {
        self.buffer.push_str(&s);
    }

    fn flush(&mut self) {
        print!("{}", &self.buffer);
        self.buffer.clear();
    }

    fn clear_screen(&mut self) {
        if cfg!(target_os = "windows") {
            self.append(&"\n".repeat(50));
        } else {
            self.append("\x1b[2J\x1b[1;1H");
        }
    }
}

pub fn play(config: &Config) {
    let (tx, rx) = channel();
    spawn(move || listen_for_keys(tx));
    game_loop(rx, &config);
}

fn game_loop(rx: Receiver<Key>, config: &Config) {
    let sleep_duration = Duration::from_secs_f32(SPEEDS[config.difficulty]);
    let mut snake = vec![Cell { x: 0, y: 0 }].into_iter().collect();
    let mut op = OutputBuffer::with_capacity(CELLS as usize + 20);
    let mut snacks = VecDeque::with_capacity(MAX_SNACKS_DROPPED);
    let mut direction: Direction = match rx.recv().unwrap() {
        Key::Arrow(direction) => direction,
        Key::Space => {
            return;
        }
        _ => unreachable!(),
    };
    loop {
        op.flush();
        op.clear_screen();
        print_grid(&snake, &snacks, &mut op, &config);
        sleep(sleep_duration);
        if let Ok(key) = rx.try_recv() {
            match key {
                Key::Arrow(new_direction) => direction = new_direction,
                Key::Space => {
                    return;
                }
                Key::Pause => {
                    op.clear_screen();
                    print_grid(&snake, &snacks, &mut op, &config);
                    let _ = rx.recv();
                    op.flush();
                    wait_after_game_resume();
                }
                _ => unreachable!(),
            };
        }
        let new_head = match snake.iter().last().unwrap().neighbour(&direction) {
            Some(new_head) => new_head,
            None => {
                game_over(&mut op, snake.len() - 1, &config);
                println!("Premi spazio per continuare.");
                clear_receiver(&rx);
                let _ = rx.recv();
                clear_screen();
                return;
            }
        };
        if snake.contains(&new_head) {
            game_over(&mut op, snake.len() - 1, &config);
            println!("Premi spazio per continuare.");
            clear_receiver(&rx);
            let _ = rx.recv();
            clear_screen();
            return;
        } else if let Some(i) = snacks.iter().position(|cell| *cell == new_head) {
            snacks.remove(i);
        }
        // If the game is lost or a snack was eaten, don't pop the oldest position.
        else {
            snake.pop_front();
        }
        if snacks.is_empty() {
            generate_snacks(&snake, &mut snacks);
        }
        snake.push_back(new_head);
    }
}

fn print_grid(
    snake: &VecDeque<Cell>,
    snacks: &VecDeque<Cell>,
    op: &mut OutputBuffer,
    config: &Config,
) {
    // if cfg!(target_os = "windows") {
    for y in 0..ROWS {
        for x in 0..COLUMNS {
            if snake.contains(&Cell { x, y }) {
                op.append("|+")
            } else if snacks.contains(&Cell { x, y }) {
                op.append("|O");
            } else {
                op.append("|_");
            }
        }
        op.append("|\n");
    }
    op.append(
        format!(
            "Punteggio: {}, difficoltà: {}.\n",
            snake.len() - 1,
            DIFFICULTIES[config.difficulty]
        )
        .as_str(),
    );
}

fn generate_snacks(snake: &VecDeque<Cell>, snacks: &mut VecDeque<Cell>) {
    for _ in 0..MAX_SNACKS_DROPPED {
        let snack_location = Cell {
            x: random::<u8>() % ROWS,
            y: random::<u8>() % COLUMNS,
        };
        if !(snake.contains(&snack_location) || snacks.contains(&snack_location)) {
            snacks.push_front(snack_location);
        }
    }
}

fn game_over(op: &mut OutputBuffer, score: usize, config: &Config) {
    op.append("HAI PERSO.\n");
    op.flush();
    sleep(Duration::from_secs(2));
    let client = reqwest::Client::new();
    match client
        .post(&format!(
            "http://167.172.50.64/upload_score/{}/{}/{}",
            config.difficulty, config.username, score
        ))
        .send()
        .map(|r| (r.status(), r))
    {
        Ok((reqwest::StatusCode::NO_CONTENT, _)) => println!("Punteggio salvato!"),
        Ok((reqwest::StatusCode::CREATED, mut response)) => {
            println!("Punteggio salvato!");
            match response.json::<serde_json::Value>() {
                Ok(serde_json::Value::Object(json)) => match json.get("beaten") {
                    Some(serde_json::Value::String(beaten)) => match beaten.as_str() {
                        "absolute" => println!(
                            "Nuovo record assoluto di {}!",
                            DIFFICULTIES[config.difficulty]
                        ),
                        "personal" => println!(
                            "Nuovo record personale di {}!",
                            DIFFICULTIES[config.difficulty]
                        ),
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            }
        }
        _ => println!("A causa di un problema non ho potuto salvare il punteggio."),
    }
}

fn clear_receiver(rx: &Receiver<Key>) {
    while let Ok(_) = rx.try_recv() {}
}

fn wait_after_game_resume() {
    let mut so = stdout();
    print!("RIPARTENZA IN 3");
    so.flush().unwrap();
    sleep(Duration::from_secs(1));
    print!(", 2");
    so.flush().unwrap();
    sleep(Duration::from_secs(1));
    print!(", 1");
    so.flush().unwrap();
    sleep(Duration::from_secs(1));
}
