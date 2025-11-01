#[cfg(not(any(tarpaulin, coverage)))]
use std::io::Write;
#[cfg(not(any(tarpaulin, coverage)))]
const BLUE: &str = "[94m";
#[cfg(not(any(tarpaulin, coverage)))]
const RESET: &str = "[0m";

#[cfg(not(any(tarpaulin, coverage)))]
fn main() {
    env_logger::Builder::new()
        .format(|buf, record| {
            if record.level() == log::Level::Error {
                writeln!(buf, "{}Error:{} {}", BLUE, RESET, record.args())
            } else {
                writeln!(buf, "{}", record.args())
            }
        })
        .filter(None, log::LevelFilter::Info)
        .init();

    if let Err(e) = mdcode::run() {
        eprintln!("{}Error:{} {}", BLUE, RESET, e);
        std::process::exit(1);
    }
}

#[cfg(any(tarpaulin, coverage))]
fn main() {}
