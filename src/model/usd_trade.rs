use chrono::DateTime;
use chrono_tz::Tz;

use crate::model::{trade::Trade, Currency};

const BITCOINTAX_INPUT_COLUMNS: &str = "Date,Action,Source,Symbol,Volume,Price,Currency,Fee";
const BITCOINTAX_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";

/// SELL/BUY USD trade used as input for bitcoin.tax platform.
/// This approach treats crypto-to-crypto trades as pairs of crypto-USD and USD-crypto ones.
/// TODO: investigate further how precise it is when exchanges take fees in coins but not in USD.
pub struct UsdTrade {
    datetime: DateTime<Tz>,
    is_buy: bool,
    exchange_name: String,
    currency: Currency,
    volume: f64,
    price_usd: f64,
    fees_usd: f64,
}

impl UsdTrade {
    pub fn from_trade(trade: &Trade, is_buy: bool) -> Self {
        let datetime = trade.datetime.clone();
        let exchange_name = trade.exchange_name.clone();
        let currency;
        let volume;
        let price_usd;
        let fees_usd;

        if is_buy {
            currency = trade.currency_to.clone();
            volume = trade.volume_to;
            price_usd = trade.currency_to_price_usd;
            fees_usd = trade.fees_usd();
        } else {
            currency = trade.currency_from.clone();
            volume = trade.volume_from;
            price_usd = trade.currency_from_price_usd;
            // Fees are included in the cost basis of currency_to if not USD.
            fees_usd = if trade.currency_to == "USD" {
                trade.fees_usd()
            } else {
                0.0
            };
        }

        Self {
            datetime,
            is_buy,
            exchange_name,
            currency,
            volume,
            price_usd,
            fees_usd,
        }
    }

    pub fn print_header() {
        println!("{}", BITCOINTAX_INPUT_COLUMNS);
    }

    pub fn print(&self) {
        let dt = self.datetime.format(BITCOINTAX_TIME_FORMAT);
        let action = if self.is_buy { "BUY" } else { "SELL" };
        println!(
            "{},{},{},{},{:.9},{:.9},USD,{:.9}",
            dt,
            action,
            self.exchange_name,
            self.currency,
            self.volume,
            self.price_usd,
            self.fees_usd,
        );
    }
}
