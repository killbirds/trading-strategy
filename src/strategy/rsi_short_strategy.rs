use super::Strategy;
use super::StrategyType;
use super::context::{GetCandle, StrategyContextOps, StrategyDataOps};
use crate::candle_store::CandleStore;
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

/// RSI 숏 전략 설정
#[derive(Debug, Deserialize)]
pub struct RSIShortStrategyConfig {
    /// RSI 계산 기간
    pub rsi_period: usize,
    /// RSI 상단 경계 (과매수 기준)
    pub rsi_upper: f64,
    /// RSI 하단 경계 (과매도 기준)
    pub rsi_lower: f64,
    /// RSI 신호 확인 기간
    pub rsi_count: usize,
    /// 이동평균 계산 방식
    pub ma: MAType,
    /// 이동평균 기간 목록
    pub ma_periods: Vec<usize>,
}

impl Default for RSIShortStrategyConfig {
    /// 기본 설정값 반환
    fn default() -> Self {
        RSIShortStrategyConfig {
            rsi_period: 14,
            rsi_upper: 70.0,
            rsi_lower: 30.0,
            rsi_count: 3,
            ma: MAType::EMA,
            ma_periods: vec![5, 20, 60],
        }
    }
}

impl RSIShortStrategyConfig {
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
                "RSI 하한({})은 상한({})보다 작아야 합니다",
                self.rsi_lower, self.rsi_upper
            ));
        }

        if self.rsi_count == 0 {
            return Err("RSI 판정 횟수는 0보다 커야 합니다".to_string());
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
    /// * `Result<RSIShortStrategyConfig, String>` - 로드된 설정 또는 오류
    fn from_json(json: &str) -> Result<RSIShortStrategyConfig, String> {
        match serde_json::from_str::<RSIShortStrategyConfig>(json) {
            Ok(config) => {
                config.validate()?;
                Ok(config)
            }
            Err(e) => Err(format!("JSON 설정 역직렬화 실패: {}", e)),
        }
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(config: &HashMap<String, String>) -> Result<RSIShortStrategyConfig, String> {
        // RSI 기간 설정
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

        // RSI 상한 설정
        let rsi_upper = match config.get("rsi_upper") {
            Some(upper_str) => upper_str
                .parse::<f64>()
                .map_err(|_| "RSI 상한 파싱 오류".to_string())?,
            None => return Err("rsi_upper 설정이 필요합니다".to_string()),
        };

        // RSI 하한 설정
        let rsi_lower = match config.get("rsi_lower") {
            Some(lower_str) => lower_str
                .parse::<f64>()
                .map_err(|_| "RSI 하한 파싱 오류".to_string())?,
            None => return Err("rsi_lower 설정이 필요합니다".to_string()),
        };

        if rsi_lower >= rsi_upper {
            return Err(format!(
                "RSI 하한({})은 상한({})보다 작아야 합니다",
                rsi_lower, rsi_upper
            ));
        }

        // RSI 판정 횟수 설정
        let rsi_count = match config.get("rsi_count") {
            Some(count_str) => {
                let count = count_str
                    .parse::<usize>()
                    .map_err(|_| "RSI 판정 횟수 파싱 오류".to_string())?;

                if count == 0 {
                    return Err("RSI 판정 횟수는 0보다 커야 합니다".to_string());
                }

                count
            }
            None => return Err("rsi_count 설정이 필요합니다".to_string()),
        };

        // 이동평균 타입 설정
        let ma = match config.get("ma").map(|s| s.as_str()) {
            Some("sma") => MAType::SMA,
            Some("ema") => MAType::EMA,
            Some(unknown) => return Err(format!("알 수 없는 이동평균 유형: {}", unknown)),
            None => MAType::EMA,
        };

        // 이동평균 기간 설정
        let ma_periods = match config.get("ma_periods") {
            Some(periods_str) => match crate::strategy::split(periods_str) {
                Ok(periods) => {
                    if periods.is_empty() {
                        return Err("이동평균 기간이 지정되지 않았습니다".to_string());
                    }
                    periods
                }
                Err(e) => return Err(format!("이동평균 기간 파싱 오류: {}", e)),
            },
            None => return Err("ma_periods 설정이 필요합니다".to_string()),
        };

        let result = RSIShortStrategyConfig {
            rsi_period,
            rsi_upper,
            rsi_lower,
            rsi_count,
            ma,
            ma_periods,
        };

        result.validate()?;
        Ok(result)
    }
}

/// RSI 숏 전략 데이터
#[derive(Debug)]
struct StrategyData<C: Candle> {
    /// 현재 캔들 데이터
    candle: C,
    /// RSI 지표
    rsi: RSI,
    /// 이동평균선 집합
    mas: MAs,
}

impl<C: Candle> StrategyData<C> {
    /// 새 전략 데이터 생성
    fn new(candle: C, rsi: RSI, mas: MAs) -> StrategyData<C> {
        StrategyData { candle, rsi, mas }
    }

    /// 이동평균이 정규 배열(오름차순)인지 확인
    fn is_ma_regular_arrangement(&self) -> bool {
        self.is_regular_arrangement(|data| &data.mas, |ma| ma.get())
    }

    /// 이동평균이 역배열(내림차순)인지 확인
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

/// RSI 숏 전략 컨텍스트
#[derive(Debug)]
struct StrategyContext<C: Candle> {
    /// RSI 빌더
    rsibuilder: RSIBuilder<C>,
    /// 이동평균 빌더
    masbuilder: MAsBuilder<C>,
    /// 전략 데이터 히스토리 (최신 데이터가 인덱스 0)
    items: Vec<StrategyData<C>>,
}

impl<C: Candle> Display for StrategyContext<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(
                f,
                "캔들: {}, RSI: {}, MAs: {}",
                first.candle, first.rsi, first.mas
            ),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> StrategyContext<C> {
    /// 새 전략 컨텍스트 생성
    fn new(config: &RSIShortStrategyConfig, storage: &CandleStore<C>) -> StrategyContext<C> {
        let rsibuilder = RSIBuilder::new(config.rsi_period);
        let masbuilder = MAsBuilderFactory::build::<C>(&config.ma, &config.ma_periods);

        let mut ctx = StrategyContext {
            rsibuilder,
            masbuilder,
            items: vec![],
        };

        ctx.init(storage.get_reversed_items());
        ctx
    }

    /// n개의 연속 데이터에서 이동평균이 정규 배열인지 확인
    fn is_ma_regular_arrangement(&self, n: usize) -> bool {
        self.is_all(|data| data.is_ma_regular_arrangement(), n)
    }

    /// n개의 연속 데이터에서 이동평균이 역배열인지 확인
    fn is_ma_reverse_arrangement(&self, n: usize) -> bool {
        self.is_all(|data| data.is_ma_reverse_arrangement(), n)
    }
}

impl<C: Candle> StrategyContextOps<StrategyData<C>, C> for StrategyContext<C> {
    fn next_data(&mut self, candle: C) -> StrategyData<C> {
        let rsi = self.rsibuilder.next(&candle);
        let mas = self.masbuilder.next(&candle);
        StrategyData::new(candle, rsi, mas)
    }

    fn datum(&self) -> &Vec<StrategyData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<StrategyData<C>> {
        &mut self.items
    }
}

/// RSI 기반 숏 트레이딩 전략
#[derive(Debug)]
pub struct RSIShortStrategy<C: Candle> {
    /// 전략 설정
    config: RSIShortStrategyConfig,
    /// 전략 컨텍스트 (데이터 보관 및 연산)
    ctx: StrategyContext<C>,
}

impl<C: Candle> Display for RSIShortStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[RSI숏전략] 설정: {{RSI기간: {}, 상한: {}, 하한: {}}}, 컨텍스트: {}",
            self.config.rsi_period, self.config.rsi_upper, self.config.rsi_lower, self.ctx
        )
    }
}

