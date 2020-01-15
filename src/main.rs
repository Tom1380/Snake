use {
    colored::*,
    dialoguer::{theme::ColorfulTheme, Input, PasswordInput, Select},
    fs::File,
    getch::*,
    rand::random,
    reqwest,
    serde_json::{self, json},
    std::{
        collections::{HashMap, VecDeque},
        fs,
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
                                            // const SPEED: f32 = 0.08; // Seconds to cross a cell.
const DIFFICULTIES: &'static [&'static str] = &[
    "Mi faccio le 2006",
    "Snitch",
    "Pick-Up Coach",
    "Sono russo dentro",
    "Iran 2020",
];
const SPEEDS: &'static [f32] = &[0.3, 0.2, 0.1, 0.08, 0.05];

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

enum Key {
    Arrow(Direction),
    Enter,
    Other,
}

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
}

fn read_key(g: &Getch) -> Key {
    use Direction::*;
    match g.getch() {
        Ok(87) | Ok(119) => Key::Arrow(Up),
        Ok(68) | Ok(100) => Key::Arrow(Right),
        Ok(83) | Ok(115) => Key::Arrow(Down),
        Ok(65) | Ok(97) => Key::Arrow(Left),
        Ok(10) => Key::Enter,
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

fn print_grid(
    snake: &VecDeque<Cell>,
    snacks: &VecDeque<Cell>,
    op: &mut OutputBuffer,
    difficulty: usize,
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
            "Punteggio: {}, difficolta': {}\n",
            snake.len() - 1,
            DIFFICULTIES[difficulty]
        )
        .as_str(),
    );
    // } else {
    //     let colored_fs_char = "|".color("blue");
    //     let snake_char = "+".color("green");
    //     let snack_char = "O".color("yellow");
    //     let colored_underscore_char = "_".color("blue");
    //     for y in 0..ROWS {
    //         for x in 0..COLUMNS {
    //             print!(
    //                 "{}{}",
    //                 colored_fs_char,
    //                 if snake.contains(&Cell { x, y }) {
    //                     &snake_char
    //                 } else if snacks.contains(&Cell { x, y }) {
    //                     &snack_char
    //                 } else {
    //                     &colored_underscore_char
    //                 }
    //             )
    //         }
    //         println!("{}", colored_fs_char);
    //     }
    //     println!("SCORE: {}", (snake.len() - 1).to_string().color("yellow"));
    // }
}

fn op_clear_screen(op: &mut OutputBuffer) {
    if cfg!(target_os = "windows") {
        op.append(&"\n".repeat(30));
    } else {
        op.append("\x1b[2J\x1b[1;1H");
    }
}

fn clear_screen() {
    if cfg!(target_os = "windows") {
        println!("{}", "\n".repeat(30));
    } else {
        print!("\x1b[2J\x1b[1;1H");
    }
}

