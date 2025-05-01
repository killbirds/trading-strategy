use crate::strategy::ma_strategy::MAStrategy;
use crate::strategy::tests::common::{
    backtest_strategy, create_downtrend_candles, create_sideways_candles, create_test_storage,
    create_uptrend_candles,
};
use std::collections::HashMap;

// 테스트용 설정 생성 함수
fn create_ma_config() -> HashMap<String, String> {
    let mut config = HashMap::new();
    config.insert("ma".to_string(), "SMA".to_string());
    config.insert("ma_periods".to_string(), "5,10,20".to_string());
    config.insert("cross_previous_periods".to_string(), "15".to_string());
    config
}

#[test]
fn test_ma_strategy_creation() {
    // 테스트 캔들 데이터 생성
    let candles = create_uptrend_candles(50, 100.0, 1.0);
    let storage = create_test_storage(candles);

    // 설정 생성
    let config = create_ma_config();

    // 전략 인스턴스 생성
    let strategy = MAStrategy::new_with_config(&storage, Some(config)).unwrap();

    // 인스턴스가 제대로 생성되었는지 확인
    assert!(!strategy.to_string().is_empty());
}

#[test]
fn test_ma_strategy_signals_uptrend() {
    // 상승장 환경에서 테스트
    // 테스트 캔들 데이터 생성 (명확한 상승 트렌드)
    let candles = create_uptrend_candles(50, 100.0, 2.0);
    let storage = create_test_storage(candles.clone());

    // 설정 생성
    let config = create_ma_config();

    // 전략 인스턴스 생성
    let strategy = MAStrategy::new_with_config(&storage, Some(config)).unwrap();

    // 백테스팅 실행
    let result = backtest_strategy(strategy, candles, 10000.0);

    // 상승장에서는 수익이 나야함
    println!("상승장 MA 전략 결과: {:?}", result);
    println!("총 수익률: {}", result.total_profit_percentage);
    println!("승률: {}", result.win_rate);
}

#[test]
fn test_ma_strategy_signals_downtrend() {
    // 하락장 환경에서 테스트
    // 테스트 캔들 데이터 생성 (명확한 하락 트렌드)
    let candles = create_downtrend_candles(50, 150.0, 2.0);
    let storage = create_test_storage(candles.clone());

    // 설정 생성
    let config = create_ma_config();

    // 전략 인스턴스 생성
    let strategy = MAStrategy::new_with_config(&storage, Some(config)).unwrap();

    // 백테스팅 실행
    let result = backtest_strategy(strategy, candles, 10000.0);

    // 결과 출력
    println!("하락장 MA 전략 결과: {:?}", result);
}

#[test]
fn test_ma_strategy_signals_sideways() {
    // 횡보장 환경에서 테스트
    // 테스트 캔들 데이터 생성 (횡보 트렌드)
    let candles = create_sideways_candles(50, 150.0, 15.0);
    let storage = create_test_storage(candles.clone());

    // 설정 생성
    let config = create_ma_config();

    // 전략 인스턴스 생성
    let strategy = MAStrategy::new_with_config(&storage, Some(config)).unwrap();

    // 백테스팅 실행
    let result = backtest_strategy(strategy, candles, 10000.0);

    // 결과 출력
    println!("횡보장 MA 전략 결과: {:?}", result);
}

#[test]
fn test_different_ma_types() {
    // 서로 다른 MA 유형 테스트

    // 테스트 캔들 데이터 생성 (상승장 - MA 전략에 적합)
    let candles = create_uptrend_candles(50, 100.0, 2.0);

    // SMA 설정
    let mut sma_config = HashMap::new();
    sma_config.insert("ma".to_string(), "SMA".to_string());
    sma_config.insert("ma_periods".to_string(), "5,10,20".to_string());
    sma_config.insert("cross_previous_periods".to_string(), "15".to_string());

    // EMA 설정
    let mut ema_config = HashMap::new();
    ema_config.insert("ma".to_string(), "EMA".to_string());
    ema_config.insert("ma_periods".to_string(), "5,10,20".to_string());
    ema_config.insert("cross_previous_periods".to_string(), "15".to_string());

    // WMA 설정
    let mut wma_config = HashMap::new();
    wma_config.insert("ma".to_string(), "WMA".to_string());
    wma_config.insert("ma_periods".to_string(), "5,10,20".to_string());
    wma_config.insert("cross_previous_periods".to_string(), "15".to_string());

    // SMA 테스트
    let storage = create_test_storage(candles.clone());
    let strategy = MAStrategy::new_with_config(&storage, Some(sma_config)).unwrap();
    let sma_result = backtest_strategy(strategy, candles.clone(), 10000.0);

    // EMA 테스트
    let storage = create_test_storage(candles.clone());
    let strategy = MAStrategy::new_with_config(&storage, Some(ema_config)).unwrap();
    let ema_result = backtest_strategy(strategy, candles.clone(), 10000.0);

    // WMA 테스트
    let storage = create_test_storage(candles.clone());
    let strategy = MAStrategy::new_with_config(&storage, Some(wma_config)).unwrap();
    let wma_result = backtest_strategy(strategy, candles, 10000.0);

    // 결과 출력
    println!("SMA 전략 결과: {:?}", sma_result);
    println!("EMA 전략 결과: {:?}", ema_result);
    println!("WMA 전략 결과: {:?}", wma_result);
}
