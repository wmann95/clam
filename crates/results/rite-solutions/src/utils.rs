//! Some utilities

use ftlog::{
    appender::{FileAppender, Period},
    LevelFilter, LoggerGuard,
};

#[allow(dead_code)]
pub fn configure_logger<P: AsRef<std::path::Path>>(log_path: P) -> Result<LoggerGuard, String> {
    let writer = FileAppender::builder().path(&log_path).rotate(Period::Day).build();

    let log_path = log_path.as_ref().to_path_buf();
    let err_path = log_path.with_extension("err.log");

    let guard = ftlog::Builder::new()
        // global max log level
        .max_log_level(LevelFilter::Info)
        // define root appender, pass None would write to stderr
        .root(writer)
        // write `Warn` and `Error` logs in ftlog::appender to `err_path` instead of `log_path`
        .filter("ftlog::appender", "ftlog-appender", LevelFilter::Warn)
        .appender("ftlog-appender", FileAppender::new(err_path))
        .try_init()
        .map_err(|e| e.to_string())?;

    Ok(guard)
}

pub fn normalize_distances(distances: &Vec<f32>) -> Vec<f32>{
    let mut min = f32::INFINITY;
    let mut max = f32::NEG_INFINITY;
    
    distances.iter().for_each(|&f| {
        if f < min { min = f }
        if f > max { max = f }
    });
    
    let diff = (max - min).abs();
    
    distances.iter().map(|f| {
        (f - min) / diff
    }).collect::<Vec<_>>()
}

pub fn normalize_distribution(distribution: &Vec<f32>) -> Vec<f32>{
    let sum: f32 = distribution.iter().sum();
    
    distribution.iter().map(|f| f / sum).collect()
}