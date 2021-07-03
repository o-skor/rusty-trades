use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::BufReader;

use chrono::DateTime;
use chrono_tz::Tz;
use lazy_static::lazy_static;
use rand::{seq::SliceRandom, Rng};
use regex::Regex;

use crate::{
    model::Currency,
    utils::{
        read_all_lines,
        time_utils::{datetime_from_str, datetime_to_str, generate_random_datetime},
    },
};

pub struct Trade {
    pub datetime: DateTime<Tz>,
    pub exchange_name: String,
    pub volume_from: f64,
    pub currency_from: Currency,
    pub currency_from_price_usd: f64,
    pub volume_to: f64,
    pub currency_to: Currency,
    pub currency_to_price_usd: f64,
    pub notes: Vec<String>,
}

impl Trade {
    pub fn fees_usd(&self) -> f64 {
        let expected_volume_to =
            (self.volume_from * self.currency_from_price_usd) / self.currency_to_price_usd;
        let diff_volume = expected_volume_to - self.volume_to;
        diff_volume * self.currency_to_price_usd
    }

    pub fn fees_percent(&self) -> f64 {
        let expected_volume_to =
            (self.volume_from * self.currency_from_price_usd) / self.currency_to_price_usd;
        let diff_volume = expected_volume_to - self.volume_to;
        diff_volume * 100.0 / expected_volume_to
    }

    pub fn parse(line: &str) -> Self {
        lazy_static! {
            static ref RE: Regex = Regex::new(concat!(
                r"^\[(?P<datetime>.+)\] ",
                r"(?P<volume_from>\S+) (?P<currency_from>\w+) => ",
                r"(?P<volume_to>\S+) (?P<currency_to>\w+) \(",
                r"(?P<currency_from_2>\w+)=(?P<currency_from_price_usd>\S+), ",
                r"(?P<currency_to_2>\w+)=(?P<currency_to_price_usd>\S+), ",
                r"(?P<exchange_name>\w+)\)$"
            ))
            .unwrap();
        }

        let caps = RE.captures(line).unwrap();
        let datetime = datetime_from_str(&caps["datetime"]).unwrap();
        let volume_from: f64 = caps["volume_from"].parse().unwrap();
        let currency_from = caps["currency_from"].to_string();
        let volume_to: f64 = caps["volume_to"].parse().unwrap();
        let currency_to = caps["currency_to"].to_string();
        assert_eq!(currency_from, &caps["currency_from_2"]);
        let currency_from_price_usd: f64 = caps["currency_from_price_usd"].parse().unwrap();
        assert_eq!(currency_to, &caps["currency_to_2"]);
        let currency_to_price_usd: f64 = caps["currency_to_price_usd"].parse().unwrap();
        let exchange_name = caps["exchange_name"].to_string();
        let notes = vec![];

        Self {
            datetime,
            exchange_name,
            volume_from,
            currency_from,
            currency_from_price_usd,
            volume_to,
            currency_to,
            currency_to_price_usd,
            notes,
        }
    }
}

impl fmt::Display for Trade {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{}] {:.9} {} => {:.9} {} ({}={:.9}, {}={:.9}, {})",
            datetime_to_str(&self.datetime),
            self.volume_from,
            self.currency_from,
            self.volume_to,
            self.currency_to,
            self.currency_from,
            self.currency_from_price_usd,
            self.currency_to,
            self.currency_to_price_usd,
            self.exchange_name
        )
    }
}

impl fmt::Debug for Trade {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{}] {:.9} {} => {:.9} {} ({}={:.9}, {}={:.9}, {}, fees=[{:.9} USD, {:.9}%])",
            datetime_to_str(&self.datetime),
            self.volume_from,
            self.currency_from,
            self.volume_to,
            self.currency_to,
            self.currency_from,
            self.currency_from_price_usd,
            self.currency_to,
            self.currency_to_price_usd,
            self.exchange_name,
            self.fees_usd(),
            self.fees_percent(),
        )
    }
}

