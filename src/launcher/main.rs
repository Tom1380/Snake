use {
    dialoguer::{theme::ColorfulTheme, Select},
    reqwest, serde_json,
};

fn main() {
    loop {
        println!(
            "IL FOTTUTO LAUNCHER, BITCH. I TUOI CAZZO DI DEVELOPERS SONO TORNATI, NON TE L'ASPETTAVI."
        );
        let games = read_list_of_available_games().unwrap();
        let games = games.as_object().unwrap();
        let selections: Vec<&String> = games.keys().collect();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .default(0)
            .items(&selections[..])
            .interact_opt()
            .unwrap()
            .unwrap();
        //open_game(&games[selections[selection]]);
    }
}

fn read_list_of_available_games() -> Option<serde_json::Value> {
    match reqwest::get("http://167.172.50.64/available_games") {
        Ok(mut response) => match response.json() {
            Ok(json) => json,
            Err(_) => None,
        },
        Err(_) => None,
    }
}

fn open_game(name: &String) {

}