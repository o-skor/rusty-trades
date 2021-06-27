mod model;
mod utils;

use model::{trade::load_trades, wallet::Wallet};

fn main() {
    let mut wallet = Wallet::default();

    let trades = load_trades();
    for trade in trades {
        wallet.add_trade(trade);
    }

    wallet.print_trades(true);
    wallet.print_usd_trades();
    wallet.print_holdings();
}
