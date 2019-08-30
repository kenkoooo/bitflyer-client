use bitflyer_client::ExchangeHistory;
use rusqlite::{Connection, NO_PARAMS};
use std::collections::VecDeque;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;

struct HistoryRow {
    sell_history: Vec<i64>,
    buy_history: Vec<i64>,
    side: Side,
    price: i64,
}

enum Side {
    BUY,
    SELL,
}
impl Side {
    fn value(&self) -> &'static str {
        match self {
            Side::BUY => "0",
            Side::SELL => "1",
        }
    }
}

const COLUMN_NUM: usize = 250;

fn main() -> Result<(), Box<dyn Error>> {
    eprintln!("Loading from SQL...");
    let conn = Connection::open("history.db")?;
    let mut history = conn
        .prepare("SELECT * FROM bit_history")?
        .query_map::<String, _, _>(NO_PARAMS, |row| row.get("json"))?
        .flat_map(|result| result)
        .flat_map(|json| serde_json::from_str::<ExchangeHistory>(&json))
        .filter(|history| history.side.as_str() == "SELL" || history.side.as_str() == "BUY")
        .collect::<Vec<_>>();

    eprintln!("Generating table...");
    history.sort_by_key(|h| h.id);
    let mut sell: VecDeque<ExchangeHistory> = VecDeque::new();
    let mut buy: VecDeque<ExchangeHistory> = VecDeque::new();
    let mut result = vec![];
    for history in history.into_iter() {
        if sell.len() == COLUMN_NUM && buy.len() == COLUMN_NUM {
            let side = if history.side.as_str() == "SELL" {
                Side::SELL
            } else {
                Side::BUY
            };
            let head_sell_price = sell.iter().next().unwrap().price;
            let head_buy_price = buy.iter().next().unwrap().price;
            let head_price = (head_sell_price + head_buy_price) / 2.0;
            let sell_history = sell.iter().map(|h| (h.price - head_price) as i64).collect();
            let buy_history = buy.iter().map(|h| (h.price - head_price) as i64).collect();
            let row = HistoryRow {
                sell_history,
                buy_history,
                side,
                price: (history.price - head_price) as i64,
            };
            result.push(row);
        }

        match history.side.as_str() {
            "SELL" => {
                sell.push_back(history);
                if sell.len() > COLUMN_NUM {
                    sell.pop_front();
                }
            }
            "BUY" => {
                buy.push_back(history);
                if buy.len() > COLUMN_NUM {
                    buy.pop_front();
                }
            }
            _ => unreachable!(),
        }
    }

    eprintln!("Writing to a CSV file...");
    let mut file = File::create("history.csv")?;
    let mut file = BufWriter::new(&mut file);
    file.write_all((format_header()).as_bytes())?;
    file.write_all("\n".as_bytes())?;

    for (i, history) in result.iter().enumerate() {
        let row = format_row(history);
        file.write_all(row.as_bytes())?;
        file.write_all("\n".as_bytes())?;
        if (i + 1) % 1000 == 0 {
            eprintln!("{}/{}", i + 1, result.len());
        }
    }

    Ok(())
}

fn format_header() -> String {
    let sell = (0..COLUMN_NUM).fold(String::new(), |s, i| {
        s + "sell_" + i.to_string().as_str() + ","
    });
    let buy = (0..COLUMN_NUM).fold(String::new(), |s, i| {
        s + "buy_" + i.to_string().as_str() + ","
    });
    sell + buy.as_str() + "side,price"
}

fn format_row(row: &HistoryRow) -> String {
    let sell = row
        .sell_history
        .iter()
        .fold(String::new(), |s, v| s + v.to_string().as_str() + ",");
    let buy = row
        .buy_history
        .iter()
        .fold(String::new(), |s, v| s + v.to_string().as_str() + ",");
    sell + buy.as_str() + row.side.value() + "," + row.price.to_string().as_str()
}
