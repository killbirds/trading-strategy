pub mod bband_common;
pub mod bband_short_strategy;
pub mod bband_strategy;
pub mod copys_common;
pub mod copys_short_strategy;
pub mod copys_strategy;
pub mod dummy_strategy;
pub mod hybrid_common;
pub mod hybrid_short_strategy;
pub mod hybrid_strategy;
pub mod ma_common;
pub mod ma_short_strategy;
pub mod ma_strategy;
pub mod macd_common;
pub mod macd_short_strategy;
pub mod macd_strategy;
pub mod multi_timeframe_strategy;
pub mod rsi_common;
pub mod rsi_short_strategy;
pub mod rsi_strategy;
pub mod three_rsi_common;
pub mod three_rsi_short_strategy;
pub mod three_rsi_strategy;

#[cfg(test)]
mod tests;

use crate::candle_store::CandleStore;
use crate::model::PositionType;
pub use crate::{ConfigError, ConfigResult};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;
use trading_chart::Candle;

/// 거래 전략 유형
///
/// 시스템에서 사용 가능한 다양한 거래 전략을 정의합니다.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum StrategyType {
    /// 더미 전략 (테스트 용도)
    Dummy,
    /// 이동평균선(Moving Average) 기반 롱 전략
    MA,
    /// 이동평균선(Moving Average) 기반 숏 전략
    MAShort,
    /// 상대강도지수(Relative Strength Index) 기반 롱 전략
    RSI,
    /// 상대강도지수(Relative Strength Index) 기반 숏 전략
    RSIShort,
    /// 볼린저밴드(Bollinger Band) 기반 롱 전략
    BBand,
    /// 볼린저밴드(Bollinger Band) 기반 숏 전략
    BBandShort,
    /// MACD 기반 롱 전략
    MACD,
    /// MACD 기반 숏 전략
    MACDShort,
    /// Copys 전략 (커스텀 롱 전략)
    Copys,
    /// Copys 숏 전략 (커스텀 숏 전략)
    CopysShort,
    /// 3개의 RSI 지표를 조합한 롱 전략
    ThreeRSI,
    /// 3개의 RSI 지표를 조합한 숏 전략
    ThreeRSIShort,
    /// 여러 지표를 결합한 하이브리드 전략
    Hybrid,
    /// 여러 지표를 결합한 하이브리드 숏 전략
    HybridShort,
    /// 여러 타임프레임을 분석하는 전략
    MultiTimeframe,
}

impl Display for StrategyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrategyType::Dummy => write!(f, "dummy"),
            StrategyType::MA => write!(f, "ma"),
            StrategyType::MAShort => write!(f, "ma_short"),
            StrategyType::RSI => write!(f, "rsi"),
            StrategyType::RSIShort => write!(f, "rsi_short"),
            StrategyType::BBand => write!(f, "bband"),
            StrategyType::BBandShort => write!(f, "bband_short"),
            StrategyType::MACD => write!(f, "macd"),
            StrategyType::MACDShort => write!(f, "macd_short"),
            StrategyType::Copys => write!(f, "copys"),
            StrategyType::CopysShort => write!(f, "copys_short"),
            StrategyType::ThreeRSI => write!(f, "three_rsi"),
            StrategyType::ThreeRSIShort => write!(f, "three_rsi_short"),
            StrategyType::Hybrid => write!(f, "hybrid"),
            StrategyType::HybridShort => write!(f, "hybrid_short"),
            StrategyType::MultiTimeframe => write!(f, "multi_timeframe"),
        }
    }
}

/// 거래 전략 인터페이스
///
/// 모든 거래 전략은 이 트레이트를 구현해야 합니다.
pub trait Strategy<C: Candle>: Display + Send {
    /// 새로운 캔들 데이터 업데이트
    ///
    /// # Arguments
    /// * `candle` - 새 캔들 데이터
    fn next(&mut self, candle: C);

    /// 매수 신호 확인
    ///
    /// # Arguments
    /// * `candle` - 현재 캔들 데이터
    ///
    /// # Returns
    /// * `bool` - 매수 신호 여부
    fn should_enter(&self, candle: &C) -> bool;

    /// 매도 신호 확인
    ///
    /// # Arguments
    /// * `candle` - 현재 캔들 데이터
    ///
    /// # Returns
    /// * `bool` - 매도 신호 여부
    fn should_exit(&self, candle: &C) -> bool;

    /// 전략의 포지션 타입 반환
    ///
    /// # Returns
    /// * `PositionType` - 전략의 포지션 타입 (Long 또는 Short)
    fn position(&self) -> PositionType;

