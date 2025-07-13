pub mod analyzer;
pub mod candle_store;
pub mod filter;
pub mod indicator;
pub mod model;
pub mod strategy;

/// 설정 로드 오류
#[derive(Debug)]
pub enum ConfigError {
    /// 파일 오류
    FileError(String),
    /// 파싱 오류
    ParseError(String),
    /// 유효성 검사 오류
    ValidationError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileError(msg) => write!(f, "설정 파일 오류: {msg}"),
            ConfigError::ParseError(msg) => write!(f, "설정 파싱 오류: {msg}"),
            ConfigError::ValidationError(msg) => write!(f, "설정 유효성 검사 오류: {msg}"),
        }
    }
}

/// String으로 ConfigError 변환
impl From<ConfigError> for String {
    fn from(err: ConfigError) -> Self {
        match err {
            ConfigError::FileError(msg) => format!("설정 파일 오류: {msg}"),
            ConfigError::ParseError(msg) => format!("설정 파싱 오류: {msg}"),
            ConfigError::ValidationError(msg) => format!("설정 유효성 검사 오류: {msg}"),
        }
    }
}

/// 설정 로드 결과
pub type ConfigResult<T> = Result<T, ConfigError>;

/// 설정 로더 트레이트
pub trait ConfigValidation {
    /// 설정 유효성 검사
    fn validate(&self) -> ConfigResult<()>;
}

#[cfg(test)]
pub mod tests {
    use chrono::{DateTime, Utc};
    use trading_chart::Candle;
    use trading_chart::CandleInterval;

    #[derive(Debug, Clone, Default, PartialEq)]
    pub struct TestCandle {
        pub timestamp: i64,
        pub open: f64,
        pub high: f64,
        pub low: f64,
        pub close: f64,
        pub volume: f64,
    }

    impl std::fmt::Display for TestCandle {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "TestCandle(t={}, o={:.2}, h={:.2}, l={:.2}, c={:.2}, v={:.2})",
                self.timestamp, self.open, self.high, self.low, self.close, self.volume
            )
        }
    }

    impl Candle for TestCandle {
        fn open_price(&self) -> f64 {
            self.open
        }
        fn high_price(&self) -> f64 {
            self.high
        }
        fn low_price(&self) -> f64 {
            self.low
        }
        fn close_price(&self) -> f64 {
            self.close
        }
        fn market(&self) -> &str {
            "test"
        }
        fn datetime(&self) -> DateTime<Utc> {
            DateTime::from_timestamp(self.timestamp, 0).unwrap()
        }
        fn interval(&self) -> &CandleInterval {
            &CandleInterval::Minute1
        }
        fn volume(&self) -> f64 {
            self.volume
        }
        fn quote_volume(&self) -> f64 {
            self.volume
        }
        fn trade_count(&self) -> Option<u64> {
            None
        }
    }
}
