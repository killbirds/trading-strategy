use super::Strategy;
use super::StrategyType;
use super::context::{GetCandle, StrategyContextOps, StrategyDataOps};
use crate::candle_store::CandleStore;
use crate::indicator::bband::{BBand, BBandBuilder};
use crate::model::PositionType;
use crate::model::TradePosition;
use log::info;
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::fmt::Display;
use trading_chart::Candle;

/// 볼린저 밴드 전략 설정
#[derive(Debug, Deserialize)]
pub struct BBandStrategyConfig {
    /// 확인 캔들 수
    pub count: usize,
    /// 볼린저 밴드 계산 기간
    pub period: usize,
    /// 볼린저 밴드 승수 (표준편차 배수)
    pub multiplier: f64,
}

impl Default for BBandStrategyConfig {
    /// 기본 설정값 반환
    fn default() -> Self {
        BBandStrategyConfig {
            count: 2,
            period: 20,
            multiplier: 2.0,
        }
    }
}

impl BBandStrategyConfig {
    /// 설정의 유효성을 검사합니다.
    ///
    /// 모든 설정 값이 유효한지 확인하고, 유효하지 않은 경우 오류 메시지를 반환합니다.
    ///
    /// # Returns
    /// * `Result<(), String>` - 유효성 검사 결과 (성공 또는 오류 메시지)
    pub fn validate(&self) -> Result<(), String> {
        if self.count == 0 {
            return Err("확인 캔들 수는 0보다 커야 합니다".to_string());
        }

        if self.period < 2 {
            return Err("볼린저 밴드 계산 기간은 2 이상이어야 합니다".to_string());
        }

        if self.multiplier <= 0.0 {
            return Err("볼린저 밴드 승수는 0보다 커야 합니다".to_string());
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
    /// * `Result<BBandStrategyConfig, String>` - 로드된 설정 또는 오류
    fn from_json(json: &str) -> Result<BBandStrategyConfig, String> {
        match serde_json::from_str::<BBandStrategyConfig>(json) {
            Ok(config) => {
                config.validate()?;
                Ok(config)
            }
            Err(e) => Err(format!("JSON 설정 역직렬화 실패: {}", e)),
        }
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(config: &HashMap<String, String>) -> Result<BBandStrategyConfig, String> {
        // 확인 캔들 수 설정
        let count = match config.get("count") {
            Some(count_str) => {
                let count = count_str
                    .parse::<usize>()
                    .map_err(|_| "확인 캔들 수 파싱 오류".to_string())?;

                if count == 0 {
                    return Err("확인 캔들 수는 0보다 커야 합니다".to_string());
                }

                count
            }
            None => return Err("count 설정이 필요합니다".to_string()),
        };

        // 볼린저 밴드 계산 기간 설정
        let period = match config.get("period") {
            Some(period_str) => {
                let period = period_str
                    .parse::<usize>()
                    .map_err(|_| "볼린저 밴드 계산 기간 파싱 오류".to_string())?;

                if period < 2 {
                    return Err("볼린저 밴드 계산 기간은 2 이상이어야 합니다".to_string());
                }

                period
            }
            None => return Err("period 설정이 필요합니다".to_string()),
        };

        // 볼린저 밴드 승수 설정
        let multiplier = match config.get("multiplier") {
            Some(multiplier_str) => {
                let multiplier = multiplier_str
                    .parse::<f64>()
                    .map_err(|_| "볼린저 밴드 승수 파싱 오류".to_string())?;

                if multiplier <= 0.0 {
                    return Err("볼린저 밴드 승수는 0보다 커야 합니다".to_string());
                }

                multiplier
            }
            None => return Err("multiplier 설정이 필요합니다".to_string()),
        };

        let result = BBandStrategyConfig {
            count,
            period,
            multiplier,
        };

        result.validate()?;
        Ok(result)
    }
}

/// 볼린저 밴드 전략 데이터
#[derive(Debug)]
struct StrategyData<C: Candle> {
    candle: C,
    bband: BBand,
}

impl<C: Candle> StrategyData<C> {
    /// 새 전략 데이터 생성
    fn new(candle: C, bband: BBand) -> StrategyData<C> {
        StrategyData { candle, bband }
    }
}

impl<C: Candle> GetCandle<C> for StrategyData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> StrategyDataOps<C> for StrategyData<C> {}

/// 볼린저 밴드 전략 컨텍스트
#[derive(Debug)]
struct StrategyContext<C: Candle> {
    bbandbuilder: BBandBuilder<C>,
    items: Vec<StrategyData<C>>,
}

impl<C: Candle> Display for StrategyContext<C> {
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

impl<C: Candle + 'static> StrategyContext<C> {
    /// 새 전략 컨텍스트 생성
    fn new(config: &BBandStrategyConfig, storage: &CandleStore<C>) -> StrategyContext<C> {
        let bbandbuilder = BBandBuilder::new(config.period, config.multiplier);

        let mut ctx = StrategyContext {
            bbandbuilder,
            items: vec![],
        };

        for item in storage.get_reversed_items().iter().rev() {
            let bband = ctx.bbandbuilder.next(item);
            ctx.items.insert(0, StrategyData::new(item.clone(), bband));
        }

        ctx
    }
}

impl<C: Candle> StrategyContextOps<StrategyData<C>, C> for StrategyContext<C> {
    fn next_data(&mut self, candle: C) -> StrategyData<C> {
        let bband = self.bbandbuilder.next(&candle);
        StrategyData::new(candle, bband)
    }

    fn datum(&self) -> &Vec<StrategyData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<StrategyData<C>> {
        &mut self.items
    }
}

/// 볼린저 밴드 기반 트레이딩 전략
#[derive(Debug)]
pub struct BBandStrategy<C: Candle> {
    /// 전략 설정
    config: BBandStrategyConfig,
    /// 전략 컨텍스트 (데이터 보관 및 연산)
    ctx: StrategyContext<C>,
}

impl<C: Candle> Display for BBandStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[볼린저밴드전략] 설정: {{기간: {}, 승수: {}, 확인캔들수: {}}}, 컨텍스트: {}",
            self.config.period, self.config.multiplier, self.config.count, self.ctx
        )
    }
}

