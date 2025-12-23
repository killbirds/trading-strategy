mod common_test_utils;
use common_test_utils::*;

use trading_strategy::analyzer::atr_analyzer::ATRAnalyzer;
use trading_strategy::analyzer::base::AnalyzerOps;
use trading_strategy::candle_store::CandleStore;

#[test]
fn test_atr_analyzer_creation() {
    let storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let periods = vec![14, 21];
    let analyzer = ATRAnalyzer::new(&periods, &storage);
    assert_eq!(analyzer.items.len(), 0);
}

#[test]
fn test_atr_analyzer_volatility_expanding() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let periods = vec![14];
    let mut analyzer = ATRAnalyzer::new(&periods, &storage);

    let mut candles = Vec::new();
    for i in 0..30 {
        let volatility = 1.0 + (i as f64 * 0.2);
        let price = 100.0 + (i as f64);
        candles.push(TestCandle::new(
            i as i64,
            price - volatility / 2.0,
            price + volatility,
            price - volatility,
            price + volatility / 2.0,
            1000.0,
        ));
    }
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_volatility_expanding(14, 5);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_atr_analyzer_volatility_contracting() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let periods = vec![14];
    let mut analyzer = ATRAnalyzer::new(&periods, &storage);

    let mut candles = Vec::new();
    for i in 0..30 {
        let volatility = 10.0 - (i as f64 * 0.2);
        let price = 100.0 + (i as f64);
        candles.push(TestCandle::new(
            i as i64,
            price - volatility / 2.0,
            price + volatility,
            price - volatility,
            price + volatility / 2.0,
            1000.0,
        ));
    }
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_volatility_contracting(14, 5);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_atr_analyzer_high_volatility() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let periods = vec![14];
    let mut analyzer = ATRAnalyzer::new(&periods, &storage);

    let mut candles = Vec::new();
    for i in 0..30 {
        let volatility = 5.0;
        let price = 100.0 + (i as f64);
        candles.push(TestCandle::new(
            i as i64,
            price - volatility / 2.0,
            price + volatility,
            price - volatility,
            price + volatility / 2.0,
            1000.0,
        ));
    }
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_high_volatility(1, 14, 3.0, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_atr_analyzer_volatility_increasing() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let periods = vec![14];
    let mut analyzer = ATRAnalyzer::new(&periods, &storage);

    let mut candles = Vec::new();
    for i in 0..30 {
        let volatility = 1.0 + (i as f64 * 0.3);
        let price = 100.0 + (i as f64);
        candles.push(TestCandle::new(
            i as i64,
            price - volatility / 2.0,
            price + volatility,
            price - volatility,
            price + volatility / 2.0,
            1000.0,
        ));
    }
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_volatility_increasing(5, 14);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}
