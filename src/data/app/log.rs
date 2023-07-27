use colored::{Colorize, ColoredString};
use log::{Record, Level, Metadata, Log, SetLoggerError, LevelFilter};

enum Color {
    Blue,
    Yellow,
    Red,
    White
}
impl Color {
    pub fn from_level(level: Level) -> Color {
        match level {
            Level::Debug => Color::Blue,
            Level::Warn => Color::Yellow,
            Level::Error => Color::Red,
            _ => Color::White,
        }
    }

    pub fn apply(&self, string: &str) -> ColoredString {
        match self {
            Color::Blue => string.bright_blue(),
            Color::Yellow => string.yellow(),
            Color::Red => string.bright_red(),
            Color::White => string.white()
        }
    }
}

pub struct Logger;

impl Logger {
    pub fn init() -> Result<(), SetLoggerError>  {
        #[cfg(windows)] {
            let _varname = colored::control::set_virtual_terminal(true).unwrap_or(());
        }

        log::set_logger(&crate::LOGGER)
            .map(|()| log::set_max_level(LevelFilter::Debug))
    }
}
impl Log for Logger {
    #[cfg(debug_assertions)]
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
        && metadata.target().starts_with("wifu")
    }
    
    #[cfg(not(debug_assertions))]
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
        && metadata.target().starts_with("wifu")
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let color = Color::from_level(record.level());

            let date = chrono::Local::now();

            let fmt_date = date.format("%Y-%m-%d at %H:%M:%S")
                .to_string();

            let path = record.file()
                .unwrap()
                .bright_blue();

            let line = format!("{}", record.line().unwrap())
                .bright_blue();

            println!("{} [{}] {} ({}:{})", 
                fmt_date.bright_green(), 
                color.apply(record.level().as_str()), 
                color.apply(&record.args().to_string()),
                path,
                line
            );
        }
    }

    fn flush(&self) {}
}