use super::Strategy;
use super::StrategyType;
use super::context::{GetCandle, StrategyContextOps, StrategyDataOps};
use crate::candle_store::CandleStore;
use crate::indicator::bband::{BBand, BBandBuilder};
use crate::indicator::ma::{MAType, MAs, MAsBuilder, MAsBuilderFactory};
use crate::indicator::rsi::{RSI, RSIBuilder};
use crate::model::PositionType;
use crate::model::TradePosition;
use log::info;
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::fmt::Display;
use trading_chart::Candle;

/// CopyS 전략 설정
#[derive(Debug, Deserialize)]
pub struct CopysStrategyConfig {
    /// ATR 계산 기간
    pub atr_period: usize,
    /// ATR 배수 (진입가 설정용)
    pub atr_multiplier: f64,
    /// 슈퍼트렌드 계산 기간
    pub st_period: usize,
    /// 슈퍼트렌드 배수
    pub st_multiplier: f64,
    /// RSI 계산 기간
    pub rsi_period: usize,
    /// RSI 상한값
    pub rsi_upper: f64,
    /// RSI 하한값
    pub rsi_lower: f64,
    /// 볼린저밴드 계산 기간
    pub bband_period: usize,
    /// 볼린저밴드 배수
    pub bband_multiplier: f64,
}

impl Default for CopysStrategyConfig {
    /// 기본 설정값 반환
    fn default() -> Self {
        CopysStrategyConfig {
            atr_period: 14,
            atr_multiplier: 1.0,
            st_period: 10,
            st_multiplier: 3.0,
            rsi_period: 14,
            rsi_upper: 70.0,
            rsi_lower: 30.0,
            bband_period: 20,
            bband_multiplier: 2.0,
        }
    }
}

