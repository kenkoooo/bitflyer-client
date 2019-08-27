use bitflyer_client::HttpBitFlyerClient;
use rusqlite::Connection;
use rusqlite::NO_PARAMS;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() >= 2);

    let conn = Connection::open(&args[1])?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bit_history (
             id INTEGER PRIMARY KEY,
             json TEXT DEFAULT NULL
         )",
        NO_PARAMS,
    )?;

    let mut statement = conn.prepare("SELECT id FROM bit_history ORDER BY id LIMIT 1")?;
    let mut current_minimum = statement
        .query_map(NO_PARAMS, |row| row.get::<_, i64>(0))?
        .flat_map(|row| row)
        .min();

    let client = HttpBitFlyerClient::default();
    for _ in 0..3 {
        let history = client.fetch_history(current_minimum)?;
        for history in history.iter() {
            let json = serde_json::to_string(history)?;
            conn.execute(
                "INSERT INTO bit_history (id, json) values (?1, ?2)",
                &[&history.id.to_string(), &json],
            )?;
        }

        let min = history.into_iter().map(|history| history.id).min().unwrap();
        current_minimum = Some(min - 1);
        eprintln!("{:?}", current_minimum);
    }

    Ok(())
}
