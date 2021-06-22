pub mod time_utils;

use std::io::BufRead;

pub fn read_all_lines<R: BufRead>(reader: R) -> Result<Vec<String>, std::io::Error> {
    // Transforms an iterator of Result<T, E> into a Result<Vec<T>, E>.
    reader.lines().collect()
}
