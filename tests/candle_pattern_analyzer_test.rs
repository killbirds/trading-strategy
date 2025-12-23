mod common_test_utils;
use common_test_utils::*;

use trading_strategy::analyzer::base::AnalyzerOps;
use trading_strategy::analyzer::candle_pattern_analyzer::CandlePatternAnalyzer;
use trading_strategy::candle_store::CandleStore;

#[test]
fn test_candle_pattern_analyzer_creation() {
    let storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let analyzer = CandlePatternAnalyzer::default(&storage);
    assert_eq!(analyzer.items.len(), 0);
}

#[test]
fn test_candle_pattern_analyzer_strong_reversal_signal() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = CandlePatternAnalyzer::default(&storage);

    let mut candles = create_downtrend_candles(30, 100.0, 2.0);
    candles.extend(create_uptrend_candles(20, 40.0, 2.0));
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_strong_reversal_signal();
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_candle_pattern_analyzer_high_confidence_signal() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = CandlePatternAnalyzer::default(&storage);

    let candles = create_uptrend_candles(50, 100.0, 2.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_high_confidence_signal();
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_candle_pattern_analyzer_volume_confirmed_signal() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = CandlePatternAnalyzer::default(&storage);

    let mut candles = Vec::new();
    for i in 0..50 {
        let volume = if i > 40 { 3000.0 } else { 1000.0 };
        let price = 100.0 + (i as f64 * 2.0);
        candles.push(TestCandle::new(
            i as i64,
            price - 1.0,
            price + 2.0,
            price - 2.0,
            price + 1.0,
            volume,
        ));
    }
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_volume_confirmed_signal();
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_candle_pattern_analyzer_reversal_pattern() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = CandlePatternAnalyzer::default(&storage);

    let candles = create_uptrend_candles(50, 100.0, 1.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_reversal_pattern_signal(1, 0, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}
