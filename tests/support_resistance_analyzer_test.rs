mod common_test_utils;
use common_test_utils::*;

use trading_strategy::analyzer::base::AnalyzerOps;
use trading_strategy::analyzer::support_resistance_analyzer::SupportResistanceAnalyzer;
use trading_strategy::candle_store::CandleStore;

#[test]
fn test_support_resistance_analyzer_creation() {
    let storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let analyzer = SupportResistanceAnalyzer::default(&storage);
    assert_eq!(analyzer.items.len(), 0);
}

#[test]
fn test_support_resistance_analyzer_breakdown() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = SupportResistanceAnalyzer::default(&storage);

    let mut candles = Vec::new();
    for i in 0..60 {
        let price = if i < 50 { 100.0 } else { 90.0 };
        candles.push(TestCandle::new(
            i as i64,
            price,
            price + 2.0,
            price - 2.0,
            price,
            1000.0,
        ));
    }
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_support_breakdown();
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_support_resistance_analyzer_breakout() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = SupportResistanceAnalyzer::default(&storage);

    let mut candles = Vec::new();
    for i in 0..60 {
        let price = if i < 50 { 100.0 } else { 110.0 };
        candles.push(TestCandle::new(
            i as i64,
            price,
            price + 2.0,
            price - 2.0,
            price,
            1000.0,
        ));
    }
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_resistance_breakout();
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_support_resistance_analyzer_near_support() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = SupportResistanceAnalyzer::default(&storage);

    let candles = create_sideways_candles(60, 100.0, 5.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_near_support(1, 1.0, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}
