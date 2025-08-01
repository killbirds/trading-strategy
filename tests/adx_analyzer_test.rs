use chrono::{DateTime, Utc};
use trading_chart::Candle;
use trading_chart::CandleInterval;
use trading_strategy::analyzer::adx_analyzer::ADXAnalyzer;
use trading_strategy::analyzer::base::AnalyzerOps;
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
fn test_adx_analyzer_creation() {
    let storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let periods = vec![14, 21, 28];
    let analyzer = ADXAnalyzer::new(&periods, &storage);
    assert_eq!(analyzer.items.len(), 0);
}

#[test]
fn test_adx_strong_trend() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let periods = vec![14];
    let mut analyzer = ADXAnalyzer::new(&periods, &storage);

    // 강한 상승 추세를 나타내는 캔들 데이터 생성
    for i in 0..20 {
        let price = 100.0 + i as f64;
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
    assert!(analyzer.is_strong_trend(1, 0));
}

#[test]
fn test_adx_weak_trend() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let periods = vec![14];
    let mut analyzer = ADXAnalyzer::new(&periods, &storage);

    // 약한 추세를 나타내는 캔들 데이터 생성 (옆으로 움직임)
    for i in 0..50 {
        let price = 100.0 + (i % 3 - 1) as f64 * 0.5; // 더 작은 가격 변동
        let candle = TestCandle {
            timestamp: i as i64,
            open: price,
            high: price + 0.2, // 더 작은 고가/저가 범위
            low: price - 0.2,
            close: price,
            volume: 1000.0,
        };
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    assert!(analyzer.is_weak_trend(1, 0));
}

#[test]
fn test_adx_trend_strengthening() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let periods = vec![14];
    let mut analyzer = ADXAnalyzer::new(&periods, &storage);

    // 추세가 강해지는 패턴의 캔들 데이터 생성
    // 초기에는 작은 상승폭으로 시작하여 점점 상승폭이 커지도록 함
    for i in 0..50 {
        let multiplier = 1.0 + (i as f64 * 0.2);
        let base_price = 100.0 + (i as f64 * multiplier);
        let candle = TestCandle {
            timestamp: i as i64,
            open: base_price - multiplier,
            high: base_price + multiplier * 2.0,
            low: base_price - multiplier * 2.0,
            close: base_price + multiplier,
            volume: 1000.0 + (i * 100) as f64,
        };
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    assert!(analyzer.is_trend_strengthening(14, 3));
}

#[test]
fn test_adx_trend_reversal() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let periods = vec![14];
    let mut analyzer = ADXAnalyzer::new(&periods, &storage);

    // 1단계: 강한 하락 추세 (25개 캔들)
    for i in 0..25 {
        let base_price = 100.0 - (i as f64 * 2.5);
        let candle = TestCandle {
            timestamp: i as i64,
            open: base_price + 1.5,
            high: base_price + 2.0,
            low: base_price - 2.0,
            close: base_price - 1.5,
            volume: 2500.0,
        };
        storage.add(candle);
    }

    // 2단계: 횡보 구간 (15개 캔들)
    let bottom_price = 37.5;
    for i in 0..15 {
        let oscillation = (i % 3) as f64 * 0.3;
        let base_price = bottom_price + oscillation;
        let candle = TestCandle {
            timestamp: (i + 25) as i64,
            open: base_price,
            high: base_price + 0.3,
            low: base_price - 0.3,
            close: base_price,
            volume: 1000.0,
        };
        storage.add(candle);
    }

    // 3단계: 강한 상승 추세 (20개 캔들)
    for i in 0..20 {
        let base_price = 37.5 + (i as f64 * 2.0);
        let candle = TestCandle {
            timestamp: (i + 40) as i64,
            open: base_price - 1.0,
            high: base_price + 1.5,
            low: base_price - 1.5,
            close: base_price + 1.0,
            volume: 2000.0,
        };
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    assert!(analyzer.is_trend_reversal(14, 3, 3));
}
