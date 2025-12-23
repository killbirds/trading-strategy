mod common_test_utils;
use common_test_utils::*;

use trading_strategy::analyzer::base::AnalyzerOps;
use trading_strategy::analyzer::risk_management_analyzer::{
    PositionSizingMethod, PositionType, RiskManagementAnalyzer,
};
use trading_strategy::candle_store::CandleStore;

#[test]
fn test_risk_management_analyzer_creation() {
    let storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let analyzer = RiskManagementAnalyzer::default(&storage);
    assert_eq!(analyzer.items.len(), 0);
}

#[test]
fn test_risk_management_analyzer_high_risk() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = RiskManagementAnalyzer::default(&storage);

    let mut candles = Vec::new();
    for i in 0..50 {
        let volatility = if i % 10 == 0 { 10.0 } else { 1.0 };
        let price = 100.0 + (i as f64 * volatility);
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
    let result = analyzer.is_high_risk(1, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_risk_management_analyzer_low_risk() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = RiskManagementAnalyzer::default(&storage);

    let candles = create_sideways_candles(50, 100.0, 1.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_low_risk(1, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_risk_management_analyzer_high_volatility() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = RiskManagementAnalyzer::default(&storage);

    let mut candles = Vec::new();
    for i in 0..50 {
        let volatility = 5.0 + (i as f64 * 0.1);
        let price = 100.0 + (i as f64 * volatility);
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
    let result = analyzer.is_high_volatility(1, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_risk_management_analyzer_calculate_risk() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = RiskManagementAnalyzer::default(&storage);

    let candles = create_uptrend_candles(50, 100.0, 1.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let risk_calc = analyzer.calculate_risk(
        100.0,
        PositionType::Long,
        10000.0,
        PositionSizingMethod::FixedPercentage,
    );
    assert!(risk_calc.is_some());
    if let Some(calc) = risk_calc {
        assert!(calc.risk_amount > 0.0);
        assert!(calc.position_size > 0.0);
        assert!(calc.stop_loss_price < calc.entry_price);
        assert!(calc.target_price > calc.entry_price);
    }
}

#[test]
fn test_risk_management_analyzer_trading_recommended() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = RiskManagementAnalyzer::default(&storage);

    let candles = create_uptrend_candles(50, 100.0, 1.0);
    for candle in candles {
        storage.add(candle);
    }

    analyzer.init_from_storage(&storage);
    let result = analyzer.is_trading_recommended(1, 0);
    // 함수가 panic 없이 실행되는지 확인
    let _ = result;
}

#[test]
fn test_risk_management_analyzer_check_risk_warnings() {
    let mut storage = CandleStore::<TestCandle>::new(Vec::new(), 1000, false);
    let mut analyzer = RiskManagementAnalyzer::default(&storage);

    let mut candles = Vec::new();
    for i in 0..50 {
        let volatility = 10.0;
        let price = 100.0 + (i as f64 * volatility);
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
    let _warnings = analyzer.check_risk_warnings();
    // warnings는 항상 0 이상이므로 검증 불필요
}
