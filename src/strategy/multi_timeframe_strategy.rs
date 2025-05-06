use crate::candle_store::CandleStore;
use crate::model::{PositionType, Signal, TradePosition};
use crate::strategy::{Strategy, StrategyFactory, StrategyType, split};
use chrono::Utc;
use std::collections::HashMap;
use trading_chart::{Candle, CandleInterval};

/// 멀티 타임프레임 분석 전략
///
/// 여러 타임프레임의 데이터를 동시에 분석하여 매매 신호를 생성합니다.
pub struct MultiTimeframeStrategy<C: Candle + 'static> {
    storage: CandleStore<C>,
    config: HashMap<String, String>,
    timeframe_weights: HashMap<CandleInterval, f64>,
    base_strategy: StrategyType,
    confirmation_threshold: f64,
    strategies: HashMap<CandleInterval, Box<dyn Strategy<C>>>,
    position_type: PositionType,
    signals: HashMap<CandleInterval, Signal>,
}

impl<C: Candle + 'static> MultiTimeframeStrategy<C> {
    /// 설정과 함께 새로운 멀티 타임프레임 전략 인스턴스를 생성합니다.
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `config` - 전략 설정
    ///
    /// # Returns
    /// * `Result<MultiTimeframeStrategy<C>, String>` - 생성된 전략 인스턴스 또는 에러
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<MultiTimeframeStrategy<C>, String> {
        let config = config.unwrap_or_default();

        // 타임프레임 목록과 가중치 파싱
        let timeframes_str = config
            .get("timeframes")
            .ok_or("timeframes 설정이 필요합니다")?;
        let weights_str = config.get("weights").ok_or("weights 설정이 필요합니다")?;

        // 타임프레임을 문자열로 파싱
        let timeframe_strings: Vec<String> = split(timeframes_str)?;

        // 가중치 파싱
        let weights: Vec<f64> = split(weights_str)?;

        if timeframe_strings.len() != weights.len() {
            return Err("타임프레임과 가중치의 개수가 일치하지 않습니다".to_string());
        }

        // 가중치 합계가 1.0인지 확인
        let weight_sum: f64 = weights.iter().sum();
        if (weight_sum - 1.0).abs() > 0.0001 {
            return Err("가중치의 합이 1.0이어야 합니다".to_string());
        }

        // HashMap으로 타임프레임과 가중치 매핑
        let mut timeframe_weights = HashMap::new();
        let mut signals = HashMap::new();
        let mut strategies = HashMap::new();

        // 타임프레임 문자열을 CandleInterval로 변환하고 해시맵에 삽입
        for (i, tf_str) in timeframe_strings.iter().enumerate() {
            let interval = match tf_str.as_str() {
                "1m" => CandleInterval::Minute1,
                "3m" => CandleInterval::Minute3,
                "5m" => CandleInterval::Minute5,
                "15m" => CandleInterval::Minute15,
                "30m" => CandleInterval::Minute30,
                "1h" => CandleInterval::Hour1,
                "2h" => CandleInterval::Hour2,
                "4h" => CandleInterval::Hour4,
                "6h" => CandleInterval::Hour6,
                "8h" => CandleInterval::Hour8,
                "12h" => CandleInterval::Hour12,
                "1d" => CandleInterval::Day1,
                "3d" => CandleInterval::Day3,
                "1w" => CandleInterval::Week1,
                "1M" => CandleInterval::Month1,
                _ => return Err(format!("지원되지 않는 타임프레임: {}", tf_str)),
            };

            timeframe_weights.insert(interval, weights[i]);
            signals.insert(interval, Signal::Hold);
        }

        // 기본 전략 타입 파싱
        let base_strategy_str = config
            .get("base_strategy")
            .ok_or("base_strategy 설정이 필요합니다")?;
        let base_strategy = match base_strategy_str.as_str() {
            "ma" => StrategyType::MA,
            "ma_short" => StrategyType::MAShort,
            "rsi" => StrategyType::RSI,
            "rsi_short" => StrategyType::RSIShort,
            "macd" => StrategyType::MACD,
            "macd_short" => StrategyType::MACDShort,
            "bband" => StrategyType::BBand,
            "bband_short" => StrategyType::BBandShort,
            "three_rsi" => StrategyType::ThreeRSI,
            "three_rsi_short" => StrategyType::ThreeRSIShort,
            _ => {
                return Err(format!(
                    "지원되지 않는 기본 전략 타입: {}",
                    base_strategy_str
                ));
            }
        };

        // 신호 확인 임계값 파싱
        let confirmation_threshold = config
            .get("confirmation_threshold")
            .map(|s| s.parse::<f64>())
            .transpose()
            .map_err(|e| format!("confirmation_threshold 파싱 오류: {}", e))?
            .unwrap_or(0.6);

        // 포지션 타입 결정 (기본 전략의 포지션 타입을 따름)
        let position_type = StrategyFactory::position_from_strategy_type(base_strategy);

        // 각 타임프레임별 전략 인스턴스 생성
        for interval in timeframe_weights.keys() {
            // 기본 전략 인스턴스를 각 타임프레임마다 생성
            let strategy = StrategyFactory::build(base_strategy, storage, Some(config.clone()))?;
            strategies.insert(*interval, strategy);
        }

        Ok(MultiTimeframeStrategy {
            storage: CandleStore::new(Vec::new(), 1000, false),
            config,
            timeframe_weights,
            base_strategy,
            confirmation_threshold,
            strategies,
            position_type,
            signals,
        })
    }

    /// 각 타임프레임별 신호를 업데이트합니다.
    ///
    /// # Arguments
    /// * `candle` - 최신 캔들 데이터
    fn update_signals(&mut self, candle: &C) {
        // 먼저 현재 포지션 정보를 얻어옴
        let current_position = self.get_current_position(candle);

        // 각 타임프레임별로 신호 업데이트
        for (interval, strategy) in &mut self.strategies {
            // 각 타임프레임에 맞게 신호 생성
            let signal = if strategy.should_enter(candle) {
                Signal::Enter
            } else if let Some(position) = &current_position {
                if strategy.should_exit(position, candle) {
                    Signal::Exit
                } else {
                    Signal::Hold
                }
            } else {
                Signal::Hold
            };

            // 신호 저장
            self.signals.insert(*interval, signal);
        }
    }

    /// 현재 포지션을 생성합니다.
    ///
    /// # Arguments
    /// * `candle` - 현재 캔들 데이터
    ///
    /// # Returns
    /// * `Option<TradePosition>` - 현재 포지션
    fn get_current_position(&self, candle: &C) -> Option<TradePosition> {
        Some(TradePosition {
            datetime: Utc::now(),
            price: candle.close_price(),
            quantity: 1.0,
            market: "default".to_string(),
        })
    }

    /// 가중 평균 신호를 계산합니다.
    ///
    /// # Returns
    /// * `f64` - 가중 평균 신호 점수 (1.0에 가까울수록 매수, -1.0에 가까울수록 매도)
    fn calculate_weighted_signal(&self) -> f64 {
        if self.signals.is_empty() {
            return 0.0;
        }

        let mut weighted_sum = 0.0;

        for (interval, signal) in &self.signals {
            let signal_value = match signal {
                Signal::Enter => 1.0,
                Signal::Exit => -1.0,
                Signal::Hold => 0.0,
            };

            if let Some(weight) = self.timeframe_weights.get(interval) {
                weighted_sum += signal_value * weight;
            }
        }

        weighted_sum
    }

    /// 설정 파일로부터 전략 인스턴스를 생성합니다.
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `config_path` - 설정 파일 경로
    ///
    /// # Returns
    /// * `Result<MultiTimeframeStrategy<C>, String>` - 생성된 전략 인스턴스 또는 에러
    pub fn from_config_file(
        storage: &CandleStore<C>,
        config_path: &std::path::Path,
    ) -> Result<MultiTimeframeStrategy<C>, String> {
        // TOML 설정 파일 읽기
        let config_str = std::fs::read_to_string(config_path)
            .map_err(|e| format!("설정 파일 읽기 오류: {}", e))?;

        // TOML 파싱
        let config: HashMap<String, String> =
            toml::from_str(&config_str).map_err(|e| format!("TOML 파싱 오류: {}", e))?;

        // 설정으로 인스턴스 생성
        MultiTimeframeStrategy::new_with_config(storage, Some(config))
    }
}