impl CopysStrategyConfig {
    /// 설정의 유효성을 검사합니다.
    ///
    /// 모든 설정 값이 유효한지 확인하고, 유효하지 않은 경우 오류 메시지를 반환합니다.
    ///
    /// # Returns
    /// * `Result<(), String>` - 유효성 검사 결과 (성공 또는 오류 메시지)
    pub fn validate(&self) -> Result<(), String> {
        if self.atr_period == 0 {
            return Err("ATR 기간은 0보다 커야 합니다".to_string());
        }

        if self.atr_multiplier <= 0.0 {
            return Err("ATR 배수는 0보다 커야 합니다".to_string());
        }

        if self.st_period < 2 {
            return Err("슈퍼트렌드 기간은 2 이상이어야 합니다".to_string());
        }

        if self.st_multiplier <= 0.0 {
            return Err("슈퍼트렌드 배수는 0보다 커야 합니다".to_string());
        }

        if self.rsi_period < 2 {
            return Err("RSI 기간은 2 이상이어야 합니다".to_string());
        }

        if self.rsi_lower >= self.rsi_upper {
            return Err(format!(
                "RSI 하한값({})이 상한값({})보다 크거나 같을 수 없습니다",
                self.rsi_lower, self.rsi_upper
            ));
        }

        if self.bband_period < 2 {
            return Err("볼린저밴드 기간은 2 이상이어야 합니다".to_string());
        }

        if self.bband_multiplier <= 0.0 {
            return Err("볼린저밴드 배수는 0보다 커야 합니다".to_string());
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
    /// * `Result<CopysStrategyConfig, String>` - 로드된 설정 또는 오류
    fn from_json(json: &str) -> Result<CopysStrategyConfig, String> {
        match serde_json::from_str::<CopysStrategyConfig>(json) {
            Ok(config) => Ok(config),
            Err(e) => Err(format!("JSON 설정 역직렬화 실패: {}", e)),
        }
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(config: &HashMap<String, String>) -> Result<CopysStrategyConfig, String> {
        // RSI 관련 설정
        let rsi_period = match config.get("rsi_period") {
            Some(period_str) => {
                let period = period_str
                    .parse::<usize>()
                    .map_err(|_| "RSI 기간 파싱 오류".to_string())?;

                if period < 2 {
                    return Err("RSI 기간은 2 이상이어야 합니다".to_string());
                }

                period
            }
            None => return Err("rsi_period 설정이 필요합니다".to_string()),
        };

        let rsi_lower = match config.get("rsi_lower") {
            Some(lower_str) => {
                let lower = lower_str
                    .parse::<f64>()
                    .map_err(|_| "RSI 하한값 파싱 오류".to_string())?;

                if !(0.0..=100.0).contains(&lower) {
                    return Err(format!("RSI 하한값({})은 0과 100 사이여야 합니다", lower));
                }

                lower
            }
            None => return Err("rsi_lower 설정이 필요합니다".to_string()),
        };

        let rsi_upper = match config.get("rsi_upper") {
            Some(upper_str) => {
                let upper = upper_str
                    .parse::<f64>()
                    .map_err(|_| "RSI 상한값 파싱 오류".to_string())?;

                if !(0.0..=100.0).contains(&upper) {
                    return Err(format!("RSI 상한값({})은 0과 100 사이여야 합니다", upper));
                }

                upper
            }
            None => return Err("rsi_upper 설정이 필요합니다".to_string()),
        };

        if rsi_lower >= rsi_upper {
            return Err(format!(
                "RSI 하한값({})은 상한값({})보다 작아야 합니다",
                rsi_lower, rsi_upper
            ));
        }

        // 볼린저 밴드 관련 설정
        let bband_period = match config.get("bband_period") {
            Some(period_str) => {
                let period = period_str
                    .parse::<usize>()
                    .map_err(|_| "볼린저 밴드 기간 파싱 오류".to_string())?;

                if period < 2 {
                    return Err("볼린저 밴드 기간은 2 이상이어야 합니다".to_string());
                }

                period
            }
            None => return Err("bband_period 설정이 필요합니다".to_string()),
        };

        let bband_multiplier = match config.get("bband_multiplier") {
            Some(multiplier_str) => {
                let multiplier = multiplier_str
                    .parse::<f64>()
                    .map_err(|_| "볼린저 밴드 승수 파싱 오류".to_string())?;

                if multiplier <= 0.0 {
                    return Err("볼린저 밴드 승수는 0보다 커야 합니다".to_string());
                }

                multiplier
            }
            None => return Err("bband_multiplier 설정이 필요합니다".to_string()),
        };

        Ok(CopysStrategyConfig {
            atr_period: 14,
            atr_multiplier: 1.0,
            st_period: 10,
            st_multiplier: 3.0,
            rsi_period,
            rsi_lower,
            rsi_upper,
            bband_period,
            bband_multiplier,
        })
    }
}

struct StrategyData<C: Candle> {
    candle: C,
    rsi: RSI,
    mas: MAs,
    bband: BBand,
}

impl<C: Candle> StrategyData<C> {
    fn new(candle: C, rsi: RSI, mas: MAs, bband: BBand) -> StrategyData<C> {
        StrategyData {
            candle,
            rsi,
            mas,
            bband,
        }
    }

    fn is_ma_regular_arrangement(&self) -> bool {
        self.is_regular_arrangement(|data| &data.mas, |ma| ma.get())
    }

    fn is_ma_reverse_arrangement(&self) -> bool {
        self.is_reverse_arrangement(|data| &data.mas, |ma| ma.get())
    }
}

impl<C: Candle> GetCandle<C> for StrategyData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> StrategyDataOps<C> for StrategyData<C> {}

struct StrategyContext<C: Candle> {
    rsibuilder: RSIBuilder<C>,
    masbuilder: MAsBuilder<C>,
    bbandbuilder: BBandBuilder<C>,
    items: Vec<StrategyData<C>>,
}

impl<C: Candle> Display for StrategyContext<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (head, tail) = self.items.split_first().unwrap();
        write!(
            f,
            "candle: {}, rsi: [{}, {:?}], mas: {}, bband: {}",
            head.candle,
            head.rsi,
            tail.iter()
                .take(4)
                .map(|item| item.rsi.rsi)
                .collect::<Vec<_>>(),
            head.mas,
            head.bband
        )
    }
}

impl<C: Candle + 'static> StrategyContext<C> {
    fn new(config: &CopysStrategyConfig, storage: &CandleStore<C>) -> StrategyContext<C> {
        let rsibuilder = RSIBuilder::new(config.rsi_period);
        let masbuilder = MAsBuilderFactory::build::<C>(&MAType::EMA, &[10, 20, 60]);
        let bbandbuilder = BBandBuilder::new(config.bband_period, config.bband_multiplier);
        let mut ctx = StrategyContext {
            rsibuilder,
            masbuilder,
            bbandbuilder,
            items: vec![],
        };

        ctx.init(storage.get_reversed_items());
        ctx
    }

