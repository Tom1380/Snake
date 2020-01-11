use {
    colored::*,
    getch::*,
    rand::random,
    std::{
        collections::VecDeque,
        io::{self, Write},
        process::exit,
        sync::mpsc::{channel, Receiver, Sender},
        thread::{sleep, spawn},
        time::Duration,
    },
};

const ROWS: u8 = 20;
const COLUMNS: u8 = 20;
const CELLS: u16 = ROWS as u16 * COLUMNS as u16;
const MAX_SNACKS_DROPPED: u16 = CELLS / 30; // How many snacks should be dropped at a time?
const SPEED: f32 = 0.25; // Seconds to cross a cell.

#[derive(Debug, PartialEq)]
enum Axis {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone)]
enum Direction {
    Up,
    Right,
    Left,
    Down,
}

impl Direction {
    fn axis(&self) -> Axis {
        use Axis::*;
        use Direction::*;
        match self {
            Up | Down => Vertical,
            Left | Right => Horizontal,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Cell {
    x: u8,
    y: u8,
}

impl Cell {
    fn neighbour(&self, direction: &Direction) -> Cell {
        use Direction::*;
        let mut neighbour = self.clone();
        match direction {
            Up => {
                if self.y > 0 {
                    neighbour.y -= 1;
                } else {
                    game_over();
                }
            }
            Right => {
                if self.x < COLUMNS - 1 {
                    neighbour.x += 1;
                } else {
                    game_over();
                }
            }
            Down => {
                if self.y < ROWS - 1 {
                    neighbour.y += 1;
                } else {
                    game_over();
                }
            }
            Left => {
                if self.x > 0 {
                    neighbour.x -= 1;
                } else {
                    game_over();
                }
            }
        }
        neighbour
    }
}

fn read_arrow(g: &Getch) -> Option<Direction> {
    use Direction::*;
    match g.getch() {
        Ok(27) => match g.getch() {
            Ok(91) => match g.getch() {
                Ok(65) => Some(Up),
                Ok(67) => Some(Right),
                Ok(66) => Some(Down),
                Ok(68) => Some(Left),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

fn print_grid(snake: &VecDeque<Cell>, snacks: &VecDeque<Cell>) {
    let colored_fs_char = "|".color("blue");
    let snake_char = "+".color("green");
    let snack_char = "O".color("yellow");
    let colored_underscore_char = "_".color("blue");
    for y in 0..ROWS {
        for x in 0..COLUMNS {
            print!(
                "{}{}",
                colored_fs_char,
                if snake.contains(&Cell { x, y }) {
                    &snake_char
                } else if snacks.contains(&Cell { x, y }) {
                    &snack_char
                } else {
                    &colored_underscore_char
                }
            )
        }
        println!("{}", colored_fs_char);
    }
    println!("SCORE: {}", (snake.len() - 1).to_string().color("yellow"));
}

fn clear_screen() {
    io::stdout()
        .write_all("\x1b[2J\x1b[1;1H".as_bytes())
        .unwrap();
}

fn game_over() {
    println!("{}", "YOU LOST.".color("red"));
    exit(0);
}

fn generate_snacks(snake: &VecDeque<Cell>) -> VecDeque<Cell> {
    let space_left = CELLS - snake.len() as u16;
    let snacks_dropped = MAX_SNACKS_DROPPED.min(space_left);
    let mut snacks = VecDeque::with_capacity(snacks_dropped as usize);
    for _ in 0..snacks.capacity() {
        let snack_location = Cell {
            x: random::<u8>() % ROWS,
            y: random::<u8>() % COLUMNS,
        };
        if !snake.contains(&snack_location) {
            snacks.push_front(snack_location);
        }
    }
    snacks
}

fn listen_for_keys(tx: Sender<Direction>) {
    use Direction::*;
    let g = Getch::new();
    let mut direction = Down;
    tx.send(direction.clone()).unwrap();
    loop {
        match read_arrow(&g) {
            Some(arrow) => {
                if direction.axis() != arrow.axis() {
                    direction = arrow;
                    tx.send(direction.clone()).unwrap();
                }
            }
            _ => {}
        }
    }
}

fn render_grid(rx: Receiver<Direction>) {
    let mut snake = vec![
        Cell { x: 0, y: 0 },
    ]
    .into_iter()
    .collect();
    let mut snacks = generate_snacks(&snake);
    let mut direction = rx.recv().unwrap();
    loop {
        clear_screen();
        print_grid(&snake, &snacks);
        sleep(Duration::from_secs_f32(SPEED));
        if let Ok(new_direction) = rx.try_recv() {
            direction = new_direction;
        }
        let new_head = snake.iter().last().unwrap().neighbour(&direction, );
        if snake.contains(&new_head) {
            game_over();
        } else if let Some(i) = snacks.iter().position(|cell| *cell == new_head) {
            snacks.remove(i);
            if snacks.is_empty() {
                snacks = generate_snacks(&snake);
            }
        }
        // If the game is lost or a snack was eaten, don't pop the oldest position.
        else {
            snake.pop_front();
        }
        snake.push_back(new_head);
    }
}

fn main() {
    let (tx, rx) = channel();
    spawn(move || listen_for_keys(tx));
    render_grid(rx);
}
