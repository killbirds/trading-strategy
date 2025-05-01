use super::StrategyType;
use super::context::{GetCandle, StrategyContextOps, StrategyDataOps};
use super::{Strategy, split};
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

/// Copys 숏 전략 설정
#[derive(Debug, Deserialize)]
pub struct CopysShortStrategyConfig {
    /// RSI 계산 기간
    pub rsi_period: usize,
    /// RSI 상한값
    pub rsi_upper: f64,
    /// RSI 하한값
    pub rsi_lower: f64,
    /// RSI 조건 판정 횟수
    pub rsi_count: usize,
    /// 볼린저밴드 계산 기간
    pub bband_period: usize,
    /// 볼린저밴드 표준편차 승수
    pub bband_multiplier: f64,
    /// 볼린저밴드 조건 판정 횟수
    pub bband_count: usize,
    /// 이동평균 계산 방식
    pub ma: MAType,
    /// 이동평균 기간 목록
    pub ma_periods: Vec<usize>,
}

impl Default for CopysShortStrategyConfig {
    /// 기본 설정값 반환
    fn default() -> Self {
        CopysShortStrategyConfig {
            rsi_period: 14,
            rsi_upper: 70.0,
            rsi_lower: 30.0,
            rsi_count: 3,
            bband_period: 20,
            bband_multiplier: 2.0,
            bband_count: 2,
            ma: MAType::EMA,
            ma_periods: vec![10, 20, 60],
        }
    }
}

