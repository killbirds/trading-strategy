mod common_test_utils;
use common_test_utils::*;

use trading_strategy::analyzer::base::AnalyzerOps;
use trading_strategy::analyzer::three_rsi_analyzer::ThreeRSIAnalyzer;
use trading_strategy::candle_store::CandleStore;
use trading_strategy::indicator::ma::MAType;

#[test]
fn test_three_rsi_analyzer_creation() {
    let storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let rsi_periods = vec![7, 14, 21];
    let analyzer = ThreeRSIAnalyzer::new(&rsi_periods, &MAType::SMA, 20, 14, &storage);
    assert_eq!(analyzer.items.len(), 0);
}

#[test]
fn test_three_rsi_analyzer_all_less_than_50() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let rsi_periods = vec![7, 14, 21];
    let mut analyzer = ThreeRSIAnalyzer::new(&rsi_periods, &MAType::SMA, 20, 14, &storage);

    let candles = create_downtrend_candles(50, 100.0, 2.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_rsi_all_less_than_50(1, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_three_rsi_analyzer_all_greater_than_50() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let rsi_periods = vec![7, 14, 21];
    let mut analyzer = ThreeRSIAnalyzer::new(&rsi_periods, &MAType::SMA, 20, 14, &storage);

    let candles = create_uptrend_candles(50, 100.0, 2.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_rsi_all_greater_than_50(1, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_three_rsi_analyzer_regular_arrangement() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let rsi_periods = vec![7, 14, 21];
    let mut analyzer = ThreeRSIAnalyzer::new(&rsi_periods, &MAType::SMA, 20, 14, &storage);

    let candles = create_uptrend_candles(50, 100.0, 1.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_rsi_regular_arrangement(1, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_three_rsi_analyzer_adx_greater_than_20() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let rsi_periods = vec![7, 14, 21];
    let mut analyzer = ThreeRSIAnalyzer::new(&rsi_periods, &MAType::SMA, 20, 14, &storage);

    let candles = create_uptrend_candles(50, 100.0, 2.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_adx_greater_than_20(1, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}
