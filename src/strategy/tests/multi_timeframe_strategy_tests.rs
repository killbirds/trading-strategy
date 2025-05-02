use crate::strategy::multi_timeframe_strategy::MultiTimeframeStrategy;
use crate::strategy::tests::common::{
    backtest_strategy, create_downtrend_candles, create_sideways_candles, create_test_storage,
    create_uptrend_candles,
};
use std::collections::HashMap;

// 테스트용 설정 생성 함수
fn create_multi_timeframe_config() -> HashMap<String, String> {
    let mut config = HashMap::new();

    // 기본 전략 설정 (RSI 전략으로 설정)
    config.insert("base_strategy".to_string(), "rsi".to_string());

    // RSI 전략 관련 설정
    config.insert("period".to_string(), "14".to_string());
    config.insert("oversold".to_string(), "30".to_string());
    config.insert("overbought".to_string(), "70".to_string());
    config.insert("rsi_count".to_string(), "1".to_string());
    config.insert("rsi_lower".to_string(), "30".to_string());
    config.insert("rsi_middle".to_string(), "50".to_string());
    config.insert("rsi_upper".to_string(), "70".to_string());
    config.insert("rsi_period".to_string(), "14".to_string());

    // MA 관련 설정
    config.insert("ma".to_string(), "sma".to_string());
    config.insert("ma_periods".to_string(), "5,10,20".to_string());
    config.insert("ma_upper".to_string(), "20".to_string());
    config.insert("ma_lower".to_string(), "5".to_string());
    config.insert("ma_medium".to_string(), "10".to_string());

    // 멀티 타임프레임 관련 설정
    config.insert("timeframes".to_string(), "1m,5m,15m,1h".to_string());
    config.insert("weights".to_string(), "0.2,0.3,0.3,0.2".to_string());
    config.insert("confirmation_threshold".to_string(), "0.6".to_string());

    config
}

#[test]
fn test_multi_timeframe_strategy_creation() {
    // 테스트 캔들 데이터 생성
    let candles = create_uptrend_candles(50, 100.0, 1.0);
    let storage = create_test_storage(candles);

    // 설정 생성
    let config = create_multi_timeframe_config();

    // 전략 인스턴스 생성
    let strategy = MultiTimeframeStrategy::new_with_config(&storage, Some(config));

    // 인스턴스가 올바르게 생성되었는지 확인
    assert!(strategy.is_ok(), "멀티 타임프레임 전략 생성 실패");
}

#[test]
fn test_multi_timeframe_strategy_signals_uptrend() {
    // 상승장 환경에서 테스트
    // 테스트 캔들 데이터 생성 (명확한 상승 트렌드)
    let candles = create_uptrend_candles(100, 100.0, 2.0);
    let storage = create_test_storage(candles.clone());

    // 설정 생성
    let config = create_multi_timeframe_config();

    // 전략 인스턴스 생성
    let strategy = MultiTimeframeStrategy::new_with_config(&storage, Some(config)).unwrap();

    // 백테스팅 실행
    let result = backtest_strategy(strategy, candles, 10000.0);

    // 상승장에서는 수익이 나야함
    println!("상승장 멀티 타임프레임 전략 결과: {:?}", result);
    println!("총 수익률: {}", result.total_profit_percentage);
    println!("승률: {}", result.win_rate);
}

#[test]
fn test_multi_timeframe_strategy_signals_downtrend() {
    // 하락장 환경에서 테스트
    // 테스트 캔들 데이터 생성 (명확한 하락 트렌드)
    let candles = create_downtrend_candles(100, 200.0, 2.0);
    let storage = create_test_storage(candles.clone());

    // 설정 생성
    let config = create_multi_timeframe_config();

    // 전략 인스턴스 생성
    let strategy = MultiTimeframeStrategy::new_with_config(&storage, Some(config)).unwrap();

    // 백테스팅 실행
    let result = backtest_strategy(strategy, candles, 10000.0);

    // 결과 출력
    println!("하락장 멀티 타임프레임 전략 결과: {:?}", result);
    println!("총 수익률: {}", result.total_profit_percentage);
    println!("승률: {}", result.win_rate);
}

#[test]
fn test_multi_timeframe_strategy_signals_sideways() {
    // 횡보장 환경에서 테스트
    // 테스트 캔들 데이터 생성 (횡보 트렌드)
    let candles = create_sideways_candles(100, 150.0, 15.0);
    let storage = create_test_storage(candles.clone());

    // 설정 생성
    let config = create_multi_timeframe_config();

    // 전략 인스턴스 생성
    let strategy = MultiTimeframeStrategy::new_with_config(&storage, Some(config)).unwrap();

    // 백테스팅 실행
    let result = backtest_strategy(strategy, candles, 10000.0);

    // 결과 출력
    println!("횡보장 멀티 타임프레임 전략 결과: {:?}", result);
    println!("총 수익률: {}", result.total_profit_percentage);
    println!("승률: {}", result.win_rate);
}

#[test]
fn test_multi_timeframe_strategy_with_different_weights() {
    // 다양한 가중치 설정으로 테스트
    // 테스트 캔들 데이터 생성
    let candles = create_uptrend_candles(100, 100.0, 2.0);
    let storage = create_test_storage(candles.clone());

    // 다양한 가중치로 설정 생성
    let mut config = create_multi_timeframe_config();

    // 단기 타임프레임에 더 높은 가중치 부여
    config.insert("weights".to_string(), "0.4,0.3,0.2,0.1".to_string());

    // 전략 인스턴스 생성
    let strategy = MultiTimeframeStrategy::new_with_config(&storage, Some(config)).unwrap();

    // 백테스팅 실행
    let result = backtest_strategy(strategy, candles, 10000.0);

    // 결과 출력
    println!("단기 타임프레임 중시 결과: {:?}", result);
    println!("총 수익률: {}", result.total_profit_percentage);
    println!("승률: {}", result.win_rate);
}
