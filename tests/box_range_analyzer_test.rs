use chrono::{DateTime, Utc};
use trading_chart::Candle;
use trading_chart::CandleInterval;
use trading_strategy::analyzer::base::AnalyzerOps;
use trading_strategy::analyzer::box_range_analyzer::BoxRangeAnalyzer;
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

fn candle(timestamp: i64, high: f64, low: f64, close: f64) -> TestCandle {
    TestCandle {
        timestamp,
        open: close,
        high,
        low,
        close,
        volume: 1000.0,
    }
}

fn storage_from(candles: Vec<TestCandle>) -> CandleStore<TestCandle> {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    for item in candles {
        storage.add(item);
    }
    storage
}

fn box_candles() -> Vec<TestCandle> {
    vec![
        candle(1, 101.0, 99.0, 100.0),
        candle(2, 102.0, 98.5, 101.0),
        candle(3, 101.5, 99.5, 100.5),
        candle(4, 102.0, 98.0, 99.5),
        candle(5, 101.0, 99.0, 100.0),
    ]
}

#[test]
fn test_box_range_analyzer_creation() {
    let storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let analyzer = BoxRangeAnalyzer::new(20, 0.05, &storage);

    assert_eq!(analyzer.items.len(), 0);
    assert_eq!(analyzer.get_box_range(), (0.0, 0.0, 0.0));
}

#[test]
fn test_box_range_analyzer_detects_box_range() {
    let storage = storage_from(box_candles());
    let analyzer = BoxRangeAnalyzer::new(5, 0.05, &storage);

    assert_eq!(analyzer.items.len(), 5);
    assert!(analyzer.is_box_range(1, 0));
    assert!(analyzer.is_close_inside_box(1, 0));
    assert_eq!(analyzer.get_box_range(), (98.0, 100.0, 102.0));
    assert_eq!(analyzer.get_box_width_ratio(), 0.04);
}

#[test]
fn test_box_range_analyzer_rejects_wide_range() {
    let mut candles = box_candles();
    candles.push(candle(6, 130.0, 80.0, 110.0));
    let storage = storage_from(candles);
    let analyzer = BoxRangeAnalyzer::new(5, 0.05, &storage);

    assert!(!analyzer.is_box_range(1, 0));
    assert!(analyzer.get_box_width_ratio() > 0.05);
}

#[test]
fn test_box_range_analyzer_detects_breakout_above() {
    let mut analyzer = BoxRangeAnalyzer::new(
        5,
        0.05,
        &CandleStore::<TestCandle>::new(Vec::new(), 1000, false),
    );

    for item in box_candles() {
        analyzer.next(item);
    }
    analyzer.next(candle(6, 103.0, 102.0, 103.0));

    assert!(analyzer.is_close_above_box(1, 0));
    assert!(analyzer.is_high_break_through_upper_box(1, 0));
    assert!(analyzer.is_box_breakout_above(1));
}

#[test]
fn test_box_range_analyzer_detects_breakout_below() {
    let mut analyzer = BoxRangeAnalyzer::new(
        5,
        0.05,
        &CandleStore::<TestCandle>::new(Vec::new(), 1000, false),
    );

    for item in box_candles() {
        analyzer.next(item);
    }
    analyzer.next(candle(6, 98.0, 97.0, 97.0));

    assert!(analyzer.is_close_below_box(1, 0));
    assert!(analyzer.is_low_break_through_lower_box(1, 0));
    assert!(analyzer.is_box_breakout_below(1));
}

#[test]
fn test_box_range_analyzer_breakout_helpers_require_history() {
    let mut analyzer = BoxRangeAnalyzer::new(
        5,
        0.05,
        &CandleStore::<TestCandle>::new(Vec::new(), 1000, false),
    );

    analyzer.next(candle(1, 103.0, 102.0, 103.0));

    assert!(!analyzer.is_close_above_box(1, 0));
    assert!(!analyzer.is_close_below_box(1, 0));
    assert!(!analyzer.is_high_break_through_upper_box(1, 0));
    assert!(!analyzer.is_low_break_through_lower_box(1, 0));
}

#[test]
fn test_box_range_analyzer_breakout_helpers_require_previous_box() {
    let mut analyzer = BoxRangeAnalyzer::new(
        5,
        0.05,
        &CandleStore::<TestCandle>::new(Vec::new(), 1000, false),
    );

    analyzer.next(candle(1, 130.0, 80.0, 100.0));
    analyzer.next(candle(2, 101.0, 99.0, 100.0));
    analyzer.next(candle(3, 101.0, 99.0, 100.0));
    analyzer.next(candle(4, 101.0, 99.0, 100.0));
    analyzer.next(candle(5, 101.0, 99.0, 100.0));
    analyzer.next(candle(6, 103.0, 102.0, 103.0));

    assert!(!analyzer.is_close_above_box(1, 0));
    assert!(!analyzer.is_high_break_through_upper_box(1, 0));
}

#[test]
fn test_box_range_analyzer_detects_box_start() {
    let mut analyzer = BoxRangeAnalyzer::new(
        5,
        0.05,
        &CandleStore::<TestCandle>::new(Vec::new(), 1000, false),
    );

    analyzer.next(candle(1, 130.0, 80.0, 100.0));
    analyzer.next(candle(2, 101.0, 99.0, 100.0));
    analyzer.next(candle(3, 101.0, 99.0, 100.0));
    analyzer.next(candle(4, 101.0, 99.0, 100.0));
    analyzer.next(candle(5, 101.0, 99.0, 100.0));
    analyzer.next(candle(6, 101.0, 99.0, 100.0));

    assert!(analyzer.is_box_range_start(1, 1, 0));
}
