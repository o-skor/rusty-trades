use std::collections::HashMap;

use chrono::DateTime;
use chrono_tz::Tz;

use crate::{
    model::{
        sell_trade::SellTrade,
        trade::Trade,
        usd_trade::{UsdTrade, BITCOINTAX_INPUT_COLUMNS},
        Currency,
    },
    utils::time_utils::{datetime_to_str, datetime_within_limits},
};

struct HoldingsItem {
    volume: f64,
    cost_basis: f64,
    trade_idx: usize,
}

#[derive(Default)]
pub struct Wallet {
    trades: Vec<Trade>,
    sell_trades: Vec<SellTrade>,
    usd_trades: Vec<UsdTrade>,
    holdings: HashMap<Currency, Vec<HoldingsItem>>,
}

impl Wallet {
    pub fn add_trade(&mut self, trade: Trade) {
        const EPS: f64 = 1e-5;

        if trade.currency_from != "USD" {
            self.usd_trades.push(UsdTrade::new(&trade, false));

            let holdings_bucket = self.holdings.get_mut(&trade.currency_from).unwrap();
            let mut volume_to_sell = trade.volume_from;

            while volume_to_sell > EPS {
                let buy_trade_idx = holdings_bucket[0].trade_idx;
                let buy_trade = &self.trades[buy_trade_idx];
                assert_eq!(trade.currency_from, buy_trade.currency_to);

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
                    cost_basis_sold = holdings_bucket[0].cost_basis;
                    holdings_bucket.remove(0);
                }

                self.sell_trades.push(SellTrade::new(
                    volume_sold,
                    cost_basis_sold,
                    buy_trade,
                    &trade,
                    buy_trade_idx,
                    self.trades.len(),
                ));

                volume_to_sell -= volume_sold;
            }
        }

        if trade.currency_to != "USD" {
            self.usd_trades.push(UsdTrade::new(&trade, true));

            let holdings_bucket = self
                .holdings
                .entry(trade.currency_to.to_owned())
                .or_insert(Vec::new());
            let cost_basis = (trade.volume_to * trade.currency_to_price_usd) + trade.fees_usd();
            holdings_bucket.push(HoldingsItem {
                volume: trade.volume_to,
                cost_basis,
                trade_idx: self.trades.len(),
            });
        }

        self.trades.push(trade);
    }

    pub fn print_trades(&self, print_notes: bool, dt_from: &DateTime<Tz>, dt_to: &DateTime<Tz>) {
        for trade in &self.trades {
            if !datetime_within_limits(&trade.datetime, dt_from, dt_to) {
                continue;
            }
            println!("{}", trade);
            if print_notes {
                for note in &trade.notes {
                    println!("--- {}", note);
                }
            }
        }
    }

    pub fn print_usd_trades(&self, dt_from: &DateTime<Tz>, dt_to: &DateTime<Tz>) {
        println!("{}", BITCOINTAX_INPUT_COLUMNS);
        for usd_trade in &self.usd_trades {
            if !datetime_within_limits(&usd_trade.datetime, dt_from, dt_to) {
                continue;
            }
            println!("{}", usd_trade);
        }
    }

    pub fn print_sell_trades(&self, full_info: bool, dt_from: &DateTime<Tz>, dt_to: &DateTime<Tz>) {
        for (i, sell_trade) in (&self.sell_trades).iter().enumerate() {
            if !datetime_within_limits(&sell_trade.sell_datetime, dt_from, dt_to) {
                continue;
            }
            println!("{}", sell_trade);
            if full_info {
                let st = &self.trades[sell_trade.sell_trade_idx];
                println!("- SELL: {}", st);
                for note in &st.notes {
                    println!("--- {}", note);
                }
                let bt = &self.trades[sell_trade.buy_trade_idx];
                println!("- BUY: {}", bt);
                for note in &bt.notes {
                    println!("--- {}", note);
                }
                if i + 1 < self.sell_trades.len() {
                    println!();
                }
            }
        }
    }

    pub fn print_proceeds(&self, dt_from: &DateTime<Tz>, dt_to: &DateTime<Tz>) {
        #[derive(Default)]
        struct ProceedsInfo {
            volume: f64,
            proceeds: f64,
            cost_basis: f64,
            gain: f64,
        }
        let mut proceeds_lt = HashMap::<Currency, ProceedsInfo>::new();
        let mut proceeds_st = HashMap::<Currency, ProceedsInfo>::new();

        for st in &self.sell_trades {
            if !datetime_within_limits(&st.sell_datetime, dt_from, dt_to) {
                continue;
            }
            let infos = if st.is_long_term() {
                &mut proceeds_lt
            } else {
                &mut proceeds_st
            };
            let entry = infos
                .entry(st.currency.clone())
                .or_insert(ProceedsInfo::default());
            entry.volume += st.volume;
            entry.proceeds += st.proceeds;
            entry.cost_basis += st.cost_basis;
            entry.gain += st.gain();
        }

        for &is_lt in &[true, false] {
            let (proceeds, label) = if is_lt {
                (&proceeds_lt, "LONG")
            } else {
                (&proceeds_st, "SHORT")
            };
            let mut totals = ProceedsInfo::default();
            let mut currencies = vec![];
            for (currency, info) in proceeds {
                currencies.push(currency);
                totals.volume += info.volume;
                totals.proceeds += info.proceeds;
                totals.cost_basis += info.cost_basis;
                totals.gain += info.gain;
            }
            currencies.sort();

            if !is_lt {
                println!();
            }
            println!("Target period {}-TERM gains:", label);
            for currency in currencies {
                let info = proceeds.get(currency).unwrap();
                println!(
                    "- {}: volume={:.9} proceeds={:.9} cost_basis={:.9} gains={:.9}",
                    currency, info.volume, info.proceeds, info.cost_basis, info.gain
                );
            }
            println!(
                "total_volume={:.9} total_proceeds={:.9} total_cost_basis={:.9} total_gains={:.9}",
                totals.volume, totals.proceeds, totals.cost_basis, totals.gain
            );
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
                let trade = &self.trades[item.trade_idx];
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
