mod symbol;

extern crate yahoo_finance_api;
extern crate clap;
extern crate chrono;
extern crate crossbeam;

use clap::{App, Arg};

use chrono::prelude::*;
use crossbeam::channel::{unbounded};

use crate::symbol::{Range, TickerInfo};


fn main() {
    let args = App::new("Stock Tracking cli")
        .about("A tool to track stock prices")
        .arg(Arg::with_name("from").required(true).help("rfc3339 (yyyy-MM-dd) date").default_value("2021-01-30"))
        .arg(Arg::with_name("tickers").possible_values(&["MSFT", "GOOG", "AAPL", "UBER", "IBM"]).required(true).multiple(true).min_values(1).default_value("AAPL"))
        .get_matches();
    let from = args.value_of("from").unwrap();
    let start = match NaiveDate::parse_from_str(from, "%Y-%m-%d"){
        Err(e) =>{
            eprintln!("invalid from date {:?}", e);
            return;
        },
        Ok(date) => DateTime::<Utc>::from_utc(NaiveDateTime::new(date, NaiveTime::from_hms_milli(23, 59, 59, 999)), Utc),
    };
    let tickers = args.values_of("tickers").unwrap();


    let end = Utc::now();

    let range = Range{end, start};

    let (tx, rx) = unbounded();

    let len = tickers.len();

    for symbol in tickers {
        TickerInfo::query_info(String::from(symbol), range, tx.clone());
    }
    let mut info = Vec::<TickerInfo>::new();
    for _ in 0..len{
        let result = rx.recv().unwrap();
        match result{
            Err(e) => eprintln!("{}", e),
            Ok(ti) => info.push(ti)
        }
    }
    println!("{}", TickerInfo::print_columns());
    for x in info {
        println!("{}", x)
    }
}