    /// 전략의 타입 반환
    ///
    /// # Returns
    /// * `StrategyType` - 전략의 타입
    fn name(&self) -> StrategyType;
}

/// 전략 팩토리
///
/// 전략 유형에 따라 실제 전략 인스턴스를 생성합니다.
pub struct StrategyFactory;

impl StrategyFactory {
    /// 전략 유형과 캔들 저장소로부터 전략 인스턴스 생성
    ///
    /// # Arguments
    /// * `strategy_type` - 생성할 전략 유형
    /// * `storage` - 캔들 데이터 저장소
    /// * `config` - 선택적 설정 (HashMap 형태로 제공)
    ///
    /// # Returns
    /// * `Result<Box<dyn Strategy>, String>` - 생성된 전략 인스턴스 또는 에러
    ///
    /// # Panics
    /// * 알 수 없는 전략 유형이 지정된 경우 패닉 발생
    pub fn build<C: Candle + 'static>(
        strategy_type: StrategyType,
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<Box<dyn Strategy<C>>, String> {
        info!("전략 빌드 시작: {strategy_type}");
        debug!("캔들 데이터 상태: 항목 수={}", storage.len());

        if storage.is_empty() {
            warn!("캔들 데이터가 비어 있습니다. 전략이 제대로 작동하지 않을 수 있습니다.");
        }

        let result = match strategy_type {
            StrategyType::Dummy => {
                debug!("더미 전략 초기화 시작");
                dummy_strategy::DummyStrategy::new_with_config(storage, config)
                    .map(|s| Box::new(s) as Box<dyn Strategy<C>>)
            }
            StrategyType::MA => {
                debug!("MA 전략 초기화 시작");
                ma_strategy::MAStrategy::new_with_config(storage, config)
                    .map(|s| Box::new(s) as Box<dyn Strategy<C>>)
            }
            StrategyType::MAShort => {
                debug!("MA 숏 전략 초기화 시작");
                ma_short_strategy::MAShortStrategy::new_with_config(storage, config)
                    .map(|s| Box::new(s) as Box<dyn Strategy<C>>)
            }
            StrategyType::RSI => {
                debug!("RSI 전략 초기화 시작");
                rsi_strategy::RSIStrategy::new_with_config(storage, config)
                    .map(|s| Box::new(s) as Box<dyn Strategy<C>>)
            }
            StrategyType::RSIShort => {
                debug!("RSI 숏 전략 초기화 시작");
                rsi_short_strategy::RSIShortStrategy::new_with_config(storage, config)
                    .map(|s| Box::new(s) as Box<dyn Strategy<C>>)
            }
            StrategyType::BBand => {
                debug!("볼린저 밴드 전략 초기화 시작");
                bband_strategy::BBandStrategy::new_with_config(storage, config)
                    .map(|s| Box::new(s) as Box<dyn Strategy<C>>)
            }
            StrategyType::BBandShort => {
                debug!("볼린저 밴드 숏 전략 초기화 시작");
                bband_short_strategy::BBandShortStrategy::new_with_config(storage, config)
                    .map(|s| Box::new(s) as Box<dyn Strategy<C>>)
            }
            StrategyType::MACD => {
                debug!("MACD 전략 초기화 시작");
                macd_strategy::MACDStrategy::new_with_config(storage, config)
                    .map(|s| Box::new(s) as Box<dyn Strategy<C>>)
            }
            StrategyType::MACDShort => {
                debug!("MACD 숏 전략 초기화 시작");
                macd_short_strategy::MACDShortStrategy::new_with_config(storage, config)
                    .map(|s| Box::new(s) as Box<dyn Strategy<C>>)
            }
            StrategyType::Copys => {
                debug!("Copys 전략 초기화 시작");
                copys_strategy::CopysStrategy::new_with_config(storage, config)
                    .map(|s| Box::new(s) as Box<dyn Strategy<C>>)
            }
            StrategyType::CopysShort => {
                debug!("Copys 숏 전략 초기화 시작");
                copys_short_strategy::CopysShortStrategy::new_with_config(storage, config)
                    .map(|s| Box::new(s) as Box<dyn Strategy<C>>)
            }
            StrategyType::ThreeRSI => {
                debug!("3-RSI 전략 초기화 시작");
                three_rsi_strategy::ThreeRSIStrategy::new_with_config(storage, config)
                    .map(|s| Box::new(s) as Box<dyn Strategy<C>>)
            }
            StrategyType::ThreeRSIShort => {
                debug!("3-RSI 숏 전략 초기화 시작");
                three_rsi_short_strategy::ThreeRSIShortStrategy::new_with_config(storage, config)
                    .map(|s| Box::new(s) as Box<dyn Strategy<C>>)
            }
            StrategyType::Hybrid => {
                debug!("하이브리드 전략 초기화 시작");
                hybrid_strategy::HybridStrategy::new_with_config(storage, config)
                    .map(|s| Box::new(s) as Box<dyn Strategy<C>>)
            }
            StrategyType::HybridShort => {
                debug!("하이브리드 숏 전략 초기화 시작");
                hybrid_short_strategy::HybridShortStrategy::new_with_config(storage, config)
                    .map(|s| Box::new(s) as Box<dyn Strategy<C>>)
            }
            StrategyType::MultiTimeframe => {
                debug!("MultiTimeframe 전략 초기화 시작");
                multi_timeframe_strategy::MultiTimeframeStrategy::new_with_config(storage, config)
                    .map(|s| Box::new(s) as Box<dyn Strategy<C>>)
            }
        };

        match &result {
            Ok(_) => {
                info!("전략 빌드 성공: {strategy_type}");
            }
            Err(e) => {
                error!("전략 빌드 실패: {strategy_type} - {e}");
            }
        }

        result
    }

