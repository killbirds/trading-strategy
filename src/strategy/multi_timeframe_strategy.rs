use crate::candle_store::CandleStore;
use crate::model::{PositionType, Signal};
use crate::strategy::{Strategy, StrategyFactory, StrategyType, split};
use std::collections::HashMap;
use std::str::FromStr;
use trading_chart::{Candle, CandleInterval};

/// 멀티 타임프레임 분석 전략
///
/// 여러 타임프레임의 데이터를 동시에 분석하여 매매 신호를 생성합니다.
/// 각 타임프레임별로 별도의 캔들 저장소를 유지하여 타임프레임별 필터링을 수행합니다.
pub struct MultiTimeframeStrategy<C: Candle + 'static> {
    /// 전체 캔들 저장소 (모든 타임프레임 통합)
    storage: CandleStore<C>,
    /// 타임프레임별 캔들 저장소
    timeframe_storages: HashMap<CandleInterval, CandleStore<C>>,
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
            let interval = CandleInterval::from_str(tf_str)?;

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
                return Err(format!("지원되지 않는 기본 전략 타입: {base_strategy_str}"));
            }
        };

        // 신호 확인 임계값 파싱
        let confirmation_threshold = config
            .get("confirmation_threshold")
            .map(|s| s.parse::<f64>())
            .transpose()
            .map_err(|e| format!("confirmation_threshold 파싱 오류: {e}"))?
            .unwrap_or(0.6);

        // 포지션 타입 결정 (기본 전략의 포지션 타입을 따름)
        let position_type = StrategyFactory::position_from_strategy_type(base_strategy);

        // 타임프레임별 캔들 저장소 초기화
        let mut timeframe_storages = HashMap::new();
        for interval in timeframe_weights.keys() {
            // 각 타임프레임별로 해당 타임프레임의 캔들만 필터링하여 저장소 생성
            let filtered_candles: Vec<C> = storage
                .items()
                .iter()
                .filter(|candle| candle.interval() == interval)
                .cloned()
                .collect();

            let timeframe_storage = CandleStore::new(
                filtered_candles,
                storage.max_size,
                storage.use_duplicated_filter,
            );
            timeframe_storages.insert(*interval, timeframe_storage);
        }

        // 각 타임프레임별 전략 인스턴스 생성
        for interval in timeframe_weights.keys() {
            // 각 타임프레임별 저장소를 사용하여 전략 생성
            let timeframe_storage = timeframe_storages.get(interval).ok_or_else(|| {
                format!("타임프레임 {:?}에 대한 저장소를 찾을 수 없습니다", interval)
            })?;
            let strategy =
                StrategyFactory::build(base_strategy, timeframe_storage, Some(config.clone()))?;
            strategies.insert(*interval, strategy);
        }

        Ok(MultiTimeframeStrategy {
            storage: CandleStore::new(
                storage.items().to_vec(),
                storage.max_size,
                storage.use_duplicated_filter,
            ),
            timeframe_storages,
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
        // 각 타임프레임별로 신호 업데이트
        for (interval, strategy) in &mut self.strategies {
            // 해당 타임프레임의 캔들인지 확인
            if candle.interval() != interval {
                // 다른 타임프레임의 캔들은 해당 전략에 전달하지 않음
                continue;
            }

            // 타임프레임별 저장소에 캔들 추가
            if let Some(timeframe_storage) = self.timeframe_storages.get_mut(interval) {
                timeframe_storage.add(candle.clone());
            }

            // 각 타임프레임에 맞게 신호 생성
            let signal = if strategy.should_enter(candle) {
                Signal::Enter
            } else if strategy.should_exit(candle) {
                Signal::Exit
            } else {
                Signal::Hold
            };

            // 신호 저장
            self.signals.insert(*interval, signal);
        }
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
            .map_err(|e| format!("설정 파일 읽기 오류: {e}"))?;

        // TOML 파싱
        let config: HashMap<String, String> =
            toml::from_str(&config_str).map_err(|e| format!("TOML 파싱 오류: {e}"))?;

        // 설정으로 인스턴스 생성
        MultiTimeframeStrategy::new_with_config(storage, Some(config))
    }
}

impl<C: Candle + 'static> Strategy<C> for MultiTimeframeStrategy<C> {
    fn next(&mut self, candle: C) {
        let candle_interval = candle.interval();

        // 전체 저장소에 캔들 추가
        self.storage.add(candle.clone());

        // 각 타임프레임별 저장소에 해당 타임프레임의 캔들만 추가
        if let Some(timeframe_storage) = self.timeframe_storages.get_mut(candle_interval) {
            timeframe_storage.add(candle.clone());
        }

        // 해당 타임프레임의 전략에만 캔들 데이터 전달
        if let Some(strategy) = self.strategies.get_mut(candle_interval) {
            strategy.next(candle.clone());
        }

        // 신호 업데이트 (참조 사용으로 클론 최소화)
        self.update_signals(&candle);
    }

    fn should_enter(&self, _candle: &C) -> bool {
        // 가중 평균 신호가 임계값보다 크면 매수
        self.calculate_weighted_signal() >= self.confirmation_threshold
    }

    fn should_exit(&self, _candle: &C) -> bool {
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
