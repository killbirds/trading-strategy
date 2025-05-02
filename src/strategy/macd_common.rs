use super::Strategy;
use super::context::{GetCandle, StrategyDataOps};
use crate::candle_store::CandleStore;
use crate::config_loader::{ConfigError, ConfigResult, ConfigValidation};
use crate::indicator::macd::{MACD, MACDBuilder};
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json;
use std::collections::HashMap;
use std::fmt::Display;
use std::path::Path;
use trading_chart::Candle;

// context에서 StrategyContextOps를 공개 가져오기
pub use super::context::StrategyContextOps;

/// MACD 전략 공통 설정
#[derive(Debug, Deserialize, Serialize)]
pub struct MACDStrategyConfigBase {
    /// 빠른 EMA 기간
    pub fast_period: usize,
    /// 느린 EMA 기간
    pub slow_period: usize,
    /// 시그널 라인 기간
    pub signal_period: usize,
    /// 히스토그램 임계값
    pub histogram_threshold: f64,
    /// 확인 기간 (몇 개의 연속된 캔들에서 조건을 충족해야 하는지)
    pub confirm_period: usize,
}

impl ConfigValidation for MACDStrategyConfigBase {
    fn validate(&self) -> ConfigResult<()> {
        if self.fast_period < 2 {
            return Err(ConfigError::ValidationError(
                "빠른 EMA 기간은 2 이상이어야 합니다".to_string(),
            ));
        }

        if self.slow_period <= self.fast_period {
            return Err(ConfigError::ValidationError(format!(
                "느린 EMA 기간({})은 빠른 EMA 기간({})보다 커야 합니다",
                self.slow_period, self.fast_period
            )));
        }

        if self.signal_period < 1 {
            return Err(ConfigError::ValidationError(
                "시그널 EMA 기간은 1 이상이어야 합니다".to_string(),
            ));
        }

        if self.confirm_period < 1 {
            return Err(ConfigError::ValidationError(
                "신호 확인 기간은 1 이상이어야 합니다".to_string(),
            ));
        }

        Ok(())
    }
}

impl MACDStrategyConfigBase {
    /// 설정 파일에서 로드
    pub fn from_file<T>(path: &Path) -> ConfigResult<T>
    where
        T: DeserializeOwned + ConfigValidation,
    {
        crate::config_loader::ConfigLoader::load_from_file(
            path,
            crate::config_loader::ConfigFormat::Auto,
        )
    }

    /// JSON 문자열에서 설정 로드
    ///
    /// JSON 문자열로부터 설정을 로드하고, 로드에 실패할 경우 오류를 반환합니다.
    ///
    /// # Arguments
    /// * `json` - JSON 형식의 문자열
    ///
    /// # Returns
    /// * `Result<T, String>` - 로드된 설정 또는 오류
    pub fn from_json<T>(json: &str, _is_long_strategy: bool) -> Result<T, String>
    where
        T: for<'de> Deserialize<'de>,
    {
        match serde_json::from_str::<T>(json) {
            Ok(config) => Ok(config),
            Err(e) => Err(format!("JSON 설정 역직렬화 실패: {}", e)),
        }
    }

