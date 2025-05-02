use super::Strategy;
use super::context::{GetCandle, StrategyDataOps};
use crate::candle_store::CandleStore;
use crate::config_loader::{ConfigError, ConfigResult, ConfigValidation};
use crate::indicator::bband::{BBand, BBandBuilder};
use log::{debug, error, info, warn};
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

/// 볼린저 밴드 전략 공통 설정 베이스
#[derive(Debug, Deserialize, Serialize)]
pub struct BBandStrategyConfigBase {
    /// 확인 캔들 수
    pub count: usize,
    /// 볼린저 밴드 계산 기간
    pub period: usize,
    /// 볼린저 밴드 승수 (표준편차 배수)
    pub multiplier: f64,
}

impl ConfigValidation for BBandStrategyConfigBase {
    fn validate(&self) -> ConfigResult<()> {
        if self.count == 0 {
            return Err(ConfigError::ValidationError(
                "확인 캔들 수는 0보다 커야 합니다".to_string(),
            ));
        }

        if self.period < 2 {
            return Err(ConfigError::ValidationError(
                "볼린저 밴드 계산 기간은 2 이상이어야 합니다".to_string(),
            ));
        }

        if self.multiplier <= 0.0 {
            return Err(ConfigError::ValidationError(
                "볼린저 밴드 승수는 0보다 커야 합니다".to_string(),
            ));
        }

        Ok(())
    }
}

impl BBandStrategyConfigBase {
    /// 설정 파일에서 로드
    pub fn from_file<T>(path: &Path) -> ConfigResult<T>
    where
        T: DeserializeOwned + ConfigValidation,
    {
        debug!("볼린저 밴드 전략 설정 파일 로드 시작: {}", path.display());
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
    /// * `Result<BBandStrategyConfigBase, String>` - 로드된 설정 또는 오류
    pub fn from_json<T>(json: &str) -> Result<T, String>
    where
        T: for<'de> Deserialize<'de>,
    {
        debug!("볼린저 밴드 전략 JSON 설정 파싱 시작");
        match serde_json::from_str::<T>(json) {
            Ok(config) => {
                debug!("볼린저 밴드 전략 JSON 설정 파싱 성공");
                Ok(config)
            }
            Err(e) => {
                let error_msg = format!("JSON 설정 역직렬화 실패: {}", e);
                error!("{}", error_msg);
                Err(error_msg)
            }
        }
    }

    /// HashMap에서 설정 로드
    pub fn from_hash_map(
        config: &HashMap<String, String>,
    ) -> Result<BBandStrategyConfigBase, String> {
        debug!("볼린저 밴드 전략 HashMap 설정 파싱 시작: {:?}", config);

        // 확인 캔들 수 설정
        let count = match config.get("count") {
            Some(count_str) => {
                debug!("count 설정 파싱: {}", count_str);
                let count = match count_str.parse::<usize>() {
                    Ok(c) => c,
                    Err(e) => {
                        let error_msg = format!("확인 캔들 수 파싱 오류: {}", e);
                        error!("{}", error_msg);
                        return Err(error_msg);
                    }
                };

                if count == 0 {
                    let error_msg = "확인 캔들 수는 0보다 커야 합니다".to_string();
                    error!("{}", error_msg);
                    return Err(error_msg);
                }

                count
            }
            None => {
                let error_msg = "count 설정이 필요합니다".to_string();
                error!("{}", error_msg);
                return Err(error_msg);
            }
        };

        // 볼린저 밴드 계산 기간 설정
        let period = match config.get("period") {
            Some(period_str) => {
                debug!("period 설정 파싱: {}", period_str);
                let period = match period_str.parse::<usize>() {
                    Ok(p) => p,
                    Err(e) => {
                        let error_msg = format!("볼린저 밴드 계산 기간 파싱 오류: {}", e);
                        error!("{}", error_msg);
                        return Err(error_msg);
                    }
                };

                if period < 2 {
                    let error_msg = "볼린저 밴드 계산 기간은 2 이상이어야 합니다".to_string();
                    error!("{}", error_msg);
                    return Err(error_msg);
                }

                period
            }
            None => {
                let error_msg = "period 설정이 필요합니다".to_string();
                error!("{}", error_msg);
                return Err(error_msg);
            }
        };

        // 볼린저 밴드 승수 설정
        let multiplier = match config.get("multiplier") {
            Some(multiplier_str) => {
                debug!("multiplier 설정 파싱: {}", multiplier_str);
                let multiplier = match multiplier_str.parse::<f64>() {
                    Ok(m) => m,
                    Err(e) => {
                        let error_msg = format!("볼린저 밴드 승수 파싱 오류: {}", e);
                        error!("{}", error_msg);
                        return Err(error_msg);
                    }
                };

                if multiplier <= 0.0 {
                    let error_msg = "볼린저 밴드 승수는 0보다 커야 합니다".to_string();
                    error!("{}", error_msg);
                    return Err(error_msg);
                }

                multiplier
            }
            None => {
                let error_msg = "multiplier 설정이 필요합니다".to_string();
                error!("{}", error_msg);
                return Err(error_msg);
            }
        };

        let result = BBandStrategyConfigBase {
            count,
            period,
            multiplier,
        };

        debug!("볼린저 밴드 전략 설정 생성 완료: {:?}", result);

        // 유효성 검사
        if let Err(e) = result.validate() {
            error!("볼린저 밴드 전략 설정 유효성 검사 실패: {}", e);
            return Err(e.to_string());
        }

        info!("볼린저 밴드 전략 설정 로드 완료: {:?}", result);
        Ok(result)
    }
}

/// 볼린저 밴드 전략 데이터
#[derive(Debug)]
pub struct BBandStrategyData<C: Candle> {
    pub candle: C,
    pub bband: BBand,
}

impl<C: Candle> BBandStrategyData<C> {
    /// 새 전략 데이터 생성
    pub fn new(candle: C, bband: BBand) -> BBandStrategyData<C> {
        BBandStrategyData { candle, bband }
    }
}

impl<C: Candle> GetCandle<C> for BBandStrategyData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> StrategyDataOps<C> for BBandStrategyData<C> {}

/// 볼린저 밴드 전략 컨텍스트
#[derive(Debug)]
pub struct BBandStrategyContext<C: Candle> {
    pub bbandbuilder: BBandBuilder<C>,
    pub items: Vec<BBandStrategyData<C>>,
}

impl<C: Candle> BBandStrategyContext<C> {
    /// 새 전략 컨텍스트 생성
    pub fn new(
        period: usize,
        multiplier: f64,
        storage: &CandleStore<C>,
    ) -> BBandStrategyContext<C> {
        debug!(
            "볼린저 밴드 전략 컨텍스트 생성 시작 (기간: {}, 승수: {})",
            period, multiplier
        );

        let bbandbuilder = BBandBuilder::new(period, multiplier);
        let mut ctx = BBandStrategyContext {
            bbandbuilder,
            items: vec![],
        };

        let items_count = storage.get_reversed_items().len();
        debug!("캔들 데이터 로드: {} 항목", items_count);

        if items_count == 0 {
            warn!("캔들 데이터가 비어 있습니다. 전략이 제대로 작동하지 않을 수 있습니다.");
        }

        for item in storage.get_reversed_items().iter().rev() {
            let bband = ctx.bbandbuilder.next(item);
            ctx.items
                .insert(0, BBandStrategyData::new(item.clone(), bband));
        }

        debug!("볼린저 밴드 지표 계산 완료: {} 캔들", ctx.items.len());

        if ctx.items.is_empty() {
            warn!("계산된 볼린저 밴드 데이터가 없습니다. 전략이 제대로 작동하지 않을 수 있습니다.");
        } else if ctx.items.len() < period {
            warn!(
                "계산된 볼린저 밴드 데이터({})가 설정된 기간({}) 미만입니다. 전략의 정확도가 떨어질 수 있습니다.",
                ctx.items.len(),
                period
            );
        }

        info!(
            "볼린저 밴드 전략 컨텍스트 생성 완료 ({} 항목)",
            ctx.items.len()
        );
        ctx
    }
}

impl<C: Candle> StrategyContextOps<BBandStrategyData<C>, C> for BBandStrategyContext<C> {
    fn next_data(&mut self, candle: C) -> BBandStrategyData<C> {
        let bband = self.bbandbuilder.next(&candle);
        BBandStrategyData::new(candle, bband)
    }

