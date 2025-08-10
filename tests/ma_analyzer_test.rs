use chrono::{DateTime, Utc};
use trading_chart::Candle;
use trading_chart::CandleInterval;
use trading_strategy::analyzer::ma_analyzer::MAAnalyzer;
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

fn create_sideways_ma_candles(count: usize, base_price: f64, range: f64) -> Vec<TestCandle> {
    let mut candles = Vec::with_capacity(count);
    let now = Utc::now();

    for i in 0..count {
        // 횡보 패턴: 작은 변동폭으로 가격이 좌우로 움직임
        let oscillation = (i % 4) as f64 * range / 4.0 - range / 2.0;
        let price = base_price + oscillation;
        let timestamp = now.timestamp() + (i as i64 * 60);
        let candle = TestCandle {
            timestamp,
            open: price,
            high: price + range / 8.0,
            low: price - range / 8.0,
            close: price,
            volume: 100.0,
        };
        candles.push(candle);
    }

    candles
}

fn create_trending_ma_candles(count: usize, base_price: f64, trend: f64) -> Vec<TestCandle> {
    let mut candles = Vec::with_capacity(count);
    let now = Utc::now();

    for i in 0..count {
        // 추세 패턴: 지속적으로 한 방향으로 움직임
        let price = base_price + (i as f64 * trend);
        let timestamp = now.timestamp() + (i as i64 * 60);
        let candle = TestCandle {
            timestamp,
            open: price,
            high: price + trend.abs() / 2.0,
            low: price - trend.abs() / 2.0,
            close: price,
            volume: 100.0,
        };
        candles.push(candle);
    }

    candles
}

#[test]
fn test_ma_analyzer_is_sideways() {
    // 횡보 데이터 생성
    let sideways_candles = create_sideways_ma_candles(50, 100.0, 5.0);
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);

    // 캔들 데이터를 스토리지에 추가
    for candle in &sideways_candles {
        storage.add(candle.clone());
    }

    // MA 분석기 생성
    let ma_periods = vec![10, 20];
    let analyzer = MAAnalyzer::new(&MAType::SMA, &ma_periods, &storage);

    // 횡보 상태 테스트 (작은 임계값 - 5% 비율)
    let is_sideways_result = analyzer.is_ma_sideways(0, 10, 0, 0.05);
    println!("횡보 데이터에서 is_sideways 결과: {is_sideways_result}");

    // 횡보 데이터에서는 true가 나와야 함
    assert!(is_sideways_result);
}

#[test]
fn test_ma_analyzer_is_not_sideways() {
    // 추세 데이터 생성
    let trending_candles = create_trending_ma_candles(50, 100.0, 1.0);
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);

    // 캔들 데이터를 스토리지에 추가
    for candle in &trending_candles {
        storage.add(candle.clone());
    }

    // MA 분석기 생성
    let ma_periods = vec![10, 20];
    let analyzer = MAAnalyzer::new(&MAType::SMA, &ma_periods, &storage);

    // 추세 상태 테스트 (작은 임계값 - 5% 비율)
    let is_sideways_result = analyzer.is_ma_sideways(0, 10, 0, 0.05);
    println!("추세 데이터에서 is_sideways 결과: {is_sideways_result}");

    // 추세 데이터에서는 false가 나와야 함
    assert!(!is_sideways_result);
}

#[test]
fn test_ma_analyzer_is_sideways_with_different_thresholds() {
    // 횡보 데이터 생성
    let sideways_candles = create_sideways_ma_candles(50, 100.0, 5.0);
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);

    // 캔들 데이터를 스토리지에 추가
    for candle in &sideways_candles {
        storage.add(candle.clone());
    }

    // MA 분석기 생성
    let ma_periods = vec![10, 20];
    let analyzer = MAAnalyzer::new(&MAType::SMA, &ma_periods, &storage);

    // 다양한 임계값으로 테스트 (비율 기반)
    let thresholds = vec![0.01, 0.05, 0.1, 0.2]; // 1%, 5%, 10%, 20%

    for threshold in thresholds {
        let result = analyzer.is_ma_sideways(0, 10, 0, threshold);
        println!(
            "임계값 {} ({}%)에서 is_sideways 결과: {}",
            threshold,
            threshold * 100.0,
            result
        );

        // 임계값이 클수록 true가 나올 가능성이 높음
        if threshold >= 0.1 {
            assert!(result);
        }
    }
}

#[test]
fn test_ma_analyzer_is_sideways_insufficient_data() {
    // 데이터가 부족한 경우 테스트
    let storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let ma_periods = vec![10, 20];
    let analyzer = MAAnalyzer::new(&MAType::SMA, &ma_periods, &storage);

    // 데이터가 부족한 경우 false 반환
    let result = analyzer.is_ma_sideways(0, 10, 0, 0.05);
    assert!(!result);
}
