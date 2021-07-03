use chrono::{DateTime, Datelike, TimeZone};
use chrono_tz::Tz;

use rand::Rng;

pub const APP_TZ: &Tz = &chrono_tz::UTC;
// pub const APP_TZ: &Tz = &chrono_tz::US::Pacific;
// pub const APP_TZ: &Tz = &chrono_tz::US::Central;

pub const APP_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S%:z";

pub fn generate_random_datetime<R: Rng>(
    dt_from: &DateTime<Tz>,
    dt_to: &DateTime<Tz>,
    rng: &mut R,
) -> DateTime<Tz> {
    assert!(dt_from < dt_to);
    let from_millis = dt_from.timestamp_millis();
    let to_millis = dt_to.timestamp_millis();
    let max_rng = to_millis - from_millis;
    let millis = from_millis + rng.gen_range(0..max_rng);
    APP_TZ.timestamp_millis(millis)
}

pub fn datetime_from_str(s: &str) -> Result<DateTime<Tz>, chrono::ParseError> {
    DateTime::parse_from_str(s, APP_TIME_FORMAT)
        .map(|dt_fixed_offset| dt_fixed_offset.with_timezone(APP_TZ))
}

pub fn datetime_to_str(dt: &DateTime<Tz>) -> String {
    dt.format(APP_TIME_FORMAT).to_string()
}

pub fn is_datetime_within_limits(
    dt: &DateTime<Tz>,
    dt_from: &DateTime<Tz>,
    dt_to: &DateTime<Tz>,
) -> bool {
    assert!(dt_from < dt_to);
    dt >= dt_from && dt < dt_to
}

pub fn start_of_the_day(dt: &DateTime<Tz>) -> DateTime<Tz> {
    APP_TZ
        .ymd(dt.year(), dt.month(), dt.day())
        .and_hms_milli(0, 0, 0, 0)
}

pub fn end_of_the_day(dt: &DateTime<Tz>) -> DateTime<Tz> {
    APP_TZ
        .ymd(dt.year(), dt.month(), dt.day())
        .and_hms_milli(23, 59, 59, 999)
}
