use chrono::{DateTime, Utc};
use trading_chart::Candle;
use trading_chart::CandleInterval;
use trading_strategy::analyzer::base::AnalyzerOps;
use trading_strategy::analyzer::bband_analyzer::BBandAnalyzer;
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
fn test_bband_analyzer_creation() {
    let storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let analyzer = BBandAnalyzer::new(20, 2.0, &storage);
    assert_eq!(analyzer.items.len(), 0);
}

#[test]
fn test_bband_below_lower_band() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = BBandAnalyzer::new(20, 2.0, &storage);

    // 하한선 아래로 가격이 내려가는 패턴 생성
    for i in 0..40 {
        let price = if i < 39 {
            100.0
        } else {
            50.0 // 마지막 캔들만 극단적으로 하락
        };
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
    println!(
        "Lower band: {}, Close price: {}",
        analyzer.items.first().unwrap().bband.lower(),
        analyzer.items.first().unwrap().candle.close_price()
    );
    assert!(analyzer.is_below_lower_band(1, 0));
}

#[test]
fn test_bband_above_upper_band() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = BBandAnalyzer::new(20, 2.0, &storage);

    // 상한선 위로 가격이 올라가는 패턴 생성
    for i in 0..40 {
        let price = if i < 39 {
            100.0
        } else {
            150.0 // 마지막 캔들만 극단적으로 상승
        };
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
    println!(
        "Upper band: {}, Close price: {}",
        analyzer.items.first().unwrap().bband.upper(),
        analyzer.items.first().unwrap().candle.close_price()
    );
    assert!(analyzer.is_above_upper_band(1, 0));
}

#[test]
fn test_bband_width_sufficient() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = BBandAnalyzer::new(20, 2.0, &storage);

    // 밴드 폭이 넓어지는 패턴 생성
    for i in 0..25 {
        let price = if i < 20 {
            100.0
        } else {
            100.0 + (i - 20) as f64 * 2.0 // 가격 변동성 증가
        };
        let candle = TestCandle {
            timestamp: i as i64,
            open: price,
            high: price + 2.0,
            low: price - 2.0,
            close: price,
            volume: 1000.0,
        };
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    assert!(analyzer.is_band_width_sufficient(0));
}
