use {
    game_arcade::clear_screen,
    std::{io, io::Write},
};

fn input(request: &str) -> String {
    let mut input = String::new();
    print!("{}", request);
    let _ = io::stdout().flush();
    io::stdin()
        .read_line(&mut input)
        .expect("failed to read from stdin");
    input.trim().to_string()
}

pub fn users() -> Vec<String> {
    clear_screen();
    loop {
        match input("Quanti sono gli utenti : ").parse::<u32>() {
            Ok(i) if i > 1 => break (0..i).map(|_| input("Nome : ")).collect(),
            Err(_) => clear_screen(),
            _ => {
                clear_screen();
                println!("Giocatori insufficienti (Min. 2 players)")
            }
        };
    }
}