    /// HashMap에서 설정 로드
    pub fn from_hash_map(
        config: &HashMap<String, String>,
        is_long_strategy: bool,
    ) -> Result<MACDStrategyConfigBase, String> {
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

                // 롱 전략인 경우 임계값 검증
                if is_long_strategy && threshold < 0.0 {
                    return Err(format!(
                        "롱 전략의 히스토그램 임계값({})은 0 이상이어야 합니다",
                        threshold
                    ));
                }

                // 숏 전략인 경우 임계값 검증
                if !is_long_strategy && threshold > 0.0 {
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

        let result = MACDStrategyConfigBase {
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

/// MACD 전략 데이터
#[derive(Debug)]
pub struct MACDStrategyData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// MACD 지표
    pub macd: MACD,
}

impl<C: Candle> MACDStrategyData<C> {
    /// 새 전략 데이터 생성
    pub fn new(candle: C, macd: MACD) -> MACDStrategyData<C> {
        MACDStrategyData { candle, macd }
    }

    /// MACD 히스토그램이 임계값보다 큰지 확인 (상승 추세)
    pub fn is_histogram_above_threshold(&self, threshold: f64) -> bool {
        self.macd.histogram > threshold
    }

    /// MACD 히스토그램이 임계값보다 작은지 확인 (하락 추세)
    pub fn is_histogram_below_threshold(&self, threshold: f64) -> bool {
        self.macd.histogram < threshold
    }

    /// MACD가 시그널 라인을 상향 돌파했는지 확인
    pub fn is_macd_above_signal(&self) -> bool {
        self.macd.macd > self.macd.signal
    }

    /// MACD가 시그널 라인을 하향 돌파했는지 확인
    pub fn is_macd_below_signal(&self) -> bool {
        self.macd.macd < self.macd.signal
    }
}

impl<C: Candle> GetCandle<C> for MACDStrategyData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> StrategyDataOps<C> for MACDStrategyData<C> {}

/// MACD 전략 컨텍스트
#[derive(Debug)]
pub struct MACDStrategyContext<C: Candle> {
    /// MACD 빌더
    pub macdbuilder: MACDBuilder<C>,
    /// 전략 데이터 히스토리 (최신 데이터가 인덱스 0)
    pub items: Vec<MACDStrategyData<C>>,
}

impl<C: Candle> Display for MACDStrategyContext<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(f, "캔들: {}, MACD: {}", first.candle, first.macd),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> MACDStrategyContext<C> {
    /// 새 전략 컨텍스트 생성
    pub fn new(
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
        storage: &CandleStore<C>,
    ) -> MACDStrategyContext<C> {
        let macdbuilder = MACDBuilder::new(fast_period, slow_period, signal_period);

        let mut ctx = MACDStrategyContext {
            macdbuilder,
            items: vec![],
        };

        ctx.init(storage.get_reversed_items());
        ctx
    }

    /// 히스토그램이 임계값보다 큰지 확인
    pub fn is_histogram_above_threshold(&self, threshold: f64, n: usize) -> bool {
        self.is_all(|data| data.is_histogram_above_threshold(threshold), n)
    }

    /// 히스토그램이 임계값보다 작은지 확인
    pub fn is_histogram_below_threshold(&self, threshold: f64, n: usize) -> bool {
        self.is_all(|data| data.is_histogram_below_threshold(threshold), n)
    }

    /// MACD가 시그널 라인을 상향 돌파했는지 확인
    pub fn is_macd_crossed_above_signal(&self, n: usize, m: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_macd_above_signal(), n, m)
    }

    /// MACD가 시그널 라인을 하향 돌파했는지 확인
    pub fn is_macd_crossed_below_signal(&self, n: usize, m: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_macd_below_signal(), n, m)
    }
}

impl<C: Candle> StrategyContextOps<MACDStrategyData<C>, C> for MACDStrategyContext<C> {
    fn next_data(&mut self, candle: C) -> MACDStrategyData<C> {
        let macd = self.macdbuilder.next(&candle);
        MACDStrategyData::new(candle, macd)
    }

    fn datum(&self) -> &Vec<MACDStrategyData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<MACDStrategyData<C>> {
        &mut self.items
    }
}

/// MACD 전략을 위한 공통 트레이트
pub trait MACDStrategyCommon<C: Candle + 'static>: Strategy<C> {
    /// 컨텍스트 참조 반환
    fn context(&self) -> &MACDStrategyContext<C>;

    /// 설정의 confirm_period 반환
    fn config_confirm_period(&self) -> usize;

    /// 설정의 histogram_threshold 반환
    fn config_histogram_threshold(&self) -> f64;
}
