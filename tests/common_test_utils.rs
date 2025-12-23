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
            "TestCandle(t={}, o={}, h={}, l={}, c={}, v={})",
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
        DateTime::from_timestamp(self.timestamp, 0).unwrap_or_default()
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

impl TestCandle {
    pub fn new(timestamp: i64, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Self {
        TestCandle {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        }
    }
}

pub fn create_uptrend_candles(count: usize, base_price: f64, step: f64) -> Vec<TestCandle> {
    let mut candles = Vec::with_capacity(count);
    for i in 0..count {
        let price = base_price + (i as f64 * step);
        candles.push(TestCandle {
            timestamp: i as i64,
            open: price - step / 2.0,
            high: price + step,
            low: price - step,
            close: price + step / 2.0,
            volume: 1000.0,
        });
    }
    candles
}

pub fn create_downtrend_candles(count: usize, base_price: f64, step: f64) -> Vec<TestCandle> {
    let mut candles = Vec::with_capacity(count);
    for i in 0..count {
        let price = base_price - (i as f64 * step);
        candles.push(TestCandle {
            timestamp: i as i64,
            open: price + step / 2.0,
            high: price + step,
            low: price - step,
            close: price - step / 2.0,
            volume: 1000.0,
        });
    }
    candles
}

pub fn create_sideways_candles(count: usize, base_price: f64, range: f64) -> Vec<TestCandle> {
    let mut candles = Vec::with_capacity(count);
    for i in 0..count {
        let oscillation = (i % 4) as f64 * range / 4.0 - range / 2.0;
        let price = base_price + oscillation;
        candles.push(TestCandle {
            timestamp: i as i64,
            open: price,
            high: price + range / 8.0,
            low: price - range / 8.0,
            close: price,
            volume: 1000.0,
        });
    }
    candles
}
