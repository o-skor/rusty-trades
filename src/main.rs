mod app;
mod model;
mod utils;

use std::env;

use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use dotenv::dotenv;

use crate::{
    app::{run, Config},
    utils::time_utils::datetime_from_str,
};

const OPT_INPUT_FILE: &str = "file";
const OPT_TIME_FROM: &str = "time-from";
const OPT_TIME_TO: &str = "time-to";

const ENV_INPUT_FILE: &str = "FILE";
const ENV_TIME_FROM: &str = "TIME_FROM";
const ENV_TIME_TO: &str = "TIME_TO";

const DEFAULT_INPUT_FILE: &str = "input.txt";

fn main() {
    dotenv().ok();

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name(OPT_INPUT_FILE)
                .short("f")
                .long(OPT_INPUT_FILE)
                .value_name(ENV_INPUT_FILE)
                .help("Name of the input file containing trades")
                .takes_value(true),
        )
        .arg(
            Arg::with_name(OPT_TIME_FROM)
                .short("l")
                .long(OPT_TIME_FROM)
                .value_name(ENV_TIME_FROM)
                .help("Left time boundary for trades analysis (e.g.: '2017-01-01 00:00:00-07:30')")
                .takes_value(true),
        )
        .arg(
            Arg::with_name(OPT_TIME_TO)
                .short("r")
                .long(OPT_TIME_TO)
                .value_name(ENV_TIME_TO)
                .help("Right time boundary for trades analysis (e.g.: '2017-12-31 23:59:59+00:00')")
                .takes_value(true),
        )
        .get_matches();

    let input_file = matches
        .value_of(OPT_INPUT_FILE)
        .map(|s| s.to_owned())
        .or(env::var(ENV_INPUT_FILE).ok())
        .or_else(|| Some(DEFAULT_INPUT_FILE.to_owned()))
        .unwrap();

    let dt_from = matches
        .value_of(OPT_TIME_FROM)
        .map(|s| s.to_owned())
        .or(env::var(ENV_TIME_FROM).ok())
        .map(|s| datetime_from_str(&s).unwrap());

    let dt_to = matches
        .value_of(OPT_TIME_TO)
        .map(|s| s.to_owned())
        .or(env::var(ENV_TIME_TO).ok())
        .map(|s| datetime_from_str(&s).unwrap());

    let config = Config {
        input_file,
        dt_from,
        dt_to,
    };

    run(config);
}
