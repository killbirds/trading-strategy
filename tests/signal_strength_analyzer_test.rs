mod common_test_utils;
use common_test_utils::*;

use trading_strategy::analyzer::base::AnalyzerOps;
use trading_strategy::analyzer::signal_strength_analyzer::SignalStrengthAnalyzer;
use trading_strategy::candle_store::CandleStore;

#[test]
fn test_signal_strength_analyzer_creation() {
    let storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let analyzer = SignalStrengthAnalyzer::default(&storage);
    assert_eq!(analyzer.items.len(), 0);
}

#[test]
fn test_signal_strength_analyzer_strong_buy_signal() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = SignalStrengthAnalyzer::default(&storage);

    let candles = create_uptrend_candles(50, 100.0, 2.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_strong_buy_signal();
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_signal_strength_analyzer_strong_sell_signal() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = SignalStrengthAnalyzer::default(&storage);

    let candles = create_downtrend_candles(50, 100.0, 2.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_strong_sell_signal();
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_signal_strength_analyzer_high_quality_signal() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = SignalStrengthAnalyzer::default(&storage);

    let candles = create_uptrend_candles(50, 100.0, 1.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_high_quality_signal();
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_signal_strength_analyzer_consistent_signal() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = SignalStrengthAnalyzer::default(&storage);

    let candles = create_uptrend_candles(50, 100.0, 1.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_consistent_signal(5);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_signal_strength_analyzer_strong_buy_signal_confirmed() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = SignalStrengthAnalyzer::default(&storage);

    let candles = create_uptrend_candles(50, 100.0, 2.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_strong_buy_signal_confirmed(1, 1, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_signal_strength_analyzer_good_market_condition() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = SignalStrengthAnalyzer::default(&storage);

    let candles = create_uptrend_candles(50, 100.0, 1.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_good_market_condition(1, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}