impl<C: Candle + 'static> BBandStrategy<C> {
    /// 새 볼린저밴드 전략 인스턴스 생성 (JSON 설정 파일 사용)
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `json_config` - JSON 형식의 설정 문자열
    ///
    /// # Returns
    /// * `Result<BBandStrategy<C>, String>` - 초기화된 볼린저밴드 전략 인스턴스 또는 오류
    pub fn new(storage: &CandleStore<C>, json_config: &str) -> Result<BBandStrategy<C>, String> {
        let config = BBandStrategyConfig::from_json(json_config)?;
        info!("볼린저밴드 전략 설정: {:?}", config);
        let ctx = StrategyContext::new(&config, storage);

        Ok(BBandStrategy { config, ctx })
    }

    /// 새 볼린저 밴드 전략 인스턴스 생성 (설정 직접 제공)
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<BBandStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => BBandStrategyConfig::from_hash_map(&cfg)?,
            None => BBandStrategyConfig::default(),
        };

        info!("볼린저 밴드 전략 설정: {:?}", strategy_config);
        let ctx = StrategyContext::new(&strategy_config, storage);

        Ok(BBandStrategy {
            config: strategy_config,
            ctx,
        })
    }

    /// 가격이 볼린저 밴드 하한선 아래로 내려갔는지 확인
    fn is_below_lower_band(&self) -> bool {
        if let Some(first) = self.ctx.items.first() {
            first.candle.close_price() < first.bband.lower()
        } else {
            false
        }
    }

    /// 가격이 볼린저 밴드 상한선 위로 올라갔는지 확인
    fn is_above_upper_band(&self) -> bool {
        if let Some(first) = self.ctx.items.first() {
            first.candle.close_price() > first.bband.upper()
        } else {
            false
        }
    }

    /// 가격이 볼린저 밴드 중앙선 위로 올라갔는지 확인
    fn is_above_middle_band(&self) -> bool {
        if let Some(first) = self.ctx.items.first() {
            first.candle.close_price() > first.bband.average()
        } else {
            false
        }
    }

    /// 가격이 볼린저 밴드 중앙선 아래로 내려갔는지 확인
    fn is_below_middle_band(&self) -> bool {
        if let Some(first) = self.ctx.items.first() {
            first.candle.close_price() < first.bband.average()
        } else {
            false
        }
    }
}

impl<C: Candle + 'static> Strategy<C> for BBandStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle)
    }

    fn should_enter(&self, _candle: &C) -> bool {
        self.is_below_lower_band()
    }

    fn should_exit(&self, _holdings: &TradePosition, _candle: &C) -> bool {
        self.is_above_middle_band()
    }

    fn get_position(&self) -> PositionType {
        PositionType::Long
    }

    fn get_name(&self) -> StrategyType {
        StrategyType::BBand
    }
}
