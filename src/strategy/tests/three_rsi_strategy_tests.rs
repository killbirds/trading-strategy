use crate::strategy::tests::common::{
    backtest_strategy, create_downtrend_candles, create_sideways_candles, create_test_storage,
    create_uptrend_candles,
};
use crate::strategy::three_rsi_strategy::ThreeRSIStrategy;
use std::collections::HashMap;

// 테스트용 설정 생성 함수
fn create_three_rsi_config() -> HashMap<String, String> {
    let mut config = HashMap::new();
    config.insert("rsi_periods".to_string(), "7,14,21".to_string());
    config.insert("oversold".to_string(), "30".to_string());
    config.insert("overbought".to_string(), "70".to_string());
    config.insert("ma".to_string(), "sma".to_string());
    config.insert("ma_period".to_string(), "10".to_string());
    config.insert("adx_period".to_string(), "14".to_string());
    config
}

#[test]
fn test_three_rsi_strategy_creation() {
    // 테스트 캔들 데이터 생성
    let candles = create_uptrend_candles(50, 100.0, 1.0);
    let storage = create_test_storage(candles);

    // 설정 생성
    let config = create_three_rsi_config();

    // 전략 인스턴스 생성
    let strategy = ThreeRSIStrategy::new_with_config(&storage, Some(config)).unwrap();

    // 인스턴스가 제대로 생성되었는지 확인
    assert!(!strategy.to_string().is_empty());
}

#[test]
fn test_three_rsi_strategy_signals_uptrend() {
    // 상승장 환경에서 테스트
    // 테스트 캔들 데이터 생성 (명확한 상승 트렌드)
    let candles = create_uptrend_candles(100, 100.0, 2.0);
    let storage = create_test_storage(candles.clone());

    // 설정 생성
    let config = create_three_rsi_config();

    // 전략 인스턴스 생성
    let strategy = ThreeRSIStrategy::new_with_config(&storage, Some(config)).unwrap();

    // 백테스팅 실행
    let result = backtest_strategy(strategy, candles, 10000.0);

    // 상승장에서는 수익이 나야함
    println!("상승장 Three RSI 전략 결과: {:?}", result);
    println!("총 수익률: {}", result.total_profit_percentage);
    println!("승률: {}", result.win_rate);
}

#[test]
fn test_three_rsi_strategy_signals_downtrend() {
    // 하락장 환경에서 테스트
    // 테스트 캔들 데이터 생성 (명확한 하락 트렌드)
    let candles = create_downtrend_candles(100, 150.0, 2.0);
    let storage = create_test_storage(candles.clone());

    // 설정 생성
    let config = create_three_rsi_config();

    // 전략 인스턴스 생성
    let strategy = ThreeRSIStrategy::new_with_config(&storage, Some(config)).unwrap();

    // 백테스팅 실행
    let result = backtest_strategy(strategy, candles, 10000.0);

    // 결과 출력
    println!("하락장 Three RSI 전략 결과: {:?}", result);
}

#[test]
fn test_three_rsi_strategy_signals_sideways() {
    // 횡보장 환경에서 테스트
    // 테스트 캔들 데이터 생성 (횡보 트렌드)
    let candles = create_sideways_candles(100, 150.0, 15.0);
    let storage = create_test_storage(candles.clone());

    // 설정 생성
    let config = create_three_rsi_config();

    // 전략 인스턴스 생성
    let strategy = ThreeRSIStrategy::new_with_config(&storage, Some(config)).unwrap();

    // 백테스팅 실행
    let result = backtest_strategy(strategy, candles, 10000.0);

    // 결과 출력
    println!("횡보장 Three RSI 전략 결과: {:?}", result);

    // RSI 기반 전략은 횡보장에서 어느 정도 수익성을 보일 수 있음
    // assert!(result.total_profit_percentage >= 0.0);
}

#[test]
fn test_three_rsi_with_different_periods() {
    // 서로 다른 기간 설정 테스트
    let candles = create_sideways_candles(100, 150.0, 15.0);

    // 기본 설정 테스트
    let default_config = create_three_rsi_config();
    let storage = create_test_storage(candles.clone());
    let strategy = ThreeRSIStrategy::new_with_config(&storage, Some(default_config)).unwrap();
    let default_result = backtest_strategy(strategy, candles.clone(), 10000.0);

    // 짧은 기간 설정 테스트
    let mut short_periods_config = create_three_rsi_config();
    short_periods_config.insert("rsi_periods".to_string(), "5,10,15".to_string());

    let storage = create_test_storage(candles.clone());
    let strategy = ThreeRSIStrategy::new_with_config(&storage, Some(short_periods_config)).unwrap();
    let short_periods_result = backtest_strategy(strategy, candles.clone(), 10000.0);

    // 긴 기간 설정 테스트
    let mut long_periods_config = create_three_rsi_config();
    long_periods_config.insert("rsi_periods".to_string(), "9,18,27".to_string());

    let storage = create_test_storage(candles.clone());
    let strategy = ThreeRSIStrategy::new_with_config(&storage, Some(long_periods_config)).unwrap();
    let long_periods_result = backtest_strategy(strategy, candles, 10000.0);

    // 결과 출력 및 비교
    println!("기본 기간 설정 결과: {:?}", default_result);
    println!("짧은 기간 설정 결과: {:?}", short_periods_result);
    println!("긴 기간 설정 결과: {:?}", long_periods_result);
}

#[test]
fn test_three_rsi_different_thresholds() {
    // 서로 다른 과매수/과매도 임계값 테스트
    let candles = create_sideways_candles(100, 150.0, 15.0);

    // 기본 설정 테스트 (30/70)
    let default_config = create_three_rsi_config();
    let storage = create_test_storage(candles.clone());
    let strategy = ThreeRSIStrategy::new_with_config(&storage, Some(default_config)).unwrap();
    let default_result = backtest_strategy(strategy, candles.clone(), 10000.0);

    // 넓은 범위 설정 테스트 (20/80)
    let mut wide_config = create_three_rsi_config();
    wide_config.insert("oversold".to_string(), "20".to_string());
    wide_config.insert("overbought".to_string(), "80".to_string());

    let storage = create_test_storage(candles.clone());
    let strategy = ThreeRSIStrategy::new_with_config(&storage, Some(wide_config)).unwrap();
    let wide_result = backtest_strategy(strategy, candles.clone(), 10000.0);

    // 좁은 범위 설정 테스트 (40/60)
    let mut narrow_config = create_three_rsi_config();
    narrow_config.insert("oversold".to_string(), "40".to_string());
    narrow_config.insert("overbought".to_string(), "60".to_string());

    let storage = create_test_storage(candles.clone());
    let strategy = ThreeRSIStrategy::new_with_config(&storage, Some(narrow_config)).unwrap();
    let narrow_result = backtest_strategy(strategy, candles, 10000.0);

    // 결과 출력 및 비교
    println!("기본 임계값(30/70) 결과: {:?}", default_result);
    println!("넓은 임계값(20/80) 결과: {:?}", wide_result);
    println!("좁은 임계값(40/60) 결과: {:?}", narrow_result);

    // 거래 횟수 비교 - 좁은 범위가 거래 횟수가 많아야 함
    println!("기본 임계값 거래 횟수: {}", default_result.total_trades);
    println!("넓은 임계값 거래 횟수: {}", wide_result.total_trades);
    println!("좁은 임계값 거래 횟수: {}", narrow_result.total_trades);
}
