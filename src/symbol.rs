use chrono::{DateTime, Utc, SecondsFormat};
use std::fmt;
use yahoo_finance_api::{YahooConnector, Quote};
use crossbeam::channel::{Sender};
use std::thread;
#[derive(Debug)]
pub struct TickerInfo {
    range: Range,
    symbol: String,
    close: f64,
    delta: f64,
    min: f64,
    max: f64,
    sma_30: f64
}

#[derive(Debug, Copy, Clone)]
pub struct Range{
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>
}



fn process_quotes(symbol: &str, range: Range, data: Vec<Quote>) -> TickerInfo{

    let mut info = TickerInfo::new(symbol, range);
    let mut min: f64 = f64::MAX;
    let mut max: f64 = f64::MIN;
    //let closes: [f64] = data.iter().map(|q|{q.close}).collect();
    let mut closes: Vec<f64>= Vec::new();
    //info.sma_30 = calculate_sma(&closes);

    let open: f64 = data.first().unwrap().open;
    let close = data.last().unwrap().adjclose;

    for q in data.iter() {
        if q.low < min{
            min = q.low
        }
        if q.high > max{
            max = q.high
        }
        closes.push(q.adjclose);
        if closes.len() > 30{
            closes.remove(0);
        }
        info.close = q.adjclose;
    }
    info.max = max;
    info.min = min;
    let last_30_sum: f64 = closes.iter().sum();
    info.sma_30 = last_30_sum / closes.len() as f64;
    info.delta = (close - open) / close;
    info
}

impl TickerInfo {
    fn new(symbol: &str, range: Range) -> TickerInfo{
        TickerInfo{
            range,
            symbol: String::from(symbol),
            close: 0.0,
            delta: 0.0,
            min: 0.0,
            max: 0.0,
            sma_30: 0.0
        }
    }
    pub fn query_info(symbol: String, range: Range, tx: Sender<Result<TickerInfo, String>>){
        thread::spawn(move|| {
            let connector = YahooConnector::new();
            let response = connector.get_quote_history_interval(symbol.as_str(), range.start, range.end, "1d");
            let result = match response {
                Err(e) => Err(e.to_string()),
                Ok(resp) =>  match resp.quotes(){
                    Err(e) => Err(e.to_string()),
                    Ok(data) => Ok(process_quotes(symbol.as_str(), range, data))
                }
            };
            tx.send(result).unwrap();
        });
    }

    pub fn print_columns() -> &'static str{
        "period start,period end,symbol,price,change %,min,max,30d avg"
    }
}

impl fmt::Display for TickerInfo{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},{},${:.2},{:.2}%,${:.2},${:.2},${:.2}",
               self.range.start.to_rfc3339_opts(SecondsFormat::Secs, false), self.range.end.to_rfc3339_opts(SecondsFormat::Secs, false), self.symbol, self.close, self.delta, self.min, self.max, self.sma_30)
    }
}