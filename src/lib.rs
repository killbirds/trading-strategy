pub mod analyzer;
pub mod candle_store;
pub mod indicator;
pub mod model;
pub mod strategy;

/// 설정 로더
pub mod config_loader;

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
        fn candle_interval(&self) -> &CandleInterval {
            &CandleInterval::Minute1
        }
        fn acc_trade_price(&self) -> f64 {
            self.volume * self.close
        }
        fn acc_trade_volume(&self) -> f64 {
            self.volume
        }
    }
}
