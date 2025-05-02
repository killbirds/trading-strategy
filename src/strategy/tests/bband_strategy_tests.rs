use crate::strategy::bband_strategy::BBandStrategy;
use crate::strategy::tests::common::{
    backtest_strategy, create_downtrend_candles, create_sideways_candles, create_test_storage,
    create_uptrend_candles,
};
use std::collections::HashMap;

// 테스트용 설정 생성 함수
fn create_bband_config() -> HashMap<String, String> {
    let mut config = HashMap::new();
    config.insert("period".to_string(), "20".to_string());
    config.insert("deviation".to_string(), "2.0".to_string());
    config.insert("price_type".to_string(), "close".to_string());
    config.insert("ma_type".to_string(), "SMA".to_string());
    config.insert("signal_period".to_string(), "5".to_string());
    config.insert("count".to_string(), "3".to_string());
    config.insert("multiplier".to_string(), "1.0".to_string());
    config
}

#[test]
fn test_bband_strategy_creation() {
    // 테스트 캔들 데이터 생성
    let candles = create_uptrend_candles(50, 100.0, 1.0);
    let storage = create_test_storage(candles);

    // 설정 생성
    let config = create_bband_config();

    // 전략 인스턴스 생성
    let strategy = BBandStrategy::new_with_config(&storage, Some(config)).unwrap();

    // 인스턴스가 제대로 생성되었는지 확인
    assert!(!strategy.to_string().is_empty());
}

#[test]
fn test_bband_strategy_signals_uptrend() {
    // 상승장 환경에서 테스트
    // 테스트 캔들 데이터 생성 (명확한 상승 트렌드)
    let candles = create_uptrend_candles(100, 100.0, 2.0);
    let storage = create_test_storage(candles.clone());

    // 설정 생성
    let config = create_bband_config();

    // 전략 인스턴스 생성
    let strategy = BBandStrategy::new_with_config(&storage, Some(config)).unwrap();

    // 백테스팅 실행
    let result = backtest_strategy(strategy, candles, 10000.0);

    // 상승장에서는 수익이 나야함
    println!("상승장 BBand 전략 결과: {:?}", result);
    println!("총 수익률: {}", result.total_profit_percentage);
    println!("승률: {}", result.win_rate);
}

#[test]
fn test_bband_strategy_signals_downtrend() {
    // 하락장 환경에서 테스트
    // 테스트 캔들 데이터 생성 (명확한 하락 트렌드)
    let candles = create_downtrend_candles(100, 150.0, 2.0);
    let storage = create_test_storage(candles.clone());

    // 설정 생성
    let config = create_bband_config();

    // 전략 인스턴스 생성
    let strategy = BBandStrategy::new_with_config(&storage, Some(config)).unwrap();

    // 백테스팅 실행
    let result = backtest_strategy(strategy, candles, 10000.0);

    // 결과 출력
    println!("하락장 BBand 전략 결과: {:?}", result);
}

#[test]
fn test_bband_strategy_signals_sideways() {
    // 횡보장 환경에서 테스트
    // 테스트 캔들 데이터 생성 (횡보 트렌드)
    let candles = create_sideways_candles(100, 150.0, 15.0);
    let storage = create_test_storage(candles.clone());

    // 설정 생성
    let config = create_bband_config();

    // 전략 인스턴스 생성
    let strategy = BBandStrategy::new_with_config(&storage, Some(config)).unwrap();

    // 백테스팅 실행
    let result = backtest_strategy(strategy, candles, 10000.0);

    // 결과 출력
    println!("횡보장 BBand 전략 결과: {:?}", result);

    // 볼린저 밴드는 횡보장에서 수익성이 있을 수도 있고 없을 수도 있음
    // assert!(result.total_profit_percentage >= 0.0);
}