    /// 기본 설정으로 전략 인스턴스 생성 (이전 버전과의 호환성 유지)
    ///
    /// # Arguments
    /// * `strategy_type` - 생성할 전략 유형
    /// * `storage` - 캔들 데이터 저장소
    ///
    /// # Returns
    /// * `Result<Box<dyn Strategy>, String>` - 생성된 전략 인스턴스 또는 에러
    pub fn build_with_default<C: Candle + 'static>(
        strategy_type: StrategyType,
        storage: &CandleStore<C>,
    ) -> Result<Box<dyn Strategy<C>>, String> {
        Self::build(strategy_type, storage, None)
    }

    /// 전략 유형에서 포지션 타입 반환
    ///
    /// # Arguments
    /// * `strategy_type` - 포지션 타입을 확인할 전략 유형
    ///
    /// # Returns
    /// * `PositionType` - 해당 전략의 포지션 타입 (Long 또는 Short)
    pub fn position_from_strategy_type(strategy_type: StrategyType) -> PositionType {
        match strategy_type {
            StrategyType::Dummy => PositionType::Long,
            StrategyType::MA => PositionType::Long,
            StrategyType::MAShort => PositionType::Short,
            StrategyType::RSI => PositionType::Long,
            StrategyType::RSIShort => PositionType::Short,
            StrategyType::BBand => PositionType::Long,
            StrategyType::BBandShort => PositionType::Short,
            StrategyType::MACD => PositionType::Long,
            StrategyType::MACDShort => PositionType::Short,
            StrategyType::Copys => PositionType::Long,
            StrategyType::CopysShort => PositionType::Short,
            StrategyType::ThreeRSI => PositionType::Long,
            StrategyType::ThreeRSIShort => PositionType::Short,
            StrategyType::Hybrid => PositionType::Long,
            StrategyType::HybridShort => PositionType::Short,
            StrategyType::MultiTimeframe => PositionType::Long,
        }
    }

    /// 기본 설정 파일 경로 생성
    ///
    /// # Arguments
    /// * `strategy_type` - 전략 유형
    ///
    /// # Returns
    /// * `std::path::PathBuf` - 설정 파일 경로
    pub fn default_config_path(strategy_type: StrategyType) -> std::path::PathBuf {
        let filename = format!("{strategy_type}.toml");
        std::path::PathBuf::from("config").join(filename)
    }
}

/// 문자열을 분리하여 벡터로 변환
///
/// # Arguments
/// * `input` - 분리할 문자열
///
/// # Returns
/// * `Result<Vec<T>, String>` - 분리된 값 벡터 또는 에러
pub fn split<T: FromStr>(input: &str) -> Result<Vec<T>, String>
where
    <T as FromStr>::Err: Debug + Display,
{
    if input.is_empty() {
        return Ok(vec![]);
    }

    input
        .split(',')
        .map(|s| {
            let trimmed = s.trim();
            trimmed
                .parse::<T>()
                .map_err(|e| format!("파싱 오류: {e}"))
        })
        .collect()
}

/// 문자열을 안전하게 분리하여 벡터로 변환 (에러 시 빈 벡터 반환)
///
/// # Arguments
/// * `input` - 분리할 문자열
///
/// # Returns
/// * `Result<Vec<T>, String>` - 분리된 값 벡터 또는 에러
pub fn split_safe<T: FromStr>(input: &str) -> Result<Vec<T>, String>
where
    <T as FromStr>::Err: Debug + Display,
{
    match split::<T>(input) {
        Ok(v) => Ok(v),
        Err(e) => {
            log::error!("분리 오류: {e}");
            Ok(vec![])
        }
    }
}
