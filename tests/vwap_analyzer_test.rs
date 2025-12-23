mod common_test_utils;
use common_test_utils::*;

use trading_strategy::analyzer::base::AnalyzerOps;
use trading_strategy::analyzer::vwap_analyzer::VWAPAnalyzer;
use trading_strategy::candle_store::CandleStore;
use trading_strategy::indicator::vwap::VWAPParams;

#[test]
fn test_vwap_analyzer_creation() {
    let storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let params = vec![VWAPParams::default()];
    let analyzer = VWAPAnalyzer::new(&params, &storage);
    assert_eq!(analyzer.items.len(), 0);
}

#[test]
fn test_vwap_analyzer_price_above_vwap() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let params = vec![VWAPParams::default()];
    let mut analyzer = VWAPAnalyzer::new(&params, &storage);

    let candles = create_uptrend_candles(50, 100.0, 1.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let param = &params[0];
    let result = analyzer.is_price_above_vwap(param, 1, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_vwap_analyzer_price_below_vwap() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let params = vec![VWAPParams::default()];
    let mut analyzer = VWAPAnalyzer::new(&params, &storage);

    let candles = create_downtrend_candles(50, 100.0, 1.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let param = &params[0];
    let result = analyzer.is_price_below_vwap(param, 1, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_vwap_analyzer_breakout_up() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let params = vec![VWAPParams::default()];
    let mut analyzer = VWAPAnalyzer::new(&params, &storage);

    let mut candles = create_downtrend_candles(30, 100.0, 1.0);
    candles.extend(create_uptrend_candles(20, 70.0, 2.0));
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let param = &params[0];
    let result = analyzer.is_vwap_breakout_up(param);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_vwap_analyzer_converging_to_vwap() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let params = vec![VWAPParams::default()];
    let mut analyzer = VWAPAnalyzer::new(&params, &storage);

    let candles = create_sideways_candles(50, 100.0, 2.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let param = &params[0];
    let result = analyzer.is_converging_to_vwap(param, 5);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}