impl CopysShortStrategyConfig {
    /// 설정의 유효성을 검사합니다.
    ///
    /// 모든 설정 값이 유효한지 확인하고, 유효하지 않은 경우 오류 메시지를 반환합니다.
    ///
    /// # Returns
    /// * `Result<(), String>` - 유효성 검사 결과 (성공 또는 오류 메시지)
    pub fn validate(&self) -> Result<(), String> {
        if self.rsi_period < 2 {
            return Err("RSI 기간은 2 이상이어야 합니다".to_string());
        }

        if self.rsi_lower >= self.rsi_upper {
            return Err(format!(
                "RSI 하한값({})이 상한값({})보다 크거나 같을 수 없습니다",
                self.rsi_lower, self.rsi_upper
            ));
        }

        if self.rsi_count == 0 {
            return Err("RSI 판정 횟수는 0보다 커야 합니다".to_string());
        }

        if self.bband_period < 2 {
            return Err("볼린저밴드 기간은 2 이상이어야 합니다".to_string());
        }

        if self.bband_multiplier <= 0.0 {
            return Err("볼린저밴드 승수는 0보다 커야 합니다".to_string());
        }

        if self.bband_count == 0 {
            return Err("볼린저밴드 판정 횟수는 0보다 커야 합니다".to_string());
        }

        if self.ma_periods.is_empty() {
            return Err("이동평균 기간이 지정되지 않았습니다".to_string());
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
    /// * `Result<CopysShortStrategyConfig, String>` - 로드된 설정 또는 오류
    fn from_json(json: &str) -> Result<CopysShortStrategyConfig, String> {
        match serde_json::from_str::<CopysShortStrategyConfig>(json) {
            Ok(config) => {
                config.validate()?;
                Ok(config)
            }
            Err(e) => Err(format!("JSON 설정 역직렬화 실패: {}", e)),
        }
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(config: &HashMap<String, String>) -> Result<CopysShortStrategyConfig, String> {
        // RSI 관련 설정
        let rsi_count = match config.get("rsi_count") {
            Some(value_str) => {
                let value = value_str
                    .parse::<usize>()
                    .map_err(|_| "RSI 연속 판정 횟수 파싱 오류".to_string())?;

                if value == 0 {
                    return Err("RSI 연속 판정 횟수는 0보다 커야 합니다".to_string());
                }

                value
            }
            None => return Err("rsi_count 설정이 필요합니다".to_string()),
        };

        let rsi_lower = match config.get("rsi_lower") {
            Some(value_str) => value_str
                .parse::<f64>()
                .map_err(|_| "RSI 하한 임계값 파싱 오류".to_string())?,
            None => return Err("rsi_lower 설정이 필요합니다".to_string()),
        };

        let rsi_upper = match config.get("rsi_upper") {
            Some(value_str) => value_str
                .parse::<f64>()
                .map_err(|_| "RSI 상한 임계값 파싱 오류".to_string())?,
            None => return Err("rsi_upper 설정이 필요합니다".to_string()),
        };

        if rsi_lower >= rsi_upper {
            return Err(format!(
                "숏 전략에서 RSI 하한값({})은 상한값({})보다 작거나 같아야 합니다",
                rsi_lower, rsi_upper
            ));
        }

        let rsi_period = match config.get("rsi_period") {
            Some(value_str) => {
                let value = value_str
                    .parse::<usize>()
                    .map_err(|_| "RSI 계산 기간 파싱 오류".to_string())?;

                if value == 0 {
                    return Err("RSI 계산 기간은 0보다 커야 합니다".to_string());
                }

                value
            }
            None => return Err("rsi_period 설정이 필요합니다".to_string()),
        };

        // 볼린저밴드 관련 설정
        let bband_count = match config.get("bband_count") {
            Some(value_str) => {
                let value = value_str
                    .parse::<usize>()
                    .map_err(|_| "볼린저밴드 연속 판정 횟수 파싱 오류".to_string())?;

                if value == 0 {
                    return Err("볼린저밴드 연속 판정 횟수는 0보다 커야 합니다".to_string());
                }

                value
            }
            None => return Err("bband_count 설정이 필요합니다".to_string()),
        };

        let bband_period = match config.get("bband_period") {
            Some(value_str) => {
                let value = value_str
                    .parse::<usize>()
                    .map_err(|_| "볼린저밴드 계산 기간 파싱 오류".to_string())?;

                if value == 0 {
                    return Err("볼린저밴드 계산 기간은 0보다 커야 합니다".to_string());
                }

                value
            }
            None => return Err("bband_period 설정이 필요합니다".to_string()),
        };

        let bband_multiplier = match config.get("bband_multiplier") {
            Some(value_str) => {
                let value = value_str
                    .parse::<f64>()
                    .map_err(|_| "볼린저밴드 표준편차 승수 파싱 오류".to_string())?;

                if value <= 0.0 {
                    return Err("볼린저밴드 표준편차 승수는 0보다 커야 합니다".to_string());
                }

                value
            }
            None => return Err("bband_multiplier 설정이 필요합니다".to_string()),
        };

        // 이동평균 관련 설정
        let ma = match config.get("ma").map(|s| s.as_str()) {
            Some("sma") => MAType::SMA,
            Some("ema") => MAType::EMA,
            Some(unknown) => return Err(format!("알 수 없는 이동평균 유형: {}", unknown)),
            None => return Err("ma 설정이 필요합니다".to_string()),
        };

        let ma_periods = match config.get("ma_periods") {
            Some(periods_str) => match split(periods_str) {
                Ok(periods) => {
                    if periods.is_empty() {
                        return Err("이동평균 기간이 비어 있습니다".to_string());
                    }

                    for &period in &periods {
                        if period == 0 {
                            return Err("이동평균 기간은 0보다 커야 합니다".to_string());
                        }
                    }

                    periods
                }
                Err(e) => return Err(format!("이동평균 기간 파싱 오류: {}", e)),
            },
            None => return Err("ma_periods 설정이 필요합니다".to_string()),
        };

        Ok(CopysShortStrategyConfig {
            rsi_count,
            rsi_lower,
            rsi_upper,
            rsi_period,
            bband_count,
            bband_period,
            bband_multiplier,
            ma,
            ma_periods,
        })
    }
}

/// Copys 숏 전략 데이터
#[derive(Debug)]
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

    /// 이동평균선이 정상 배열인지 검사 (숏 전략에서는 역배열 조건에서 청산)
    fn is_ma_regular_arrangement(&self) -> bool {
        self.is_regular_arrangement(|data| &data.mas, |ma| ma.get())
    }

    /// 이동평균선이 역배열인지 검사 (숏 전략에서는 역배열 조건에서 진입)
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

/// Copys 숏 전략 컨텍스트
#[derive(Debug)]
struct StrategyContext<C: Candle> {
    rsibuilder: RSIBuilder<C>,
    masbuilder: MAsBuilder<C>,
    bbandbuilder: BBandBuilder<C>,
    items: Vec<StrategyData<C>>,
}

impl<C: Candle> Display for StrategyContext<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(head) = self.items.first() {
            let tail_rsis = self
                .items
                .iter()
                .skip(1)
                .take(4)
                .map(|item| item.rsi.rsi)
                .collect::<Vec<_>>();

            write!(
                f,
                "캔들: {}, RSI: [{}, {:?}], MAs: {}, BBand: {}",
                head.candle, head.rsi, tail_rsis, head.mas, head.bband
            )
        } else {
            write!(f, "데이터 없음")
        }
    }
}

impl<C: Candle + 'static> StrategyContext<C> {
    fn new(config: &CopysShortStrategyConfig, storage: &CandleStore<C>) -> StrategyContext<C> {
        let rsibuilder = RSIBuilder::new(config.rsi_period);
        let masbuilder = MAsBuilderFactory::build::<C>(&config.ma, &config.ma_periods);
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

    /// 이동평균선이 정상 배열인지 확인
    fn is_ma_regular_arrangement(&self, n: usize) -> bool {
        self.is_all(|data| data.is_ma_regular_arrangement(), n)
    }

    /// 이동평균선이 역배열인지 확인
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

/// Copys 기반 숏 전략
#[derive(Debug)]
pub struct CopysShortStrategy<C: Candle> {
    config: CopysShortStrategyConfig,
    ctx: StrategyContext<C>,
}

impl<C: Candle> Display for CopysShortStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let periods = self
            .config
            .ma_periods
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");

        write!(
            f,
            "[Copys숏전략] 설정: {{RSI: {}(상:{}/하:{}), BB: {}({}), MA타입: {:?}({})}}, 컨텍스트: {}",
            self.config.rsi_period,
            self.config.rsi_upper,
            self.config.rsi_lower,
            self.config.bband_period,
            self.config.bband_multiplier,
            self.config.ma,
            periods,
            self.ctx
        )
    }
}