pub fn load_trades(filename: &str) -> Vec<Trade> {
    let reader = BufReader::new(File::open(filename).unwrap());
    let lines = read_all_lines(reader).unwrap();
    let mut trades = vec![];

    let mut i = 0;
    while i < lines.len() {
        let mut t = Trade::parse(&lines[i]);
        i += 1;
        while i < lines.len() && lines[i].starts_with("--- ") {
            let note = &lines[i][4..];
            t.notes.push(note.to_string());
            i += 1;
        }
        trades.push(t);
    }

    trades
}

#[allow(dead_code)]
pub fn generate_random_consistent_trades(
    n: usize,
    dt_from: &DateTime<Tz>,
    dt_to: &DateTime<Tz>,
) -> Vec<Trade> {
    let mut rng = rand::thread_rng();

    let mut datetimes: Vec<DateTime<Tz>> = (0..n)
        .map(|_| generate_random_datetime(dt_from, dt_to, &mut rng))
        .collect();
    datetimes.sort();

    const CURRENCIES: &[&str] = &["USD", "BTC", "ETH", "DOGE"];
    const FEE_MULTS: &[f64] = &[0.99, 0.995, 0.999];
    const EXCHANGES: &[&str] = &["Exchange_1", "Exchange_2", "Exchange_3"];
    const EPS: f64 = 1e-5;

    let mut wallet: HashMap<Currency, f64> = HashMap::new();
    let mut trades: Vec<Trade> = Vec::new();

    for i in 0..n {
        let mut can_sell: Vec<String> = Vec::new();
        for (currency, &volume) in wallet.iter() {
            if volume > EPS {
                can_sell.push(currency.to_owned());
            }
        }
        let is_sell = !can_sell.is_empty() && rng.gen_range(0..1) == 0;
        let currency_from: &str;
        let currency_from_price_usd: f64;
        let volume_from: f64;
        let currency_to: &str;
        let currency_to_price_usd: f64;

        if is_sell {
            currency_from = can_sell.choose(&mut rng).unwrap();
            currency_from_price_usd = if currency_from == "USD" {
                1.0
            } else {
                rng.gen_range(1.0..100.0)
            };
            let volume_have = *wallet.get(currency_from).unwrap();
            volume_from = rng.gen_range(0.1 * volume_have..volume_have);
            let mut c = currency_from;
            while c == currency_from {
                c = CURRENCIES.choose(&mut rng).unwrap();
            }
            currency_to = c;
            currency_to_price_usd = if currency_to == "USD" {
                1.0
            } else {
                rng.gen_range(1.0..100.0)
            };
        } else {
            currency_from = "USD";
            currency_from_price_usd = 1.0;
            volume_from = rng.gen_range(10.0..1000.0);
            let mut c = currency_from;
            while c == currency_from {
                c = CURRENCIES.choose(&mut rng).unwrap();
            }
            currency_to = c;
            currency_to_price_usd = rng.gen_range(1.0..100.0);
        }
        let volume_to_full = (volume_from * currency_from_price_usd) / currency_to_price_usd;
        let volume_to = volume_to_full * FEE_MULTS.choose(&mut rng).unwrap();

        if is_sell {
            let volume_from_cur = wallet.get_mut(currency_from).unwrap();
            assert!(*volume_from_cur > volume_from);
            *volume_from_cur -= volume_from;
        }
        let volume_to_cur = wallet.entry(currency_to.to_owned()).or_insert(0.0);
        *volume_to_cur += volume_to;

        let datetime = datetimes[i];
        let exchange_name = (*EXCHANGES.choose(&mut rng).unwrap()).to_owned();
        let currency_from = currency_from.to_owned();
        let currency_to = currency_to.to_owned();
        let notes = Vec::new();

        trades.push(Trade {
            datetime,
            exchange_name,
            volume_from,
            currency_from,
            currency_from_price_usd,
            volume_to,
            currency_to,
            currency_to_price_usd,
            notes,
        });
    }

    trades
}
