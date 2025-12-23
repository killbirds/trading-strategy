mod common_test_utils;
use common_test_utils::*;

use trading_strategy::analyzer::supertrend_analyzer::SuperTrendAnalyzer;
use trading_strategy::candle_store::CandleStore;

#[test]
fn test_supertrend_analyzer_creation() {
    let storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let periods = vec![(10, 2.0), (20, 3.0)];
    let analyzer = SuperTrendAnalyzer::new(&periods, &storage);
    assert_eq!(analyzer.items.len(), 0);
}

#[test]
fn test_supertrend_analyzer_all_uptrend() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let periods = vec![(10, 2.0)];
    let mut analyzer = SuperTrendAnalyzer::new(&periods, &storage);

    let candles = create_uptrend_candles(50, 100.0, 2.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_all_uptrend();
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_supertrend_analyzer_price_above_supertrend() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let periods = vec![(10, 2.0)];
    let mut analyzer = SuperTrendAnalyzer::new(&periods, &storage);

    let candles = create_uptrend_candles(50, 100.0, 2.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let period = 10;
    let multiplier = 2.0;
    let result = analyzer.is_price_above_supertrend(&period, &multiplier);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_supertrend_analyzer_trend_changed() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let periods = vec![(10, 2.0)];
    let mut analyzer = SuperTrendAnalyzer::new(&periods, &storage);

    let mut candles = create_downtrend_candles(30, 100.0, 2.0);
    candles.extend(create_uptrend_candles(20, 40.0, 2.0));
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let period = 10;
    let multiplier = 2.0;
    let result = analyzer.is_trend_changed(&period, &multiplier, 5);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}