impl<C: Candle + 'static> CopysShortStrategy<C> {
    /// 새 코피스 숏 전략 인스턴스 생성 (JSON 설정 파일 사용)
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `json_config` - JSON 형식의 설정 문자열
    ///
    /// # Returns
    /// * `Result<CopysShortStrategy<C>, String>` - 초기화된 코피스 숏 전략 인스턴스 또는 오류
    pub fn new(
        storage: &CandleStore<C>,
        json_config: &str,
    ) -> Result<CopysShortStrategy<C>, String> {
        let config = CopysShortStrategyConfig::from_json(json_config)?;
        info!("코피스 숏 전략 설정: {:?}", config);
        let ctx = StrategyContext::new(&config, storage);

        Ok(CopysShortStrategy { config, ctx })
    }

    /// 새 코피스 숏 전략 인스턴스 생성 (설정 직접 제공)
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `config` - 전략 설정 (HashMap 형태)
    ///
    /// # Returns
    /// * `Result<CopysShortStrategy<C>, String>` - 초기화된 코피스 숏 전략 인스턴스 또는 오류
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<CopysShortStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => CopysShortStrategyConfig::from_hash_map(&cfg)?,
            None => CopysShortStrategyConfig::default(),
        };

        info!("코피스 숏 전략 설정: {:?}", strategy_config);
        let ctx = StrategyContext::new(&strategy_config, storage);

        Ok(CopysShortStrategy {
            config: strategy_config,
            ctx,
        })
    }
}

impl<C: Candle + 'static> Strategy<C> for CopysShortStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle);
    }

    fn should_enter(&self, _candle: &C) -> bool {
        // 이동평균선이 정상 배열이면 거래하지 않음 (역배열을 원함)
        if self.ctx.is_ma_regular_arrangement(1) {
            false
        } else {
            // RSI가 상한선보다 높으면 숏 진입 (롱 전략과 반대로 높은 값에서 진입)
            self.ctx.is_break_through_by_satisfying(
                |data| data.rsi.rsi > self.config.rsi_lower,
                1,
                self.config.rsi_count,
            )
        }
    }

    fn should_exit(&self, _holdings: &TradePosition, _candle: &C) -> bool {
        // 이동평균선이 역배열이면 거래하지 않음 (정상 배열을 원함)
        if self.ctx.is_ma_reverse_arrangement(1) {
            false
        } else {
            // RSI가 하한선보다 낮으면 숏 청산
            self.ctx.is_break_through_by_satisfying(
                |data| data.rsi.rsi < self.config.rsi_upper,
                1,
                self.config.rsi_count,
            )
        }
    }

    fn get_position(&self) -> PositionType {
        PositionType::Short
    }

    fn get_name(&self) -> StrategyType {
        StrategyType::CopysShort
    }
}
