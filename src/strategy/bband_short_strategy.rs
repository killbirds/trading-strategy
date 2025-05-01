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

/// 볼린저밴드 숏 전략 설정
#[derive(Debug, Deserialize)]
pub struct BBandShortStrategyConfig {
    /// 판정 기간
    pub count: usize,
    /// 볼린저밴드 계산 기간
    pub period: usize,
    /// 표준편차 배수
    pub multiplier: f64,
}

impl Default for BBandShortStrategyConfig {
    /// 기본 설정값 반환
    fn default() -> Self {
        BBandShortStrategyConfig {
            count: 3,
            period: 20,
            multiplier: 2.0,
        }
    }
}

impl BBandShortStrategyConfig {
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
    /// * `Result<BBandShortStrategyConfig, String>` - 로드된 설정 또는 오류
    fn from_json(json: &str) -> Result<BBandShortStrategyConfig, String> {
        match serde_json::from_str::<BBandShortStrategyConfig>(json) {
            Ok(config) => {
                config.validate()?;
                Ok(config)
            }
            Err(e) => Err(format!("JSON 설정 역직렬화 실패: {}", e)),
        }
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(config: &HashMap<String, String>) -> Result<BBandShortStrategyConfig, String> {
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

        let result = BBandShortStrategyConfig {
            count,
            period,
            multiplier,
        };

        result.validate()?;
        Ok(result)
    }
}

/// 볼린저밴드 숏 전략 데이터
#[derive(Debug)]
struct StrategyData<C: Candle> {
    candle: C,
    bband: BBand,
}

impl<C: Candle> StrategyData<C> {
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

/// 볼린저밴드 숏 전략 컨텍스트
#[derive(Debug)]
struct StrategyContext<C: Candle> {
    bbandbuilder: BBandBuilder<C>,
    items: Vec<StrategyData<C>>,
}

impl<C: Candle> Display for StrategyContext<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(first) = self.items.first() {
            write!(f, "{}", first.bband)
        } else {
            write!(f, "데이터 없음")
        }
    }
}

impl<C: Candle + 'static> StrategyContext<C> {
    fn new(config: &BBandShortStrategyConfig, storage: &CandleStore<C>) -> StrategyContext<C> {
        let bbandbuilder = BBandBuilder::new(config.period, config.multiplier);
        let mut ctx = StrategyContext {
            bbandbuilder,
            items: vec![],
        };

        ctx.init(storage.get_reversed_items());
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

/// 볼린저밴드 기반 숏 트레이딩 전략
#[derive(Debug)]
pub struct BBandShortStrategy<C: Candle> {
    config: BBandShortStrategyConfig,
    ctx: StrategyContext<C>,
}

impl<C: Candle> Display for BBandShortStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[볼린저밴드숏전략] 설정: {{기간: {}, 승수: {}, 확인캔들수: {}}}, 컨텍스트: {}",
            self.config.period, self.config.multiplier, self.config.count, self.ctx
        )
    }
}

impl<C: Candle + 'static> BBandShortStrategy<C> {
    /// 새 볼린저밴드 숏 전략 인스턴스 생성 (JSON 설정 파일 사용)
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `json_config` - JSON 형식의 설정 문자열
    ///
    /// # Returns
    /// * `Result<BBandShortStrategy<C>, String>` - 초기화된 볼린저밴드 숏 전략 인스턴스 또는 오류
    pub fn new(
        storage: &CandleStore<C>,
        json_config: &str,
    ) -> Result<BBandShortStrategy<C>, String> {
        let config = BBandShortStrategyConfig::from_json(json_config)?;
        info!("볼린저밴드 숏 전략 설정: {:?}", config);
        let ctx = StrategyContext::new(&config, storage);

        Ok(BBandShortStrategy { config, ctx })
    }

    /// 새 볼린저밴드 숏 전략 인스턴스 생성 (설정 직접 제공)
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `config` - 전략 설정 (HashMap 형태)
    ///
    /// # Returns
    /// * `Result<BBandShortStrategy<C>, String>` - 초기화된 볼린저밴드 숏 전략 인스턴스 또는 오류
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<BBandShortStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => BBandShortStrategyConfig::from_hash_map(&cfg)?,
            None => BBandShortStrategyConfig::default(),
        };

        info!("볼린저밴드 숏 전략 설정: {:?}", strategy_config);
        let ctx = StrategyContext::new(&strategy_config, storage);

        Ok(BBandShortStrategy {
            config: strategy_config,
            ctx,
        })
    }

    /// 밴드 폭이 충분히 넓은지 확인
    fn is_band_width_sufficient(&self) -> bool {
        self.ctx.is_greater_than_target(
            |data| (data.bband.upper() - data.bband.lower()) / data.bband.average(),
            0.02,
            1,
        )
    }
}

impl<C: Candle + 'static> Strategy<C> for BBandShortStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle);
    }

    fn should_enter(&self, _candle: &C) -> bool {
        // 가격이 상단 밴드를 상향 돌파했을 때 숏 진입 신호
        let is_buy = self.ctx.is_break_through_by_greater_than_candle(
            |data| data.bband.upper(),
            |candle| candle.high_price(),
            1,
            self.config.count,
        );

        // 밴드 폭이 충분히 넓은지 확인
        is_buy && self.is_band_width_sufficient()
    }

    fn should_exit(&self, _holdings: &TradePosition, _candle: &C) -> bool {
        // 가격이 하단 밴드를 하향 돌파했을 때 숏 청산 신호
        self.ctx.is_break_through_by_less_than_candle(
            |data| data.bband.lower(),
            |candle| candle.low_price(),
            1,
            self.config.count,
        )
    }

    fn get_position(&self) -> PositionType {
        PositionType::Short
    }

    fn get_name(&self) -> StrategyType {
        StrategyType::BBandShort
    }
}
