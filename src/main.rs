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
    op.append(format!("Score: {}\n", snake.len() - 1).as_str());
}

fn clear_screen(op: &mut OutputBuffer) {
    if cfg!(target_os = "windows") {
        op.append(&"\n".repeat(30));
    } else {
        op.append("\x1b[2J\x1b[1;1H");
    }
}

fn game_over(op: &mut OutputBuffer) {
    op.append("YOU LOST.\n"); //.color("red"));
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