    fn datum(&self) -> &Vec<BBandStrategyData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<BBandStrategyData<C>> {
        &mut self.items
    }
}

impl<C: Candle> Display for BBandStrategyContext<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(
                f,
                "캔들: {}, 밴드: {{상: {:.2}, 중: {:.2}, 하: {:.2}}}",
                first.candle,
                first.bband.upper(),
                first.bband.average(),
                first.bband.lower()
            ),
            None => write!(f, "데이터 없음"),
        }
    }
}

/// 볼린저 밴드 전략을 위한 공통 트레이트
pub trait BBandStrategyCommon<C: Candle + 'static>: Strategy<C> {
    /// 컨텍스트 참조 반환
    fn context(&self) -> &BBandStrategyContext<C>;

    /// 밴드 폭이 충분히 넓은지 확인
    fn is_band_width_sufficient(&self) -> bool {
        self.context().is_greater_than_target(
            |data| (data.bband.upper() - data.bband.lower()) / data.bband.average(),
            0.02,
            1,
        )
    }

    /// 가격이 볼린저 밴드 하한선 아래로 내려갔는지 확인
    fn is_below_lower_band(&self) -> bool {
        if let Some(first) = self.context().items.first() {
            first.candle.close_price() < first.bband.lower()
        } else {
            false
        }
    }

    /// 가격이 볼린저 밴드 상한선 위로 올라갔는지 확인
    fn is_above_upper_band(&self) -> bool {
        if let Some(first) = self.context().items.first() {
            first.candle.close_price() > first.bband.upper()
        } else {
            false
        }
    }

    /// 가격이 볼린저 밴드 중앙선 위로 올라갔는지 확인
    fn is_above_middle_band(&self) -> bool {
        if let Some(first) = self.context().items.first() {
            first.candle.close_price() > first.bband.average()
        } else {
            false
        }
    }

    /// 가격이 볼린저 밴드 중앙선 아래로 내려갔는지 확인
    fn is_below_middle_band(&self) -> bool {
        if let Some(first) = self.context().items.first() {
            first.candle.close_price() < first.bband.average()
        } else {
            false
        }
    }
}
