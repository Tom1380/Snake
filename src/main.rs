
use {
    dialoguer::{theme::ColorfulTheme, Input, PasswordInput, Select},
    getch::*,
    rand::random,
    fs::{File},
    std::{
        fs,
        collections::{VecDeque, HashMap},
        process::exit,
        sync::mpsc::{channel, Receiver, Sender},
        thread::{sleep, spawn},
        time::Duration,
    },
    serde_json
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

#[derive( Debug, Clone)]
struct User {
    username: String,
    password: String,
}

enum Acces {
    Granted,
    Denied,
}
// mut config: &HashMap<String,String>
fn login(mut config : HashMap<String, String>) -> (Acces,User) {
    let username: String = Input::new().with_prompt("Username").interact().unwrap();
    let password: String = PasswordInput::with_theme(&ColorfulTheme::default())
        .with_prompt("Password")
        .interact()
        .unwrap();
    
    config.insert("username".to_string(), username.to_owned());
    config.insert("password".to_string(), password.to_owned());
    serde_json::to_writer(&File::create("config.json").unwrap(), &config).unwrap();
    let user = User{username,password};
    (Acces::Granted,user)
}
fn signup() -> (Acces,User) {
    let mut config: HashMap<String, String> = HashMap::new();
    let username: String = Input::new().with_prompt("Username").interact().unwrap();
    let password = PasswordInput::with_theme(&ColorfulTheme::default())
        .with_prompt("Password")
        .with_confirmation("Repeat password", "Error: the passwords don't match.")
        .interact()
        .unwrap();
    config.insert("username".to_string(), username.to_owned());
    config.insert("password".to_string(), password.to_owned());
    let user = User { username, password };
    serde_json::to_writer(&File::create("config.json").unwrap(), &config).unwrap();
    (Acces::Granted,user)
}
fn main_menu(tx: std::sync::mpsc::Sender<Direction>, rx: std::sync::mpsc::Receiver<Direction>,user:User) {
    println!("\x1b[2J\x1b[1;1H");
    let selections = &["Play", "Settings", "Scores"];

    let selection = Select::with_theme(&ColorfulTheme::default()).with_prompt(format!("Ciao, {}", user.username).as_str())
        .default(0)
        .items(&selections[..])
        .interact_opt()
        .unwrap();

    if let Some(selection) = selection {
        match selections[selection] {
            "Play" => {
                spawn(move || listen_for_keys(tx));
                render_grid(rx);
            }
            "Settings" => {
                unimplemented!();
            }
            "Scores" => {
                unimplemented!()
            }
            _ => unreachable!(),
        }
    } else {
        println!("Non hai selezionato nulla");
    }
}

fn read_config() -> serde_json::value::Value {
    let file = fs::File::open("config.json").expect("file should open read only");
    serde_json::from_reader(file).expect("file should be proper JSON")
}
fn hashmapper(data: serde_json::value::Value) -> HashMap<String, String> {
    data.as_object()
        .unwrap()
        .iter()
        .map(|(key, value)| {
            (key.clone(), value.as_str().unwrap().to_string(),)
        })
        .collect()
}
fn main() {
    let config = read_config() ;
    let (tx, rx) = channel();
    // if first Acces or without account this screen will show up
    let selections = &["Log-in", "Sign-up"];

    let hashmap = hashmapper(config);
    if !hashmap.contains_key("username") && !hashmap.contains_key("password") {
            let selection = Select::with_theme(&ColorfulTheme::default())
        .default(0)
        .items(&selections[..])
        .interact_opt()
        .unwrap();
        if let Some(selection) = selection {
            match selections[selection] {
                "Log-in" => match login( hashmap) {
                    (Acces::Granted,user) => {
                        main_menu(tx, rx,user);
                    }
                    _ => unimplemented!(),
                },
                "Sign-up" => match signup() {
                    (Acces::Granted, user) => {
                        main_menu(tx, rx,user);
                    }
                    _ => unimplemented!(),
                },
                _ => unreachable!(),
            }
        } else {
            println!("Non hai selezionato nulla");
        }
    } 
    else {
        let user = User{
            username : hashmap.get("username").unwrap().to_string(),
            password : hashmap.get("password").unwrap().to_string()
        };
        main_menu(tx, rx,user);
    }

    
}
