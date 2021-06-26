use std::collections::HashMap;

use super::{trade::Trade, Currency};
use crate::utils::time_utils::datetime_to_str;

struct HoldingsItem {
    volume: f64,
    cost_basis: f64,
    trades_idx: usize,
}

#[derive(Default)]
pub struct Wallet {
    trades: Vec<Trade>,
    holdings: HashMap<Currency, Vec<HoldingsItem>>,
}

impl Wallet {
    pub fn add_trade(&mut self, trade: Trade) {
        const EPS: f64 = 1e-5;

        if trade.currency_from != "USD" {
            let holdings_bucket = self.holdings.get_mut(&trade.currency_from).unwrap();
            let mut volume_to_sell = trade.volume_from;

            while volume_to_sell > EPS {
                let volume_sold: f64;
                let cost_basis_sold: f64;

                if holdings_bucket[0].volume > volume_to_sell + EPS {
                    volume_sold = volume_to_sell;
                    cost_basis_sold = holdings_bucket[0].cost_basis
                        * (volume_to_sell / holdings_bucket[0].volume);
                    holdings_bucket[0].volume -= volume_sold;
                    holdings_bucket[0].cost_basis -= cost_basis_sold;
                } else {
                    volume_sold = holdings_bucket[0].volume;
                    // cost_basis_sold = holdings_bucket[0].cost_basis;
                    holdings_bucket.remove(0);
                }

                volume_to_sell -= volume_sold;
            }
        }

        if trade.currency_to != "USD" {
            let holdings_bucket = self
                .holdings
                .entry(trade.currency_to.to_owned())
                .or_insert(Vec::new());
            let cost_basis = (trade.volume_to * trade.currency_to_price_usd) + trade.fees_usd();
            holdings_bucket.push(HoldingsItem {
                volume: trade.volume_to,
                cost_basis,
                trades_idx: self.trades.len(),
            });
        }

        self.trades.push(trade);
    }

    pub fn print_trades(&self, print_notes: bool) {
        for trade in &self.trades {
            println!("{}", trade);
            if print_notes {
                for note in &trade.notes {
                    println!("--- {}", note);
                }
            }
        }
    }

    pub fn print_holdings(&self) {
        struct BucketInfo {
            total_volume: f64,
            total_cost_basis: f64,
            currency: Currency,
        }
        let mut infos: Vec<BucketInfo> = Vec::new();
        for (currency, holdings_bucket) in &self.holdings {
            let mut info = BucketInfo {
                total_volume: 0.0,
                total_cost_basis: 0.0,
                currency: currency.to_owned(),
            };
            for item in holdings_bucket {
                info.total_volume += item.volume;
                info.total_cost_basis += item.cost_basis;
            }
            infos.push(info);
        }
        infos.sort_by(|info_1, info_2| {
            info_2
                .total_cost_basis
                .partial_cmp(&info_1.total_cost_basis)
                .unwrap()
        });
        for info in infos {
            // avg_cost may differ from average price because cost basis includes fees.
            println!(
                "{}: volume={:.9}, cost_basis={:.9}, avg_cost={:.9}",
                info.currency,
                info.total_volume,
                info.total_cost_basis,
                info.total_cost_basis / info.total_volume,
            );
            for item in self.holdings.get(&info.currency).unwrap() {
                let trade = &self.trades[item.trades_idx];
                println!(
                    "  - {:.9} {} (cost_basis={:.9}, price={:.9}, {})",
                    item.volume,
                    trade.currency_to,
                    item.cost_basis,
                    trade.currency_to_price_usd,
                    datetime_to_str(&trade.datetime),
                );
            }
        }
    }
}