fn game_over(op: &mut OutputBuffer, score: usize, config: &HashMap<String, serde_json::Value>) {
    op.append("HAI PERSO.\n"); //.color("red"));
    op.flush();
    let difficulty = config["difficulty"].as_i64().unwrap() as usize;
    let username = config["username"].as_str().unwrap();
    let client = reqwest::Client::new();
    match client
        .post(&format!(
            "http://167.172.50.64/upload_score/{}/{}/{}",
            difficulty, username, score
        ))
        .send()
    {
        Ok(_) => println!("Punteggio salvato!"),
        Err(_) => println!("A causa di un problema non ho potuto salvare il punteggio."),
    }
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

fn listen_for_keys(tx: Sender<Key>) {
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

fn render_grid(rx: Receiver<Key>, difficulty: usize, config: &HashMap<String, serde_json::Value>) {
    let sleep_duration = Duration::from_secs_f32(SPEEDS[difficulty]);
    let mut snake = vec![Cell { x: 0, y: 0 }].into_iter().collect();
    let mut op = OutputBuffer::with_capacity(CELLS as usize + 20);
    let mut snacks = generate_snacks(&snake);
    let mut direction: Direction = match rx.recv().unwrap() {
        Key::Arrow(direction) => direction,
        Key::Enter => {
            return;
        }
        _ => unreachable!(),
    };
    loop {
        op.flush();
        op_clear_screen(&mut op);
        print_grid(&snake, &snacks, &mut op, difficulty);
        sleep(sleep_duration);
        if let Ok(key) = rx.try_recv() {
            match key {
                Key::Arrow(new_direction) => direction = new_direction,
                Key::Enter => {
                    return;
                }
                _ => unreachable!(),
            };
        }
        let new_head = match snake.iter().last().unwrap().neighbour(&direction) {
            Some(new_head) => new_head,
            None => {
                game_over(&mut op, snake.len() - 1, &config);
                println!("Premi invio per continuare.");
                let _ = rx.recv();
                clear_screen();
                return;
            }
        };
        if snake.contains(&new_head) {
            game_over(&mut op, snake.len() - 1, &config);
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

#[derive(Debug, Clone)]
struct User {
    username: String,
}

enum Access {
    Granted,
    Denied,
}
// mut config: &HashMap<String,String>
fn login(config: &mut HashMap<String, serde_json::Value>) -> (Access, User) {
    let username: String = Input::new().with_prompt("Username").interact().unwrap();

    config.insert("username".to_string(), json!(username.to_owned()));
    serde_json::to_writer(&File::create("config.json").unwrap(), &config).unwrap();
    let user = User { username };
    (Access::Granted, user)
}
fn signup() -> (Access, User) {
    let mut config: HashMap<String, String> = HashMap::new();
    let username: String = Input::new().with_prompt("Username").interact().unwrap();
    let password = PasswordInput::with_theme(&ColorfulTheme::default())
        .with_prompt("Password")
        .with_confirmation("Repeat password", "Error: the passwords don't match.")
        .interact()
        .unwrap();
    config.insert("username".to_string(), username.to_owned());
    config.insert("password".to_string(), password.to_owned());
    let user = User { username };
    serde_json::to_writer(&File::create("config.json").unwrap(), &config).unwrap();
    (Access::Granted, user)
}
fn main_menu(user: User, mut config: HashMap<String, serde_json::Value>) {
    loop {
        clear_screen();
        println!("");
        let selections = &["Gioca", "Impostazioni", "Punteggi"];
        println!("SNAKE");
        println!("Quando giochi, usa WASD per muoverti.");
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Eccoti, {}", user.username).as_str())
            .default(0)
            .items(&selections[..])
            .interact_opt()
            .unwrap();

        if let Some(selection) = selection {
            match selection {
                0 => {
                    let (tx, rx) = channel();
                    spawn(move || listen_for_keys(tx));
                    render_grid(
                        rx,
                        config[&String::from("difficulty")].as_i64().unwrap() as usize,
                        &config,
                    );
                }
                1 => {
                    settings(&mut config);
                }
                2 => {
                    scores(&config);
                }
                _ => unreachable!(),
            }
        }
    }
}

fn settings(config: &mut HashMap<String, serde_json::Value>) {
    let selections = DIFFICULTIES;
    clear_screen();
    let difficulty = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Difficolta'")
        .default(0)
        .items(&selections[..])
        .interact_opt()
        .unwrap();
    config.insert(String::from("difficulty"), json!(difficulty.unwrap()));
    serde_json::to_writer(&File::create("config.json").unwrap(), &config).unwrap();
}

fn scores(config: &HashMap<String, serde_json::Value>) {
    let client = reqwest::Client::new();
    let difficulty = DIFFICULTIES[config["difficulty"].as_i64().unwrap() as usize];
    let scores: serde_json::Value = client
        .get(&format!("http://167.172.50.64/scores/{}", difficulty))
        .send()
        .expect("Qualcosa è andato storto")
        .json()
        .unwrap();
    let scores: &Vec<serde_json::Value> = scores.as_array().unwrap();
    if scores.len() == 0 {
        println!("Nessun punteggio per la difficoltà {}.", difficulty);
    }
    else {
        println!("Punteggi per la difficoltà {}:", difficulty);
    }
    for (i, score) in scores.iter().map(|j| hashmapper(j.clone())).enumerate() {
        let username = score["username"].as_str().unwrap();
        let data = score["data"].as_str().unwrap();
        let score = score["score"].as_str().unwrap();
        println!("{}) {} - {} - {}", i + 1, score, username, data);
    }
    let g = Getch::new();
    let _ = g.getch().unwrap();
    println!("Premi qualsiasi tasto per uscire.");
}

fn read_config() -> serde_json::value::Value {
    let default = json!({"difficulty": 2});
    match fs::File::open("config.json") {
        Ok(file) => match serde_json::from_reader(file) {
            Ok(json) => json,
            Err(_) => default,
        },
        Err(_) => default,
    }
}
fn hashmapper(data: serde_json::value::Value) -> HashMap<String, serde_json::Value> {
    data.as_object()
        .unwrap()
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}
fn main() {
    let config = read_config();
    // if first Access or without account this screen will show up
    let mut hashmap = hashmapper(config);
    if !hashmap.contains_key("username") {
        match login(&mut hashmap) {
            (Access::Granted, user) => {
                main_menu(user, hashmap);
            }
            _ => unreachable!(),
        }
    } else {
        let user = User {
            username: hashmap.get("username").unwrap().to_string(),
        };
        main_menu(user, hashmap);
    }
}
