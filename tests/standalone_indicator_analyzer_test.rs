use chrono::{DateTime, Utc};
use trading_chart::{Candle, CandleInterval};
use trading_strategy::analyzer::base::AnalyzerOps;
use trading_strategy::analyzer::{
    AroonAnalyzer, AroonAnalyzerParams, ChaikinAnalyzer, ChaikinAnalyzerParams, ChoppinessAnalyzer,
    ChoppinessAnalyzerParams, DonchianAnalyzer, DonchianAnalyzerParams, KAMAAnalyzer,
    KAMAAnalyzerParams, KeltnerAnalyzer, KeltnerAnalyzerParams, MFIAnalyzer, MFIAnalyzerParams,
    OBVAnalyzer, PPOAnalyzer, PPOAnalyzerParams, ParabolicSARAnalyzer, ParabolicSARAnalyzerParams,
};
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

fn candle(timestamp: i64, high: f64, low: f64, close: f64, volume: f64) -> TestCandle {
    TestCandle {
        timestamp,
        open: close,
        high,
        low,
        close,
        volume,
    }
}

fn storage_from(candles: Vec<TestCandle>) -> CandleStore<TestCandle> {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    for item in candles {
        storage.add(item);
    }
    storage
}

fn sample_storage() -> CandleStore<TestCandle> {
    storage_from(vec![
        candle(1, 10.0, 8.0, 9.0, 100.0),
        candle(2, 11.0, 9.0, 10.0, 110.0),
        candle(3, 12.0, 10.0, 11.0, 120.0),
        candle(4, 13.0, 11.0, 12.0, 130.0),
    ])
}

#[test]
fn channel_analyzers_store_current_candle_and_outputs() {
    let storage = sample_storage();

    let donchian = DonchianAnalyzer::new(&storage, DonchianAnalyzerParams { period: 3 });
    let current = donchian.current().expect("donchian data");
    assert_eq!(donchian.items.len(), 4);
    assert_eq!(current.candle.close_price(), 12.0);
    assert_eq!(current.donchian.upper(), 13.0);
    assert_eq!(current.donchian.lower(), 9.0);

    let keltner = KeltnerAnalyzer::new(
        &storage,
        KeltnerAnalyzerParams {
            period: 3,
            multiplier: 2.0,
        },
    );
    let current = keltner.current().expect("keltner data");
    assert!(current.keltner.upper() >= current.keltner.middle());
}

#[test]
fn volume_and_oscillator_analyzers_store_single_outputs() {
    let storage = sample_storage();

    let obv = OBVAnalyzer::new(&storage);
    assert!(obv.current().unwrap().obv.value() > 0.0);

    let mfi = MFIAnalyzer::new(&storage, MFIAnalyzerParams { period: 3 });
    assert!(mfi.current().unwrap().mfi.value() >= 0.0);

    let aroon = AroonAnalyzer::new(&storage, AroonAnalyzerParams { period: 3 });
    assert!(aroon.current().unwrap().aroon.up() >= aroon.current().unwrap().aroon.down());

    let choppiness = ChoppinessAnalyzer::new(&storage, ChoppinessAnalyzerParams { period: 3 });
    assert!(choppiness.current().unwrap().choppiness.value().is_finite());
}

#[test]
fn adaptive_and_trend_analyzers_store_single_outputs() {
    let storage = sample_storage();

    let kama = KAMAAnalyzer::new(
        &storage,
        KAMAAnalyzerParams {
            period: 3,
            fast_period: 2,
            slow_period: 4,
        },
    );
    assert!(kama.current().unwrap().kama.value().is_finite());

    let chaikin = ChaikinAnalyzer::new(
        &storage,
        ChaikinAnalyzerParams {
            cmf_period: 3,
            fast_period: 2,
            slow_period: 4,
        },
    );
    assert!(chaikin.current().unwrap().chaikin.cmf().is_finite());

    let ppo = PPOAnalyzer::new(
        &storage,
        PPOAnalyzerParams {
            fast_period: 2,
            slow_period: 3,
            signal_period: 2,
        },
    );
    assert!(ppo.current().unwrap().ppo.histogram().is_finite());

    let sar = ParabolicSARAnalyzer::new(
        &storage,
        ParabolicSARAnalyzerParams {
            step: 0.02,
            max_step: 0.2,
        },
    );
    assert!(sar.current().unwrap().parabolic_sar.value().is_finite());
}

#[test]
fn standalone_analyzer_next_updates_newest_item() {
    let storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = DonchianAnalyzer::new(&storage, DonchianAnalyzerParams { period: 3 });

    analyzer.next(candle(1, 10.0, 8.0, 9.0, 100.0));
    analyzer.next(candle(2, 11.0, 9.0, 10.0, 110.0));

    assert_eq!(analyzer.items.len(), 2);
    assert_eq!(analyzer.current().unwrap().candle.close_price(), 10.0);
    assert_eq!(analyzer.items[1].candle.close_price(), 9.0);
}