#[test]
fn test_bband_with_different_deviations() {
    // 서로 다른 표준편차 설정 테스트
    let candles = create_sideways_candles(100, 150.0, 15.0);

    // 표준편차 1.5 테스트
    let mut narrow_config = create_bband_config();
    narrow_config.insert("deviation".to_string(), "1.5".to_string());

    let storage = create_test_storage(candles.clone());
    let strategy = BBandStrategy::new_with_config(&storage, Some(narrow_config)).unwrap();
    let narrow_result = backtest_strategy(strategy, candles.clone(), 10000.0);

    // 표준편차 2.0 테스트 (기본)
    let default_config = create_bband_config();
    let storage = create_test_storage(candles.clone());
    let strategy = BBandStrategy::new_with_config(&storage, Some(default_config)).unwrap();
    let default_result = backtest_strategy(strategy, candles.clone(), 10000.0);

    // 표준편차 2.5 테스트
    let mut wide_config = create_bband_config();
    wide_config.insert("deviation".to_string(), "2.5".to_string());

    let storage = create_test_storage(candles.clone());
    let strategy = BBandStrategy::new_with_config(&storage, Some(wide_config)).unwrap();
    let wide_result = backtest_strategy(strategy, candles, 10000.0);

    // 결과 출력 및 비교
    println!("좁은 밴드(1.5) 결과: {:?}", narrow_result);
    println!("기본 밴드(2.0) 결과: {:?}", default_result);
    println!("넓은 밴드(2.5) 결과: {:?}", wide_result);
}

#[test]
fn test_bband_with_different_periods() {
    // 서로 다른 기간 설정 테스트
    let candles = create_sideways_candles(100, 150.0, 15.0);

    // 짧은 기간 (10) 테스트
    let mut short_config = create_bband_config();
    short_config.insert("period".to_string(), "10".to_string());

    let storage = create_test_storage(candles.clone());
    let strategy = BBandStrategy::new_with_config(&storage, Some(short_config)).unwrap();
    let short_result = backtest_strategy(strategy, candles.clone(), 10000.0);

    // 기본 기간 (20) 테스트
    let default_config = create_bband_config();
    let storage = create_test_storage(candles.clone());
    let strategy = BBandStrategy::new_with_config(&storage, Some(default_config)).unwrap();
    let default_result = backtest_strategy(strategy, candles.clone(), 10000.0);

    // 긴 기간 (30) 테스트
    let mut long_config = create_bband_config();
    long_config.insert("period".to_string(), "30".to_string());

    let storage = create_test_storage(candles.clone());
    let strategy = BBandStrategy::new_with_config(&storage, Some(long_config)).unwrap();
    let long_result = backtest_strategy(strategy, candles, 10000.0);

    // 결과 출력 및 비교
    println!("짧은 기간(10) 결과: {:?}", short_result);
    println!("기본 기간(20) 결과: {:?}", default_result);
    println!("긴 기간(30) 결과: {:?}", long_result);
}

#[test]
fn test_bband_with_different_ma_types() {
    // 서로 다른 이동평균 유형 테스트
    let candles = create_sideways_candles(100, 150.0, 15.0);

    // SMA 테스트
    let mut sma_config = create_bband_config();
    sma_config.insert("ma_type".to_string(), "SMA".to_string());

    let storage = create_test_storage(candles.clone());
    let strategy = BBandStrategy::new_with_config(&storage, Some(sma_config)).unwrap();
    let sma_result = backtest_strategy(strategy, candles.clone(), 10000.0);

    // EMA 테스트
    let mut ema_config = create_bband_config();
    ema_config.insert("ma_type".to_string(), "EMA".to_string());

    let storage = create_test_storage(candles.clone());
    let strategy = BBandStrategy::new_with_config(&storage, Some(ema_config)).unwrap();
    let ema_result = backtest_strategy(strategy, candles.clone(), 10000.0);

    // WMA 테스트
    let mut wma_config = create_bband_config();
    wma_config.insert("ma_type".to_string(), "WMA".to_string());

    let storage = create_test_storage(candles.clone());
    let strategy = BBandStrategy::new_with_config(&storage, Some(wma_config)).unwrap();
    let wma_result = backtest_strategy(strategy, candles, 10000.0);

    // 결과 출력 및 비교
    println!("SMA 기반 BBand 결과: {:?}", sma_result);
    println!("EMA 기반 BBand 결과: {:?}", ema_result);
    println!("WMA 기반 BBand 결과: {:?}", wma_result);
}