impl<C: Candle + 'static> RSIShortStrategy<C> {
    /// 새 RSI 숏 전략 인스턴스 생성 (JSON 설정 파일 사용)
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `json_config` - JSON 형식의 설정 문자열
    ///
    /// # Returns
    /// * `Result<RSIShortStrategy<C>, String>` - 초기화된 RSI 숏 전략 인스턴스 또는 오류
    pub fn new(storage: &CandleStore<C>, json_config: &str) -> Result<RSIShortStrategy<C>, String> {
        let config = RSIShortStrategyConfig::from_json(json_config)?;
        info!("RSI 숏 전략 설정: {:?}", config);
        let ctx = StrategyContext::new(&config, storage);

        Ok(RSIShortStrategy { config, ctx })
    }

    /// 새 RSI 숏 전략 인스턴스 생성 (설정 직접 제공)
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<RSIShortStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => RSIShortStrategyConfig::from_hash_map(&cfg)?,
            None => RSIShortStrategyConfig::default(),
        };

        info!("RSI 숏 전략 설정: {:?}", strategy_config);
        let ctx = StrategyContext::new(&strategy_config, storage);

        Ok(RSIShortStrategy {
            config: strategy_config,
            ctx,
        })
    }

    /// RSI가 과매수 영역(상한선 이상)인지 확인
    fn is_rsi_overbought(&self) -> bool {
        if let Some(first) = self.ctx.items.first() {
            first.rsi.rsi >= self.config.rsi_upper
        } else {
            false
        }
    }

    /// RSI가 과매도 영역(하한선 이하)인지 확인
    fn is_rsi_oversold(&self) -> bool {
        if let Some(first) = self.ctx.items.first() {
            first.rsi.rsi <= self.config.rsi_lower
        } else {
            false
        }
    }
}

impl<C: Candle + 'static> Strategy<C> for RSIShortStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle)
    }

    fn should_enter(&self, _candle: &C) -> bool {
        // 이동평균이 정규 배열이면 숏 진입 금지 (상승 추세)
        if self.ctx.is_ma_regular_arrangement(1) {
            return false;
        }

        // RSI가 과매수 구간을 돌파했을 때 숏 진입 신호
        self.is_rsi_overbought()
    }

    fn should_exit(&self, _holdings: &TradePosition, _candle: &C) -> bool {
        // 이동평균이 역배열이면 숏 청산 금지 (하락 추세)
        if self.ctx.is_ma_reverse_arrangement(1) {
            return false;
        }

        // RSI가 과매도 구간을 돌파했을 때 숏 청산 신호
        self.is_rsi_oversold()
    }

    fn get_position(&self) -> PositionType {
        PositionType::Short
    }

    fn get_name(&self) -> StrategyType {
        StrategyType::RSIShort
    }
}
