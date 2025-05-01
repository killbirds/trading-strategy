use super::Strategy;
use super::StrategyType;
use super::context::{GetCandle, StrategyContextOps, StrategyDataOps};
use crate::candle_store::CandleStore;
use crate::indicator::macd::{MACD, MACDBuilder};
use crate::model::PositionType;
use crate::model::TradePosition;
use log::info;
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::fmt::Display;
use trading_chart::Candle;

/// MACD 숏 전략 설정
#[derive(Debug, Deserialize)]
pub struct MACDShortStrategyConfig {
    /// 빠른 EMA 기간
    pub fast_period: usize,
    /// 느린 EMA 기간
    pub slow_period: usize,
    /// 시그널 라인 기간
    pub signal_period: usize,
    /// 히스토그램 임계값 (0보다 작을 때 숏 진입)
    pub histogram_threshold: f64,
    /// 확인 기간 (몇 개의 연속된 캔들에서 조건을 충족해야 하는지)
    pub confirm_period: usize,
}

impl Default for MACDShortStrategyConfig {
    /// 기본 설정값 반환
    fn default() -> Self {
        MACDShortStrategyConfig {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
            histogram_threshold: 0.0,
            confirm_period: 3,
        }
    }
}

impl MACDShortStrategyConfig {
    /// 설정의 유효성을 검사합니다.
    ///
    /// 모든 설정 값이 유효한지 확인하고, 유효하지 않은 경우 오류 메시지를 반환합니다.
    ///
    /// # Returns
    /// * `Result<(), String>` - 유효성 검사 결과 (성공 또는 오류 메시지)
    pub fn validate(&self) -> Result<(), String> {
        if self.fast_period < 2 {
            return Err("빠른 EMA 기간은 2 이상이어야 합니다".to_string());
        }

        if self.slow_period <= self.fast_period {
            return Err(format!(
                "느린 EMA 기간({})은 빠른 EMA 기간({})보다 커야 합니다",
                self.slow_period, self.fast_period
            ));
        }

        if self.signal_period < 1 {
            return Err("시그널 EMA 기간은 1 이상이어야 합니다".to_string());
        }

        if self.histogram_threshold > 0.0 {
            return Err(format!(
                "숏 전략의 히스토그램 임계값({})은 0보다 작아야 합니다",
                self.histogram_threshold
            ));
        }

        if self.confirm_period < 1 {
            return Err("신호 확인 기간은 1 이상이어야 합니다".to_string());
        }

        Ok(())
    }

