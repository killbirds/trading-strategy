mod common_test_utils;
use common_test_utils::*;

use trading_strategy::analyzer::base::AnalyzerOps;
use trading_strategy::analyzer::momentum_analyzer::MomentumAnalyzer;
use trading_strategy::candle_store::CandleStore;

#[test]
fn test_momentum_analyzer_creation() {
    let storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let analyzer = MomentumAnalyzer::default(&storage);
    assert_eq!(analyzer.items.len(), 0);
}

#[test]
fn test_momentum_analyzer_strong_momentum_signal() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = MomentumAnalyzer::default(&storage);

    let candles = create_uptrend_candles(50, 100.0, 2.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_strong_momentum_signal();
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_momentum_analyzer_divergence_signal() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = MomentumAnalyzer::default(&storage);

    let candles = create_uptrend_candles(50, 100.0, 1.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_momentum_divergence_signal();
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_momentum_analyzer_reversal_signal() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = MomentumAnalyzer::default(&storage);

    let mut candles = create_uptrend_candles(30, 100.0, 2.0);
    candles.extend(create_downtrend_candles(20, 160.0, 2.0));
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_momentum_reversal_signal();
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_momentum_analyzer_persistent_momentum_signal() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = MomentumAnalyzer::default(&storage);

    let candles = create_uptrend_candles(50, 100.0, 1.5);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_persistent_momentum_signal();
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_momentum_analyzer_strong_positive_momentum() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = MomentumAnalyzer::default(&storage);

    let candles = create_uptrend_candles(50, 100.0, 2.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_strong_positive_momentum(1, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_momentum_analyzer_overbought() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = MomentumAnalyzer::default(&storage);

    let candles = create_uptrend_candles(50, 100.0, 3.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_overbought(1, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_momentum_analyzer_oversold() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = MomentumAnalyzer::default(&storage);

    let candles = create_downtrend_candles(50, 100.0, 3.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_oversold(1, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}
