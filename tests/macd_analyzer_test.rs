use chrono::{DateTime, Utc};
use trading_chart::Candle;
use trading_chart::CandleInterval;
use trading_strategy::analyzer::base::AnalyzerOps;
use trading_strategy::analyzer::macd_analyzer::MACDAnalyzer;
use trading_strategy::candle_store::CandleStore;

#[derive(Debug, Clone, Default, PartialEq)]
struct TestCandle {
    timestamp: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
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

#[test]
fn test_macd_analyzer_creation() {
    let storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let analyzer = MACDAnalyzer::new(12, 26, 9, &storage);
    assert_eq!(analyzer.items.len(), 0);
}

#[test]
fn test_macd_histogram_above_threshold() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = MACDAnalyzer::new(12, 26, 9, &storage);

    // 상승 추세를 나타내는 캔들 데이터 생성
    for i in 0..30 {
        let price = 100.0 + i as f64 * 2.0;
        let candle = TestCandle {
            timestamp: i as i64,
            open: price,
            high: price + 1.0,
            low: price - 1.0,
            close: price,
            volume: 1000.0,
        };
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    assert!(analyzer.is_histogram_above_threshold(0.0, 1));
}

#[test]
fn test_macd_crossed_above_signal() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = MACDAnalyzer::new(12, 26, 9, &storage);

    // MACD가 시그널 라인을 상향 돌파하는 패턴 생성
    // 먼저 하락 추세
    for i in 0..30 {
        let price = 100.0 - i as f64 * 0.5;
        let candle = TestCandle {
            timestamp: i as i64,
            open: price,
            high: price + 1.0,
            low: price - 1.0,
            close: price,
            volume: 1000.0,
        };
        storage.add(candle);
    }

    // 그 다음 상승 추세
    for i in 0..10 {
        let price = 85.0 + i as f64 * 2.0;
        let candle = TestCandle {
            timestamp: (i + 30) as i64,
            open: price,
            high: price + 1.0,
            low: price - 1.0,
            close: price,
            volume: 1000.0,
        };
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    assert!(analyzer.is_macd_crossed_above_signal(1, 1));
}

#[test]
fn test_macd_crossed_below_signal() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = MACDAnalyzer::new(12, 26, 9, &storage);

    // MACD가 시그널 라인을 하향 돌파하는 패턴 생성
    // 먼저 상승 추세
    for i in 0..30 {
        let price = 100.0 + i as f64 * 0.5;
        let candle = TestCandle {
            timestamp: i as i64,
            open: price,
            high: price + 1.0,
            low: price - 1.0,
            close: price,
            volume: 1000.0,
        };
        storage.add(candle);
    }

    // 그 다음 하락 추세
    for i in 0..10 {
        let price = 115.0 - i as f64 * 2.0;
        let candle = TestCandle {
            timestamp: (i + 30) as i64,
            open: price,
            high: price + 1.0,
            low: price - 1.0,
            close: price,
            volume: 1000.0,
        };
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    assert!(analyzer.is_macd_crossed_below_signal(1, 1));
}