impl<C: Candle + 'static> Strategy<C> for MultiTimeframeStrategy<C> {
    fn next(&mut self, candle: C) {
        // 저장소에 캔들 추가
        self.storage.add(candle.clone());

        // 각 전략에 캔들 데이터 전달
        for strategy in self.strategies.values_mut() {
            strategy.next(candle.clone());
        }

        // 신호 업데이트
        self.update_signals(&candle);
    }

    fn should_enter(&self, _candle: &C) -> bool {
        // 가중 평균 신호가 임계값보다 크면 매수
        self.calculate_weighted_signal() >= self.confirmation_threshold
    }

    fn should_exit(&self, _holdings: &TradePosition, _candle: &C) -> bool {
        // 가중 평균 신호가 임계값보다 작으면 매도
        if self.position() == PositionType::Long {
            self.calculate_weighted_signal() <= -self.confirmation_threshold
        } else {
            self.calculate_weighted_signal() >= self.confirmation_threshold
        }
    }

    fn position(&self) -> PositionType {
        self.position_type
    }

    fn name(&self) -> StrategyType {
        StrategyType::MultiTimeframe
    }
}

impl<C: Candle + 'static> std::fmt::Display for MultiTimeframeStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "멀티 타임프레임 전략 (기본: {})", self.base_strategy)
    }
}
