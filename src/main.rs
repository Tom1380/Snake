use {
    getch::*,
    rand::random,
    std::{
        collections::VecDeque,
        io::Write,
        process::exit,
        sync::mpsc::{channel, Receiver, Sender},
        thread::{sleep, spawn},
        time::Duration,
    },
    termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor},
};

const ROWS: u8 = 20;
const COLUMNS: u8 = 20;
const CELLS: u16 = ROWS as u16 * COLUMNS as u16;
const MAX_SNACKS_DROPPED: u16 = CELLS / 30; // How many snacks should be dropped at a time?
const SPEED: f32 = 0.08; // Seconds to cross a cell.

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
    fn neighbour(&self, direction: &Direction, mut op: &mut OutputBuffer) -> Cell {
        use Direction::*;
        let mut neighbour = self.clone();
        match direction {
            Up => {
                if self.y > 0 {
                    neighbour.y -= 1;
                } else {
                    game_over(&mut op);
                }
            }
            Right => {
                if self.x < COLUMNS - 1 {
                    neighbour.x += 1;
                } else {
                    game_over(&mut op);
                }
            }
            Down => {
                if self.y < ROWS - 1 {
                    neighbour.y += 1;
                } else {
                    game_over(&mut op);
                }
            }
            Left => {
                if self.x > 0 {
                    neighbour.x -= 1;
                } else {
                    game_over(&mut op);
                }
            }
        }
        neighbour
    }
}

struct OutputBuffer {
    buffer: Vec<(String, Option<Color>)>,
    stdout: StandardStream,
}

impl OutputBuffer {
    fn with_capacity(capacity: usize) -> OutputBuffer {
        OutputBuffer {
            buffer: Vec::with_capacity(capacity),
            stdout: StandardStream::stdout(ColorChoice::Always),
        }
    }

    fn append(&mut self, tup: &(String, Option<Color>)) {
        let len = self.buffer.len();
        if len == 0 {
            self.buffer.push((tup.0.clone(), (tup.1)));
            return;
        }
        let last = &mut self.buffer[len - 1];
        if (tup.1) == last.1 {
            last.0 += &tup.0;
        } else {
            self.buffer.push((tup.0.clone(), (tup.1)));
        }
    }

    fn flush(&mut self) {
        for (s, color) in &self.buffer {
            self.stdout
                .set_color(ColorSpec::new().set_fg(*color))
                .unwrap();
            write!(&mut self.stdout, "{}", s);
        }
        self.buffer.clear();
    }
}

fn read_arrow(g: &Getch) -> Option<Direction> {
    use Direction::*;
    match g.getch() {
        Ok(87) | Ok(119) => Some(Up),
        Ok(68) | Ok(100) => Some(Right),
        Ok(83) | Ok(115) => Some(Down),
        Ok(65) | Ok(97) => Some(Left),
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

fn print_grid(snake: &VecDeque<Cell>, snacks: &VecDeque<Cell>, op: &mut OutputBuffer) {
    let colored_fs_char = (String::from("|"), Some(Color::Blue));
    let snake_char = (String::from("+"), Some(Color::Green));
    let snack_char = (String::from("O"), Some(Color::Yellow));
    for y in 0..ROWS {
        for x in 0..COLUMNS {
            if snake.contains(&Cell { x, y }) {
                op.append(&colored_fs_char);
                op.append(&snake_char);
            } else if snacks.contains(&Cell { x, y }) {
                op.append(&colored_fs_char);
                op.append(&snack_char);
            } else {
                op.append(&(String::from("|_"), Some(Color::Blue)));
            }
        }
        op.append(&(String::from("|\n"), Some(Color::Blue)));
    }
    op.append(&(String::from("Score: "), Some(Color::White)));
    op.append(&((snake.len() - 1).to_string() + &"\n", Some(Color::Yellow))); //.color("yellow"));
}

fn clear_screen(op: &mut OutputBuffer) {
    if cfg!(target_os = "windows") {
        op.append(&("\n".repeat(30), None));
    }
    else {
        op.append(&(String::from("\x1b[2J\x1b[1;1H"), None));
    }
}

fn game_over(op: &mut OutputBuffer) {
    op.append(&(String::from("YOU LOST.\n"), Some(Color::Red))); //.color("red"));
    op.flush();
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
    let mut snake = vec![Cell { x: 0, y: 0 }].into_iter().collect();
    let mut op = OutputBuffer::with_capacity(CELLS as usize + 20);
    let mut snacks = generate_snacks(&snake);
    let mut direction = rx.recv().unwrap();
    loop {
        op.flush();
        clear_screen(&mut op);
        print_grid(&snake, &snacks, &mut op);
        sleep(Duration::from_secs_f32(SPEED));
        if let Ok(new_direction) = rx.try_recv() {
            direction = new_direction;
        }
        let new_head = snake.iter().last().unwrap().neighbour(&direction, &mut op);
        if snake.contains(&new_head) {
            game_over(&mut op);
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
