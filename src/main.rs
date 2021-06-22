mod model;
mod utils;

use model::trade::load_trades;

fn main() {
    let trades = load_trades();

    for t in trades {
        println!("{}", t);
        for note in t.notes {
            println!("--- {}", note);
        }
    }
}
