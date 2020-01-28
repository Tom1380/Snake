mod config;
mod game;
mod scores;

use {
    config::*,
    dialoguer::{theme::ColorfulTheme, Select},
    fs::File,
    game::*,
    getch::*,
    reqwest,
    scores::*,
    std::{fs, process::exit},
};

const DIFFICULTIES: &'static [&'static str] = &[
    "Mi Faccio Le 2006",
    "Snitch",
    "Pick-Up Coach",
    "Sono Russo Dentro",
    "Iran 2020",
];

fn clear_screen() {
    if cfg!(target_os = "windows") {
        println!("{}", "\n".repeat(30));
    } else {
        print!("\x1b[2J\x1b[1;1H");
    }
}

fn main_menu(mut config: &mut Config) {
    loop {
        clear_screen();
        print_snake_ascii_art();
        let selections = &["Gioca", "Impostazioni", "Punteggi", "Esci"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(
                format!(
"Sviluppato da Tommaso TC e Dilec P.
Ringraziamento speciale al creative designer Andrea B,
al nostro primo beta tester, che ci ha creduto fin dal giorno zero: Lorenzo S
e al nostro pluripremiato bug fixer: l'egregio signor Giorgio M.

Quando giochi, usa WASD per muoverti.
Difficolta' impostata: {}
Eccoti, {}",
                    DIFFICULTIES[config.difficulty], config.username
                )
                .as_str(),
            )
            .default(0)
            .items(&selections[..])
            .interact_opt()
            .unwrap();

        if let Some(selection) = selection {
            match selection {
                0 => play(&config),
                1 => settings(&mut config),
                2 => scores(&config),
                3 => exit(0),
                _ => unreachable!(),
            }
        }
    }
}

fn settings(config: &mut Config) {
    println!("");
    let selections = DIFFICULTIES;
    clear_screen();
    config.difficulty = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Difficolta'")
        .default(config.difficulty)
        .items(&selections[..])
        .interact_opt()
        .unwrap()
        .unwrap();
    serde_json::to_writer(&File::create("config.json").unwrap(), &config).unwrap();
}

fn print_snake_ascii_art() {
    println!(".d8888b. 88d888b. .d8888b. 88  .dP  .d8888b.\nY8ooooo. 88'  `88 88'  `88 88888\"   88____d8\n      88 88    88 88.  .88 88  `8b. 88  \n`88888P' dP    dP `88888P8 dP   `YP `88888P'");
}

fn main() {
    clear_screen();
    let mut config = read_config();
    main_menu(&mut config);
}
