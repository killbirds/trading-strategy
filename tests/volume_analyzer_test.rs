mod common_test_utils;
use common_test_utils::*;

use trading_strategy::analyzer::base::AnalyzerOps;
use trading_strategy::analyzer::volume_analyzer::VolumeAnalyzer;
use trading_strategy::candle_store::CandleStore;

#[test]
fn test_volume_analyzer_creation() {
    let storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let periods = vec![10, 20];
    let analyzer = VolumeAnalyzer::new(&periods, &storage);
    assert_eq!(analyzer.items.len(), 0);
}

#[test]
fn test_volume_analyzer_above_average() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let periods = vec![10];
    let mut analyzer = VolumeAnalyzer::new(&periods, &storage);

    let mut candles = Vec::new();
    for i in 0..30 {
        let volume = if i < 20 { 1000.0 } else { 3000.0 };
        let price = 100.0 + (i as f64 * 0.5);
        candles.push(TestCandle::new(
            i as i64,
            price,
            price + 1.0,
            price - 1.0,
            price,
            volume,
        ));
    }
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_volume_above_average(1, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_volume_analyzer_surge() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let periods = vec![10];
    let mut analyzer = VolumeAnalyzer::new(&periods, &storage);

    let mut candles = Vec::new();
    for i in 0..30 {
        let volume = if i < 29 { 1000.0 } else { 5000.0 };
        let price = 100.0 + (i as f64 * 0.5);
        candles.push(TestCandle::new(
            i as i64,
            price,
            price + 1.0,
            price - 1.0,
            price,
            volume,
        ));
    }
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_volume_surge(10, 2.0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_volume_analyzer_bullish_with_increased_volume() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let periods = vec![10];
    let mut analyzer = VolumeAnalyzer::new(&periods, &storage);

    let mut candles = Vec::new();
    for i in 0..30 {
        let volume = if i < 20 { 1000.0 } else { 2500.0 };
        let price = 100.0 + (i as f64 * 1.0);
        candles.push(TestCandle::new(
            i as i64,
            price - 0.5,
            price + 1.0,
            price - 1.0,
            price + 0.5,
            volume,
        ));
    }
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_bullish_with_increased_volume(1, 10, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}
