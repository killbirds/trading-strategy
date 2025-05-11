use chrono::{DateTime, Utc};
use trading_chart::Candle;
use trading_chart::CandleInterval;
use trading_strategy::analyzer::base::AnalyzerOps;
use trading_strategy::analyzer::rsi_analyzer::RSIAnalyzer;
use trading_strategy::candle_store::CandleStore;
use trading_strategy::indicator::ma::MAType;

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
fn test_rsi_analyzer_creation() {
    let storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let ma_periods = vec![20, 50, 200];
    let analyzer = RSIAnalyzer::new(14, &MAType::SMA, &ma_periods, &storage);
    assert_eq!(analyzer.items.len(), 0);
}

#[test]
fn test_rsi_less_than() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let ma_periods = vec![20];
    let mut analyzer = RSIAnalyzer::new(14, &MAType::SMA, &ma_periods, &storage);

    // 과매도 상태를 나타내는 캔들 데이터 생성
    for i in 0..20 {
        let price = 100.0 - i as f64 * 2.0;
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
    assert!(analyzer.is_rsi_less_than(30.0, 1));
}

#[test]
fn test_rsi_greater_than() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let ma_periods = vec![20];
    let mut analyzer = RSIAnalyzer::new(14, &MAType::SMA, &ma_periods, &storage);

    // 과매수 상태를 나타내는 캔들 데이터 생성
    for i in 0..20 {
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
    assert!(analyzer.is_rsi_greater_than(70.0, 1));
}

#[test]
fn test_ma_regular_arrangement() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let ma_periods = vec![5, 10, 20];
    let mut analyzer = RSIAnalyzer::new(14, &MAType::SMA, &ma_periods, &storage);

    // 상승 추세를 나타내는 캔들 데이터 생성
    for i in 0..30 {
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
    assert!(analyzer.is_ma_regular_arrangement(1));
}

#[test]
fn test_ma_crossed() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let ma_periods = vec![5, 20];
    let mut analyzer = RSIAnalyzer::new(14, &MAType::SMA, &ma_periods, &storage);

    // 골든 크로스 패턴 생성
    // 먼저 하락 추세
    for i in 0..30 {
        let price = 100.0 - i as f64 * 1.5;
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

    // 그 다음 급격한 상승 추세
    for i in 0..15 {
        let price = 55.0 + i as f64 * 3.0;
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

    println!("테스트 캔들 데이터 생성 완료: {} 개", storage.len());

    analyzer.init_from_storage(&storage);

    // MA 값 출력 (디버깅)
    for i in 0..analyzer.items.len().min(5) {
        let short_ma = analyzer.items[i].mas.get_by_key_index(0).get();
        let long_ma = analyzer.items[i].mas.get_by_key_index(1).get();
        println!(
            "캔들 {}: 단기MA={:.2}, 장기MA={:.2}, 가격={:.2}",
            i,
            short_ma,
            long_ma,
            analyzer.items[i].candle.close_price()
        );
    }

    assert!(analyzer.is_ma_crossed(0, 1));
}
