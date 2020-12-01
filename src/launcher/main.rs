use {
    dialoguer::{theme::ColorfulTheme, Select},
    game_arcade::clear_screen,
    reqwest, serde_json,
    std::{collections::HashMap, process::Command},
};

fn main() {
    loop {
        clear_screen();
        println!(
            "IL FOTTUTO LAUNCHER, BITCH. I TUOI CAZZO DI DEVELOPERS SONO TORNATI, NON TE L'ASPETTAVI."
        );
        let games = downloadable_games();
        let selections: Vec<&String> = games.keys().collect();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .default(0)
            .items(&selections[..])
            .interact_opt()
            .unwrap()
            .unwrap();
        open_game(&games[selections[selection]]);
    }
}

fn downloadable_games() -> HashMap<String, String> {
    reqwest::get("http://167.172.50.64/available_games")
        .unwrap()
        .json::<serde_json::Value>()
        .unwrap()
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), v.as_str().unwrap().to_string()))
        .collect()
}

fn download_game(name: &String) {

}

fn open_game(name: &String) {
    Command::new(format!("games/{}", name))
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}