    fn is_ma_regular_arrangement(&self, n: usize) -> bool {
        self.is_all(|data| data.is_ma_regular_arrangement(), n)
    }

    fn is_ma_reverse_arrangement(&self, n: usize) -> bool {
        self.is_all(|data| data.is_ma_reverse_arrangement(), n)
    }
}

impl<C: Candle> StrategyContextOps<StrategyData<C>, C> for StrategyContext<C> {
    fn next_data(&mut self, candle: C) -> StrategyData<C> {
        let rsi = self.rsibuilder.next(&candle);
        let mas = self.masbuilder.next(&candle);
        let bband = self.bbandbuilder.next(&candle);
        StrategyData::new(candle, rsi, mas, bband)
    }

    fn datum(&self) -> &Vec<StrategyData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<StrategyData<C>> {
        &mut self.items
    }
}

pub struct CopysStrategy<C: Candle> {
    config: CopysStrategyConfig,
    ctx: StrategyContext<C>,
}

impl<C: Candle> Display for CopysStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[Copys전략] 설정: {{RSI: {}(상:{}/하:{}), BB: {}({}), ATR: {}({})}}, 컨텍스트: {}",
            self.config.rsi_period,
            self.config.rsi_upper,
            self.config.rsi_lower,
            self.config.bband_period,
            self.config.bband_multiplier,
            self.config.atr_period,
            self.config.atr_multiplier,
            self.ctx
        )
    }
}

impl<C: Candle + 'static> CopysStrategy<C> {
    /// 새 CopyS 전략 인스턴스 생성 (JSON 설정 파일 사용)
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `json_config` - JSON 형식의 설정 문자열
    ///
    /// # Returns
    /// * `Result<CopysStrategy<C>, String>` - 초기화된 CopyS 전략 인스턴스 또는 오류
    pub fn new(storage: &CandleStore<C>, json_config: &str) -> Result<CopysStrategy<C>, String> {
        let config = CopysStrategyConfig::from_json(json_config)?;
        info!("CopyS 전략 설정: {:?}", config);
        let ctx = StrategyContext::new(&config, storage);

        Ok(CopysStrategy { config, ctx })
    }

    /// 새 CopyS 전략 인스턴스 생성 (설정 직접 제공)
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<CopysStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => CopysStrategyConfig::from_hash_map(&cfg)?,
            None => CopysStrategyConfig::default(),
        };

        info!("CopyS 전략 설정: {:?}", strategy_config);
        let ctx = StrategyContext::new(&strategy_config, storage);

        Ok(CopysStrategy {
            config: strategy_config,
            ctx,
        })
    }
}

impl<C: Candle + 'static> Strategy<C> for CopysStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle);
    }

    fn should_enter(&self, _candle: &C) -> bool {
        if self.ctx.is_ma_reverse_arrangement(1) {
            false
        } else {
            self.ctx.is_break_through_by_satisfying(
                |data| data.rsi.rsi > self.config.rsi_lower,
                1,
                3,
            )
        }
    }

    fn should_exit(&self, _holdings: &TradePosition, _candle: &C) -> bool {
        if self.ctx.is_ma_regular_arrangement(1) {
            false
        } else {
            self.ctx.is_break_through_by_satisfying(
                |data| data.rsi.rsi < self.config.rsi_upper,
                1,
                3,
            )
        }
    }

    fn get_position(&self) -> PositionType {
        PositionType::Long
    }

    fn get_name(&self) -> StrategyType {
        StrategyType::Copys
    }
}
