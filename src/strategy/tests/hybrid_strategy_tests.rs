use crate::strategy::hybrid_strategy::HybridStrategy;
use crate::strategy::ma_strategy::MAStrategy;
use crate::strategy::rsi_strategy::RSIStrategy;
use crate::strategy::tests::common::{
    backtest_strategy, create_downtrend_candles, create_sideways_candles, create_test_storage,
    create_uptrend_candles,
};
use std::collections::HashMap;

// 테스트용 설정 생성 함수
fn create_hybrid_config() -> HashMap<String, String> {
    let mut config = HashMap::new();
    config.insert("count".to_string(), "30".to_string());
    config.insert("ma_type".to_string(), "sma".to_string());
    config.insert("ma_period".to_string(), "20".to_string());
    config.insert("macd_fast_period".to_string(), "12".to_string());
    config.insert("macd_slow_period".to_string(), "26".to_string());
    config.insert("macd_signal_period".to_string(), "9".to_string());
    config.insert("rsi_period".to_string(), "14".to_string());
    config.insert("rsi_lower".to_string(), "30".to_string());
    config.insert("rsi_upper".to_string(), "70".to_string());
    config
}

#[test]
fn test_hybrid_strategy_creation() {
    // 테스트 캔들 데이터 생성
    let candles = create_uptrend_candles(50, 100.0, 1.0);
    let storage = create_test_storage(candles);

    // 설정 생성
    let config = create_hybrid_config();

    // 전략 인스턴스 생성
    let strategy = HybridStrategy::new_with_config(&storage, Some(config)).unwrap();

    // 인스턴스가 제대로 생성되었는지 확인
    assert!(!strategy.to_string().is_empty());
}

#[test]
fn test_hybrid_strategy_signals_uptrend() {
    // 상승장 환경에서 테스트
    // 테스트 캔들 데이터 생성 (명확한 상승 트렌드)
    let candles = create_uptrend_candles(60, 100.0, 2.0);
    let storage = create_test_storage(candles.clone());

    // 설정 생성
    let config = create_hybrid_config();

    // 전략 인스턴스 생성
    let strategy = HybridStrategy::new_with_config(&storage, Some(config)).unwrap();

    // 백테스팅 실행
    let result = backtest_strategy(strategy, candles, 10000.0);

    // 결과 출력
    println!("상승장 하이브리드 전략 결과: {:?}", result);

    // 상승장에서는 긍정적인 결과가 나와야 함
    assert!(result.total_profit_percentage > 0.0);
}

#[test]
fn test_hybrid_strategy_signals_downtrend() {
    // 하락장 환경에서 테스트
    // 테스트 캔들 데이터 생성 (명확한 하락 트렌드)
    let candles = create_downtrend_candles(60, 200.0, 2.0);
    let storage = create_test_storage(candles.clone());

    // 설정 생성
    let config = create_hybrid_config();

    // 전략 인스턴스 생성
    let strategy = HybridStrategy::new_with_config(&storage, Some(config)).unwrap();

    // 백테스팅 실행
    let result = backtest_strategy(strategy, candles, 10000.0);

    // 결과 출력
    println!("하락장 하이브리드 전략 결과: {:?}", result);
}

#[test]
fn test_hybrid_strategy_signals_sideways() {
    // 횡보장 환경에서 테스트
    // 테스트 캔들 데이터 생성 (횡보 트렌드)
    let candles = create_sideways_candles(100, 150.0, 15.0);
    let storage = create_test_storage(candles.clone());

    // 설정 생성
    let config = create_hybrid_config();

    // 전략 인스턴스 생성
    let strategy = HybridStrategy::new_with_config(&storage, Some(config)).unwrap();

    // 백테스팅 실행
    let result = backtest_strategy(strategy, candles, 10000.0);

    // 결과 출력
    println!("횡보장 하이브리드 전략 결과: {:?}", result);
}

#[test]
fn test_hybrid_vs_individual_strategies() {
    // 하이브리드 전략과 개별 전략 비교 테스트
    // 테스트 캔들 데이터 생성 (상승 -> 하락 -> 상승)
    let candles = create_uptrend_candles(100, 100.0, 1.0);
    let storage = create_test_storage(candles.clone());

    // MA 전략 설정
    let mut ma_config = HashMap::new();
    ma_config.insert("ma".to_string(), "EMA".to_string());
    ma_config.insert("ma_periods".to_string(), "5,10,20".to_string());
    ma_config.insert("cross_previous_periods".to_string(), "15".to_string());

    // RSI 전략 설정
    let mut rsi_config = HashMap::new();
    rsi_config.insert("rsi_count".to_string(), "3".to_string());
    rsi_config.insert("rsi_period".to_string(), "14".to_string());
    rsi_config.insert("ma".to_string(), "SMA".to_string());
    rsi_config.insert("ma_periods".to_string(), "9".to_string());
    rsi_config.insert("rsi_lower".to_string(), "30".to_string());
    rsi_config.insert("rsi_upper".to_string(), "70".to_string());

    // 하이브리드 전략 설정
    let hybrid_config = create_hybrid_config();

    // MA 전략 테스트
    let strategy = MAStrategy::new_with_config(&storage, Some(ma_config)).unwrap();
    let ma_result = backtest_strategy(strategy, candles.clone(), 10000.0);

    // RSI 전략 테스트
    let storage = create_test_storage(candles.clone());
    let strategy = RSIStrategy::new_with_config(&storage, Some(rsi_config)).unwrap();
    let rsi_result = backtest_strategy(strategy, candles.clone(), 10000.0);

    // 하이브리드 전략 테스트
    let storage = create_test_storage(candles.clone());
    let strategy = HybridStrategy::new_with_config(&storage, Some(hybrid_config)).unwrap();
    let hybrid_result = backtest_strategy(strategy, candles, 10000.0);

    // 결과 출력
    println!("MA 전략 결과: {:?}", ma_result);
    println!("RSI 전략 결과: {:?}", rsi_result);
    println!("하이브리드 전략 결과: {:?}", hybrid_result);
}