    /// JSON 문자열에서 설정 로드
    ///
    /// JSON 문자열로부터 설정을 로드하고, 로드에 실패할 경우 오류를 반환합니다.
    ///
    /// # Arguments
    /// * `json` - JSON 형식의 문자열
    ///
    /// # Returns
    /// * `Result<MACDShortStrategyConfig, String>` - 로드된 설정 또는 오류
    fn from_json(json: &str) -> Result<MACDShortStrategyConfig, String> {
        match serde_json::from_str::<MACDShortStrategyConfig>(json) {
            Ok(config) => {
                config.validate()?;
                Ok(config)
            }
            Err(e) => Err(format!("JSON 설정 역직렬화 실패: {}", e)),
        }
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(config: &HashMap<String, String>) -> Result<MACDShortStrategyConfig, String> {
        // 빠른 EMA 기간 설정
        let fast_period = match config.get("fast_period") {
            Some(period_str) => {
                let period = period_str
                    .parse::<usize>()
                    .map_err(|_| "빠른 EMA 기간 파싱 오류".to_string())?;

                if period < 2 {
                    return Err("빠른 EMA 기간은 2 이상이어야 합니다".to_string());
                }

                period
            }
            None => return Err("fast_period 설정이 필요합니다".to_string()),
        };

        // 느린 EMA 기간 설정
        let slow_period = match config.get("slow_period") {
            Some(period_str) => {
                let period = period_str
                    .parse::<usize>()
                    .map_err(|_| "느린 EMA 기간 파싱 오류".to_string())?;

                if period <= fast_period {
                    return Err(format!(
                        "느린 EMA 기간({})은 빠른 EMA 기간({})보다 커야 합니다",
                        period, fast_period
                    ));
                }

                period
            }
            None => return Err("slow_period 설정이 필요합니다".to_string()),
        };

        // 시그널 EMA 기간 설정
        let signal_period = match config.get("signal_period") {
            Some(period_str) => {
                let period = period_str
                    .parse::<usize>()
                    .map_err(|_| "시그널 EMA 기간 파싱 오류".to_string())?;

                if period < 1 {
                    return Err("시그널 EMA 기간은 1 이상이어야 합니다".to_string());
                }

                period
            }
            None => return Err("signal_period 설정이 필요합니다".to_string()),
        };

        // 히스토그램 임계값 설정
        let histogram_threshold = match config.get("histogram_threshold") {
            Some(threshold_str) => {
                let threshold = threshold_str
                    .parse::<f64>()
                    .map_err(|_| "히스토그램 임계값 파싱 오류".to_string())?;

                if threshold > 0.0 {
                    return Err(format!(
                        "숏 전략의 히스토그램 임계값({})은 0보다 작아야 합니다",
                        threshold
                    ));
                }

                threshold
            }
            None => return Err("histogram_threshold 설정이 필요합니다".to_string()),
        };

        // 신호 확인 기간 설정
        let confirm_period = match config.get("confirm_period") {
            Some(period_str) => {
                let period = period_str
                    .parse::<usize>()
                    .map_err(|_| "신호 확인 기간 파싱 오류".to_string())?;

                if period < 1 {
                    return Err("신호 확인 기간은 1 이상이어야 합니다".to_string());
                }

                period
            }
            None => return Err("confirm_period 설정이 필요합니다".to_string()),
        };

        let result = MACDShortStrategyConfig {
            fast_period,
            slow_period,
            signal_period,
            histogram_threshold,
            confirm_period,
        };

        result.validate()?;
        Ok(result)
    }
}

/// MACD 숏 전략 데이터
#[derive(Debug)]
struct StrategyData<C: Candle> {
    /// 현재 캔들 데이터
    candle: C,
    /// MACD 지표
    macd: MACD,
}

impl<C: Candle> StrategyData<C> {
    /// 새 전략 데이터 생성
    fn new(candle: C, macd: MACD) -> StrategyData<C> {
        StrategyData { candle, macd }
    }

    /// MACD 히스토그램이 임계값보다 작은지 확인 (하락 추세)
    fn is_histogram_below_threshold(&self, threshold: f64) -> bool {
        self.macd.histogram < threshold
    }

    /// MACD가 시그널 라인을 하향 돌파했는지 확인
    fn is_macd_below_signal(&self) -> bool {
        self.macd.macd < self.macd.signal
    }

    /// MACD가 시그널 라인을 상향 돌파했는지 확인
    fn is_macd_above_signal(&self) -> bool {
        self.macd.macd > self.macd.signal
    }
}

impl<C: Candle> GetCandle<C> for StrategyData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> StrategyDataOps<C> for StrategyData<C> {}

/// MACD 숏 전략 컨텍스트
#[derive(Debug)]
struct StrategyContext<C: Candle> {
    /// MACD 빌더
    macdbuilder: MACDBuilder<C>,
    /// 전략 데이터 히스토리 (최신 데이터가 인덱스 0)
    items: Vec<StrategyData<C>>,
}

impl<C: Candle> Display for StrategyContext<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(f, "캔들: {}, MACD: {}", first.candle, first.macd),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> StrategyContext<C> {
    /// 새 전략 컨텍스트 생성
    fn new(config: &MACDShortStrategyConfig, storage: &CandleStore<C>) -> StrategyContext<C> {
        let macdbuilder =
            MACDBuilder::new(config.fast_period, config.slow_period, config.signal_period);

        let mut ctx = StrategyContext {
            macdbuilder,
            items: vec![],
        };

        ctx.init(storage.get_reversed_items());
        ctx
    }

