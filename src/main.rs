mod model;
mod utils;

use chrono::TimeZone;

use crate::{
    model::{trade::load_trades, wallet::Wallet},
    utils::time_utils::APP_TZ,
};

fn main() {
    let mut wallet = Wallet::default();

    let trades = load_trades();
    for trade in trades {
        wallet.add_trade(trade);
    }

    let dt_from = APP_TZ.ymd(2017, 1, 1).and_hms(0, 0, 0);
    let dt_to = APP_TZ.ymd(2021, 12, 31).and_hms(23, 59, 59);

    wallet.print_trades(true, &dt_from, &dt_to);
    wallet.print_usd_trades(&dt_from, &dt_to);
    wallet.print_sell_trades(true, &dt_from, &dt_to);
    wallet.print_proceeds(&dt_from, &dt_to);
    wallet.print_holdings();
}
