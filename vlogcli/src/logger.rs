use crate::proj_config::CliLogger;
use crate::{UNKONW, error::CliError};
use flexi_logger::{DeferredNow, FileSpec, LogSpecification, Logger, Record};

pub fn init(config: CliLogger) -> Result<(), CliError> {
    let level = match config.level.as_str() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    };

    let log_specification = LogSpecification::builder().default(level).build();
    Logger::with(log_specification)
        .format_for_files(file_log_format)
        .format_for_stdout(console_log_format)
        .log_to_file(FileSpec::default().directory(config.path))
        .duplicate_to_stdout(flexi_logger::Duplicate::All)
        .write_mode(flexi_logger::WriteMode::Direct)
        .start()?;
    Ok(())
}

fn file_log_format(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> std::io::Result<()> {
    write!(
        w,
        "[{}][{}][{}][{}:{}] - {}",
        now.now().format("%Y-%m-%d %H:%M:%S%.3f"),
        record.level(),
        record.module_path().unwrap_or(UNKONW),
        record.file().unwrap_or(UNKONW),
        record.line().unwrap_or(0),
        &record.args()
    )
}

fn console_log_format(
    w: &mut dyn std::io::Write,
    _now: &mut DeferredNow,
    record: &Record,
) -> std::io::Result<()> {
    write!(w, "{}", &record.args())
}
