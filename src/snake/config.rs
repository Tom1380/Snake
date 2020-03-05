use {
    dialoguer::Input,
    serde::{Deserialize, Serialize},
    std::fs::File,
};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub username: String,
    pub difficulty: usize,
}

pub fn read_config() -> Config {
    let default = || {
        let conf = Config {
            username: login(),
            difficulty: 2,
        };
        let _ = serde_json::to_writer(&File::create("config.json").unwrap(), &conf);
        conf
    };
    match File::open("config.json") {
        Ok(file) => match serde_json::from_reader(file) {
            Ok(json) => match serde_json::from_value(json) {
                Ok(conf) => conf,
                Err(_) => default(),
            },
            Err(_) => default(),
        },
        Err(_) => default(),
    }
}

fn login() -> String {
    Input::new()
        .with_prompt("Il tuo username")
        .interact()
        .unwrap()
}
