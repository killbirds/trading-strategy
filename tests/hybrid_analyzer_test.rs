mod common_test_utils;
use common_test_utils::*;

use trading_strategy::analyzer::base::AnalyzerOps;
use trading_strategy::analyzer::hybrid_analyzer::HybridAnalyzer;
use trading_strategy::candle_store::CandleStore;
use trading_strategy::indicator::ma::MAType;

#[test]
fn test_hybrid_analyzer_creation() {
    let storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let analyzer = HybridAnalyzer::new(&MAType::SMA, 20, 12, 26, 9, 14, &storage);
    assert_eq!(analyzer.items.len(), 0);
}

#[test]
fn test_hybrid_analyzer_buy_signal_strength() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = HybridAnalyzer::new(&MAType::SMA, 20, 12, 26, 9, 14, &storage);

    let candles = create_uptrend_candles(50, 100.0, 1.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let strength = analyzer.calculate_buy_signal_strength(30.0);
    assert!((0.0..=1.0).contains(&strength));
}

#[test]
fn test_hybrid_analyzer_sell_signal_strength() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = HybridAnalyzer::new(&MAType::SMA, 20, 12, 26, 9, 14, &storage);

    let candles = create_downtrend_candles(50, 100.0, 1.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let strength = analyzer.calculate_sell_signal_strength(70.0, 0.0);
    assert!((0.0..=1.0).contains(&strength));
}

#[test]
fn test_hybrid_analyzer_enhanced_buy_signal() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = HybridAnalyzer::new(&MAType::SMA, 20, 12, 26, 9, 14, &storage);

    let candles = create_uptrend_candles(50, 100.0, 1.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let strength = analyzer.calculate_enhanced_buy_signal_strength(30.0, 0.02, 1.2);
    assert!((0.0..=1.0).contains(&strength));
}

#[test]
fn test_hybrid_analyzer_market_condition() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = HybridAnalyzer::new(&MAType::SMA, 20, 12, 26, 9, 14, &storage);

    let candles = create_uptrend_candles(50, 100.0, 1.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let condition = analyzer.evaluate_market_condition();
    assert!(!condition.is_empty());
}

#[test]
fn test_hybrid_analyzer_strong_buy_signal() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = HybridAnalyzer::new(&MAType::SMA, 20, 12, 26, 9, 14, &storage);

    let candles = create_uptrend_candles(50, 100.0, 2.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_strong_buy_signal(1, 30.0, 0.5, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_hybrid_analyzer_good_market_condition() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = HybridAnalyzer::new(&MAType::SMA, 20, 12, 26, 9, 14, &storage);

    let candles = create_uptrend_candles(50, 100.0, 1.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_good_market_condition(1, 1.0, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}
