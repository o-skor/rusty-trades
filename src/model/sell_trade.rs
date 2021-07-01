use std::fmt;

use chrono::{DateTime, Datelike, Duration, TimeZone};
use chrono_tz::Tz;

use crate::{
    model::{trade::Trade, Currency},
    utils::time_utils::{datetime_to_str, APP_TZ},
};

/// Sell trade information for Form 8949.
pub struct SellTrade {
    pub volume: f64,
    pub currency: Currency,
    pub cost_basis: f64, // volume * price_bought_usd + fees
    pub proceeds: f64,   // volume * price_sold_usd - fees (0 if crypto-to-crypto)
    pub buy_trade_idx: usize,
    pub sell_trade_idx: usize,
    buy_datetime: DateTime<Tz>,
    pub sell_datetime: DateTime<Tz>,
}

impl SellTrade {
    pub fn new(
        volume: f64,
        cost_basis: f64,
        buy_trade: &Trade,
        sell_trade: &Trade,
        buy_trade_idx: usize,
        sell_trade_idx: usize,
    ) -> Self {
        let fees = if sell_trade.currency_to == "USD" {
            (volume / sell_trade.volume_from) * sell_trade.fees_usd()
        } else {
            0.0
        };
        let proceeds = volume * sell_trade.currency_from_price_usd - fees;
        let currency = sell_trade.currency_from.clone();
        let buy_datetime = buy_trade.datetime.clone();
        let sell_datetime = sell_trade.datetime.clone();
        Self {
            volume,
            currency,
            cost_basis,
            proceeds,
            buy_trade_idx,
            sell_trade_idx,
            buy_datetime,
            sell_datetime,
        }
    }

    pub fn is_long_term(&self) -> bool {
        let mut dt = APP_TZ
            .ymd(
                self.buy_datetime.year(),
                self.buy_datetime.month(),
                self.buy_datetime.day(),
            )
            .and_hms(0, 0, 0)
            + Duration::days(1);
        if dt.month() == 2 && dt.day() == 29 {
            // leap_year/02/29 is a special case.
            dt = dt + Duration::days(1);
        }
        let dt_can_sell_lt = APP_TZ
            .ymd(dt.year() + 1, dt.month(), dt.day())
            .and_hms(0, 0, 0);
        self.sell_datetime >= dt_can_sell_lt
    }

    pub fn gain(&self) -> f64 {
        self.proceeds - self.cost_basis
    }
}

/// Display information as per Form 8949 format.
impl fmt::Display for SellTrade {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:.9} {} {} {} {:.9} {:.9} {:.9}",
            self.volume,
            self.currency,
            datetime_to_str(&self.buy_datetime),
            datetime_to_str(&self.sell_datetime),
            self.proceeds,
            self.cost_basis,
            self.gain(),
        )
    }
}
