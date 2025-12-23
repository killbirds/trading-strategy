mod common_test_utils;
use common_test_utils::*;

use trading_strategy::analyzer::base::AnalyzerOps;
use trading_strategy::analyzer::ichimoku_analyzer::IchimokuAnalyzer;
use trading_strategy::candle_store::CandleStore;
use trading_strategy::indicator::ichimoku::IchimokuParams;

#[test]
fn test_ichimoku_analyzer_creation() {
    let storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let params = vec![IchimokuParams {
        tenkan_period: 9,
        kijun_period: 26,
        senkou_period: 52,
    }];
    let analyzer = IchimokuAnalyzer::new(&params, &storage);
    assert_eq!(analyzer.items.len(), 0);
}

#[test]
fn test_ichimoku_analyzer_price_above_cloud() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let params = vec![IchimokuParams {
        tenkan_period: 9,
        kijun_period: 26,
        senkou_period: 52,
    }];
    let mut analyzer = IchimokuAnalyzer::new(&params, &storage);

    let candles = create_uptrend_candles(60, 100.0, 2.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let param = &params[0];
    let result = analyzer.is_price_above_cloud(param, 1, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_ichimoku_analyzer_golden_cross() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let params = vec![IchimokuParams {
        tenkan_period: 9,
        kijun_period: 26,
        senkou_period: 52,
    }];
    let mut analyzer = IchimokuAnalyzer::new(&params, &storage);

    let mut candles = create_downtrend_candles(30, 100.0, 1.0);
    candles.extend(create_uptrend_candles(30, 70.0, 2.0));
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let param = &params[0];
    let result = analyzer.is_golden_cross(param);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_ichimoku_analyzer_buy_signal() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let params = vec![IchimokuParams {
        tenkan_period: 9,
        kijun_period: 26,
        senkou_period: 52,
    }];
    let mut analyzer = IchimokuAnalyzer::new(&params, &storage);

    let candles = create_uptrend_candles(60, 100.0, 2.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let param = &params[0];
    let result = analyzer.is_buy_signal(param, 1, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}
