use chrono::DateTime;
use chrono_tz::Tz;

use crate::{
    model::{trade::load_trades, wallet::Wallet},
    utils::time_utils::{end_of_the_day, start_of_the_day},
};

pub struct Config {
    pub input_file: String,
    pub dt_from: Option<DateTime<Tz>>,
    pub dt_to: Option<DateTime<Tz>>,
}

pub fn run(config: Config) {
    let mut wallet = Wallet::default();

    let trades = load_trades(&config.input_file);
    for trade in trades {
        wallet.add_trade(trade);
    }

    let dt_from = config
        .dt_from
        .or_else(|| Some(start_of_the_day(&wallet.trades.first().unwrap().datetime)))
        .unwrap();
    let dt_to = config
        .dt_to
        .or_else(|| Some(end_of_the_day(&wallet.trades.last().unwrap().datetime)))
        .unwrap();

    wallet.print_trades(true, &dt_from, &dt_to);
    wallet.print_usd_trades(&dt_from, &dt_to);
    wallet.print_sell_trades(true, &dt_from, &dt_to);
    wallet.print_proceeds(&dt_from, &dt_to);
    wallet.print_holdings();
}
