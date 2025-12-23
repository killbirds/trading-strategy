mod common_test_utils;
use common_test_utils::*;

use trading_strategy::analyzer::base::AnalyzerOps;
use trading_strategy::analyzer::market_structure_analyzer::MarketStructureAnalyzer;
use trading_strategy::candle_store::CandleStore;

#[test]
fn test_market_structure_analyzer_creation() {
    let storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let analyzer = MarketStructureAnalyzer::default(&storage);
    assert_eq!(analyzer.items.len(), 0);
}

#[test]
fn test_market_structure_analyzer_uptrend() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = MarketStructureAnalyzer::default(&storage);

    let candles = create_uptrend_candles(50, 100.0, 2.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_uptrend(1, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_market_structure_analyzer_downtrend() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = MarketStructureAnalyzer::default(&storage);

    let candles = create_downtrend_candles(50, 100.0, 2.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_downtrend(1, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_market_structure_analyzer_strong_structure() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = MarketStructureAnalyzer::default(&storage);

    let candles = create_uptrend_candles(50, 100.0, 3.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_strong_structure();
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_market_structure_analyzer_structure_change_signal() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = MarketStructureAnalyzer::default(&storage);

    let mut candles = create_downtrend_candles(30, 100.0, 2.0);
    candles.extend(create_uptrend_candles(30, 40.0, 2.0));
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_structure_change_signal();
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}
