mod common_test_utils;
use common_test_utils::*;

use trading_strategy::analyzer::base::AnalyzerOps;
use trading_strategy::analyzer::price_action_analyzer::PriceActionAnalyzer;
use trading_strategy::candle_store::CandleStore;

#[test]
fn test_price_action_analyzer_creation() {
    let storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let analyzer = PriceActionAnalyzer::default(&storage);
    assert_eq!(analyzer.items.len(), 0);
}

#[test]
fn test_price_action_analyzer_bullish_reversal_signal() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = PriceActionAnalyzer::default(&storage);

    let mut candles = create_downtrend_candles(30, 100.0, 2.0);
    candles.extend(create_uptrend_candles(20, 40.0, 2.0));
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_bullish_reversal_signal();
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_price_action_analyzer_uptrend() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = PriceActionAnalyzer::default(&storage);

    let candles = create_uptrend_candles(50, 100.0, 1.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_uptrend(1, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_price_action_analyzer_strong_trend_continuation() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = PriceActionAnalyzer::default(&storage);

    let candles = create_uptrend_candles(50, 100.0, 3.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_strong_trend_continuation();
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_price_action_analyzer_reversal_pattern() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = PriceActionAnalyzer::default(&storage);

    let candles = create_uptrend_candles(50, 100.0, 1.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_bullish_reversal_signal() || analyzer.is_bearish_reversal_signal();
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}