    /// 히스토그램이 임계값보다 작은지 확인
    fn is_histogram_below_threshold(&self, threshold: f64, n: usize) -> bool {
        self.is_all(|data| data.is_histogram_below_threshold(threshold), n)
    }

    /// MACD가 시그널 라인을 하향 돌파했는지 확인
    fn is_macd_crossed_below_signal(&self, n: usize, m: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_macd_below_signal(), n, m)
    }

    /// MACD가 시그널 라인을 상향 돌파했는지 확인
    fn is_macd_crossed_above_signal(&self, n: usize, m: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_macd_above_signal(), n, m)
    }
}

impl<C: Candle> StrategyContextOps<StrategyData<C>, C> for StrategyContext<C> {
    fn next_data(&mut self, candle: C) -> StrategyData<C> {
        let macd = self.macdbuilder.next(&candle);
        StrategyData::new(candle, macd)
    }

    fn datum(&self) -> &Vec<StrategyData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<StrategyData<C>> {
        &mut self.items
    }
}

/// MACD 기반 숏 트레이딩 전략
#[derive(Debug)]
pub struct MACDShortStrategy<C: Candle> {
    /// 전략 설정
    config: MACDShortStrategyConfig,
    /// 전략 컨텍스트 (데이터 보관 및 연산)
    ctx: StrategyContext<C>,
}

impl<C: Candle> Display for MACDShortStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[MACD숏전략] 설정: {{빠른기간: {}, 느린기간: {}, 시그널기간: {}, 임계값: {}}}, 컨텍스트: {}",
            self.config.fast_period,
            self.config.slow_period,
            self.config.signal_period,
            self.config.histogram_threshold,
            self.ctx
        )
    }
}

impl<C: Candle + 'static> MACDShortStrategy<C> {
    /// 새 MACD 숏 전략 인스턴스 생성 (JSON 설정 파일 사용)
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `json_config` - JSON 형식의 설정 문자열
    ///
    /// # Returns
    /// * `Result<MACDShortStrategy<C>, String>` - 초기화된 MACD 숏 전략 인스턴스 또는 오류
    pub fn new(
        storage: &CandleStore<C>,
        json_config: &str,
    ) -> Result<MACDShortStrategy<C>, String> {
        let config = MACDShortStrategyConfig::from_json(json_config)?;
        info!("MACD 숏 전략 설정: {:?}", config);
        let ctx = StrategyContext::new(&config, storage);

        Ok(MACDShortStrategy { config, ctx })
    }

    /// 새 MACD 숏 전략 인스턴스 생성 (설정 직접 제공)
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<MACDShortStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => MACDShortStrategyConfig::from_hash_map(&cfg)?,
            None => MACDShortStrategyConfig::default(),
        };

        info!("MACD 숏 전략 설정: {:?}", strategy_config);
        let ctx = StrategyContext::new(&strategy_config, storage);

        Ok(MACDShortStrategy {
            config: strategy_config,
            ctx,
        })
    }
}

impl<C: Candle + 'static> Strategy<C> for MACDShortStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle)
    }

    fn should_enter(&self, _candle: &C) -> bool {
        // MACD가 시그널 라인을 하향 돌파하고 히스토그램이 임계값보다 작으면 숏 진입 신호
        self.ctx
            .is_macd_crossed_below_signal(1, self.config.confirm_period)
            && self
                .ctx
                .is_histogram_below_threshold(self.config.histogram_threshold, 1)
    }

    fn should_exit(&self, _holdings: &TradePosition, _candle: &C) -> bool {
        // MACD가 시그널 라인을 상향 돌파하면 숏 청산 신호
        self.ctx
            .is_macd_crossed_above_signal(1, self.config.confirm_period)
    }

    fn get_position(&self) -> PositionType {
        PositionType::Short
    }

    fn get_name(&self) -> StrategyType {
        StrategyType::MACDShort
    }
}
