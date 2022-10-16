use crate::config::*;
use crate::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Score {
    username: String,
    score: usize,
    date: String,
}

pub fn scores(config: &Config) {
    println!("");
    let client = reqwest::blocking::Client::new();
    match client
        .get(&format!(
            "http://167.172.50.64/snake/scores/{}",
            config.difficulty
        ))
        .send()
    {
        Ok(response) => match response.json::<serde_json::Value>() {
            Ok(scores) => match scores.as_array() {
                Some(scores) => print_scores(scores, &config),
                _ => println!("Qualcosa è andato storto."),
            },
            _ => println!("Qualcosa è andato storto."),
        },
        _ => println!("Qualcosa è andato storto."),
    }

    println!("Premi qualsiasi tasto per uscire.");
    let g = Getch::new();
    let _ = g.getch().unwrap();
}

fn print_scores(scores: &Vec<serde_json::Value>, config: &Config) {
    if scores.len() == 0 {
        println!(
            "Nessun punteggio per la difficoltà {}.",
            DIFFICULTIES[config.difficulty]
        );
    } else {
        println!(
            "Punteggi per la difficoltà {}:",
            DIFFICULTIES[config.difficulty]
        );
        let scores: Vec<Score> = match scores
            .iter()
            .map(move |s| serde_json::from_value(s.clone()))
            .collect()
        {
            Ok(scores) => scores,
            Err(_) => {
                println!("Qualcosa è andato storto.");
                return;
            }
        };
        let max_score_digits = scores
            .iter()
            .map(|s| s.score)
            .max()
            .unwrap()
            .to_string()
            .len();
        let max_username_length = scores.iter().map(|s| s.username.len()).max().unwrap();
        println!("");
        // How many '-' to print between the lines to separate them perfectly.
        let separator = "-".repeat(max_score_digits + max_username_length + 39);
        for (score, i) in scores.iter().zip(1..=10) {
            let space_after_score =
                " ".repeat(max_score_digits - score.score.to_string().len() + 1);
            let space_after_name = " ".repeat(max_username_length - score.username.len() + 1);
            if i != 10 {
                println!(
                    "{})  {}{}- {}{}- {}",
                    i, score.score, space_after_score, score.username, space_after_name, score.date
                );
            } else {
                println!(
                    "{}) {}{}- {}{}- {}",
                    i, score.score, space_after_score, score.username, space_after_name, score.date
                );
            }
            println!("{}", separator);
        }
    }
}
