use bitflyer_client::HttpBitFlyerClient;
use rusqlite::Connection;
use rusqlite::NO_PARAMS;
use std::env;
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let db_filepath = args.get(1).expect("Specify .db filepath.");

    let mut conn = Connection::open(db_filepath)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS bit_history (
             id INTEGER PRIMARY KEY,
             json TEXT DEFAULT NULL
         )",
        NO_PARAMS,
    )?;

    let mut current_minimum_id =
        conn.query_row("SELECT MIN(id) FROM bit_history;", NO_PARAMS, |row| {
            row.get(0)
        })?;

    let client = HttpBitFlyerClient::default();
    loop {
        match client.fetch_history(current_minimum_id) {
            Ok(history) => {
                let tx = conn.transaction()?;
                for history in history.iter() {
                    let json = serde_json::to_string(history)?;
                    tx.execute(
                        "INSERT INTO bit_history (id, json) values (?1, ?2)",
                        &[&history.id.to_string(), &json],
                    )?;
                }
                tx.commit()?;

                let min = history.into_iter().map(|history| history.id).min().unwrap();
                current_minimum_id = Some(min - 1);
                eprintln!("{:?}", current_minimum_id);
            }
            Err(e) => {
                eprintln!("{:?}", e);
                sleep(Duration::from_secs(1));
            }
        }
    }
}
