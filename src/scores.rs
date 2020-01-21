use crate::*;

pub fn scores(config: &HashMap<String, serde_json::Value>) {
    println!("");
    let client = reqwest::Client::new();
    let difficulty = config["difficulty"].as_i64().unwrap() as usize;
    match client
        .get(&format!("http://167.172.50.64/scores/{}", difficulty))
        .send()
    {
        Ok(mut response) => match response.json::<serde_json::Value>() {
            Ok(scores) => match scores.as_array() {
                Some(scores) => print_scores(&scores, &difficulty),
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

fn print_scores(scores: &Vec<serde_json::Value>, difficulty: &usize) {
    if scores.len() == 0 {
        println!(
            "Nessun punteggio per la difficoltà {}.",
            DIFFICULTIES[*difficulty]
        );
    } else {
        println!("Punteggi per la difficoltà {}:", DIFFICULTIES[*difficulty]);
    }
    for (i, score) in scores.iter().map(|j| hashmapper(j.clone())).enumerate() {
        let username = score["username"].as_str().unwrap();
        let date = score["date"].as_str().unwrap();
        let score = score["score"].as_i64().unwrap();
        println!("{}) {} - {} - {}", i + 1, score, username, date);
    }
}

pub fn absolute_and_personal_high_score(
    config: &HashMap<String, serde_json::Value>,
) -> (Option<usize>, Option<usize>) {
    let client = reqwest::Client::new();
    let difficulty = config["difficulty"].as_i64().unwrap() as usize;
    let username = config["username"].as_str().unwrap();
    match client
        .get(&format!(
            "http://167.172.50.64/absolute_and_personal_high_score/{}/{}",
            difficulty, username
        ))
        .send()
    {
        Ok(mut response) => match response.json::<serde_json::Value>() {
            Ok(scores) => (
                scores["absolute"].as_i64().map(|i| i as usize),
                scores["personal"].as_i64().map(|i| i as usize),
            ),
            _ => (None, None),
        },
        _ => (None, None),
    }
}
