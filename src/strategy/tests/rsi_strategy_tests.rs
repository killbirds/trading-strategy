use crate::strategy::rsi_strategy::RSIStrategy;
use crate::strategy::tests::common::{
    backtest_strategy, create_downtrend_candles, create_sideways_candles, create_test_storage,
    create_uptrend_candles,
};
use std::collections::HashMap;

// 테스트용 설정 생성 함수
fn create_rsi_config() -> HashMap<String, String> {
    let mut config = HashMap::new();
    config.insert("rsi_count".to_string(), "3".to_string());
    config.insert("rsi_period".to_string(), "14".to_string());
    config.insert("ma".to_string(), "SMA".to_string());
    config.insert("ma_periods".to_string(), "9".to_string());
    config.insert("rsi_lower".to_string(), "30".to_string());
    config.insert("rsi_upper".to_string(), "70".to_string());
    config
}

#[test]
fn test_rsi_strategy_creation() {
    // 테스트 캔들 데이터 생성
    let candles = create_uptrend_candles(50, 100.0, 1.0);
    let storage = create_test_storage(candles);

    // 설정 생성
    let config = create_rsi_config();

    // 전략 인스턴스 생성
    let strategy = RSIStrategy::new_with_config(&storage, Some(config)).unwrap();

    // 인스턴스가 제대로 생성되었는지 확인
    assert!(!strategy.to_string().is_empty());
}

#[test]
fn test_rsi_strategy_signals_uptrend() {
    // 상승장 환경에서 테스트
    // 테스트 캔들 데이터 생성 (명확한 상승 트렌드)
    let candles = create_uptrend_candles(50, 100.0, 2.0);
    let storage = create_test_storage(candles.clone());

    // 설정 생성
    let config = create_rsi_config();

    // 전략 인스턴스 생성
    let strategy = RSIStrategy::new_with_config(&storage, Some(config)).unwrap();

    // 백테스팅 실행
    let result = backtest_strategy(strategy, candles, 10000.0);

    // 결과 출력
    println!("상승장 RSI 전략 결과: {result:?}");

    // 상승장에서는 RSI가 고평가 영역에 진입한 후 하락할 때 매도 신호가 발생해야 함
    // 이는 수익성 있는 거래가 될 가능성이 높음
    println!("총 거래 횟수: {}", result.total_trades);
}

#[test]
fn test_rsi_strategy_signals_downtrend() {
    // 하락장 환경에서 테스트
    // 테스트 캔들 데이터 생성 (명확한 하락 트렌드)
    let candles = create_downtrend_candles(50, 150.0, 2.0);
    let storage = create_test_storage(candles.clone());

    // 설정 생성
    let config = create_rsi_config();

    // 전략 인스턴스 생성
    let strategy = RSIStrategy::new_with_config(&storage, Some(config)).unwrap();

    // 백테스팅 실행
    let result = backtest_strategy(strategy, candles, 10000.0);

    // 결과 출력
    println!("하락장 RSI 전략 결과: {result:?}");
}

#[test]
fn test_rsi_strategy_signals_sideways() {
    // 횡보장 환경에서 테스트
    // 테스트 캔들 데이터 생성 (횡보 트렌드)
    let candles = create_sideways_candles(50, 150.0, 15.0);
    let storage = create_test_storage(candles.clone());

    // 설정 생성
    let config = create_rsi_config();

    // 전략 인스턴스 생성
    let strategy = RSIStrategy::new_with_config(&storage, Some(config)).unwrap();

    // 백테스팅 실행
    let result = backtest_strategy(strategy, candles, 10000.0);

    // 결과 출력
    println!("횡보장 RSI 전략 결과: {result:?}");

    // 횡보장에서는 RSI 기반 전략이 효과적일 가능성이 높음
    // RSI가 과매수/과매도 영역에서 반전하는 신호를 활용하기 때문
    println!("총 거래 횟수: {}", result.total_trades);
}

#[test]
fn test_different_rsi_parameters() {
    // 서로 다른 RSI 파라미터 테스트

    // 테스트 캔들 데이터 생성 (횡보 환경 - RSI 전략의 강점)
    let candles = create_sideways_candles(50, 150.0, 15.0);

    // 표준 RSI(14) 설정
    let mut standard_config = HashMap::new();
    standard_config.insert("rsi_count".to_string(), "3".to_string());
    standard_config.insert("rsi_period".to_string(), "14".to_string());
    standard_config.insert("ma".to_string(), "SMA".to_string());
    standard_config.insert("ma_periods".to_string(), "9".to_string());
    standard_config.insert("rsi_lower".to_string(), "30".to_string());
    standard_config.insert("rsi_upper".to_string(), "70".to_string());

    // 민감한 RSI(7) 설정
    let mut sensitive_config = HashMap::new();
    sensitive_config.insert("rsi_count".to_string(), "3".to_string());
    sensitive_config.insert("rsi_period".to_string(), "7".to_string());
    sensitive_config.insert("ma".to_string(), "SMA".to_string());
    sensitive_config.insert("ma_periods".to_string(), "9".to_string());
    sensitive_config.insert("rsi_lower".to_string(), "30".to_string());
    sensitive_config.insert("rsi_upper".to_string(), "70".to_string());

    // 표준 RSI(14) 테스트
    let storage = create_test_storage(candles.clone());
    let strategy = RSIStrategy::new_with_config(&storage, Some(standard_config)).unwrap();
    let standard_result = backtest_strategy(strategy, candles.clone(), 10000.0);

    // 민감한 RSI(7) 테스트
    let storage = create_test_storage(candles.clone());
    let strategy = RSIStrategy::new_with_config(&storage, Some(sensitive_config)).unwrap();
    let sensitive_result = backtest_strategy(strategy, candles, 10000.0);

    // 결과 출력
    println!("표준 RSI 전략 결과 (14일): {standard_result:?}");
    println!("민감한 RSI 전략 결과 (7일): {sensitive_result:?}");
    println!("표준 RSI 거래 횟수: {}", standard_result.total_trades);
    println!("민감한 RSI 거래 횟수: {}", sensitive_result.total_trades);

    // 더 민감한 RSI(7)가 제대로 설정되었다면 거래 횟수가 더 많아야 함
    assert!(sensitive_result.total_trades >= standard_result.total_trades);
}
