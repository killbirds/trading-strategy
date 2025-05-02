use super::Strategy;
use super::context::{GetCandle, StrategyDataOps};
use crate::candle_store::CandleStore;
use crate::indicator::ma::{MAType, MAs, MAsBuilder, MAsBuilderFactory};
use crate::indicator::rsi::{RSI, RSIBuilder};
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::fmt::Display;
use trading_chart::Candle;

// context에서 StrategyContextOps를 공개 가져오기
pub use super::context::StrategyContextOps;

/// RSI 전략 공통 설정
#[derive(Debug, Deserialize)]
pub struct RSIStrategyConfigBase {
    /// RSI 판단에 필요한 연속 데이터 수
    pub rsi_count: usize,
    /// RSI 하단 기준값 (매수 신호용)
    pub rsi_lower: f64,
    /// RSI 상단 기준값 (매도 신호용)
    pub rsi_upper: f64,
    /// RSI 계산 기간
    pub rsi_period: usize,
    /// 이동평균 유형 (EMA, SMA 등)
    pub ma: MAType,
    /// 이동평균 기간 목록 (여러 이동평균선 사용)
    pub ma_periods: Vec<usize>,
}

impl RSIStrategyConfigBase {
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
    /// * `Result<T, String>` - 로드된 설정 또는 오류
    pub fn from_json<T>(json: &str) -> Result<T, String>
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
    ) -> Result<RSIStrategyConfigBase, String> {
        // RSI 카운트 설정
        let rsi_count = match config.get("rsi_count") {
            Some(count) => count
                .parse::<usize>()
                .map_err(|_| "RSI 카운트 파싱 오류".to_string())?,
            None => return Err("rsi_count 설정이 필요합니다".to_string()),
        };

        // RSI 하단값 설정
        let rsi_lower = match config.get("rsi_lower") {
            Some(lower) => lower
                .parse::<f64>()
                .map_err(|_| "RSI 하단값 파싱 오류".to_string())?,
            None => return Err("rsi_lower 설정이 필요합니다".to_string()),
        };

        // RSI 상단값 설정
        let rsi_upper = match config.get("rsi_upper") {
            Some(upper) => upper
                .parse::<f64>()
                .map_err(|_| "RSI 상단값 파싱 오류".to_string())?,
            None => return Err("rsi_upper 설정이 필요합니다".to_string()),
        };

        // RSI 기간 설정
        let rsi_period = match config.get("rsi_period") {
            Some(period) => {
                let value = period
                    .parse::<usize>()
                    .map_err(|_| "RSI 기간 파싱 오류".to_string())?;

                if value < 2 {
                    return Err("RSI 기간은 2 이상이어야 합니다".to_string());
                }

                value
            }
            None => return Err("rsi_period 설정이 필요합니다".to_string()),
        };

        // MA 유형 설정
        let ma = match config.get("ma") {
            Some(ma_type) => match ma_type.to_lowercase().as_str() {
                "sma" => MAType::SMA,
                "ema" => MAType::EMA,
                _ => return Err(format!("알 수 없는 이동평균 유형: {}", ma_type)),
            },
            None => return Err("ma 설정이 필요합니다".to_string()),
        };

        // 이동평균 기간 설정
        let ma_periods = match config.get("ma_periods") {
            Some(periods) => {
                let periods_vec = crate::strategy::split_safe::<usize>(periods)
                    .map_err(|e| format!("이동평균 기간 파싱 오류: {}", e))?;

                if periods_vec.is_empty() {
                    return Err("이동평균 기간이 지정되지 않았습니다".to_string());
                }

                periods_vec
            }
            None => return Err("ma_periods 설정이 필요합니다".to_string()),
        };

        // 유효성 검사
        if rsi_lower >= rsi_upper {
            return Err(format!(
                "RSI 하단값({})이 상단값({})보다 크거나 같을 수 없습니다",
                rsi_lower, rsi_upper
            ));
        }

        let result = RSIStrategyConfigBase {
            rsi_count,
            rsi_lower,
            rsi_upper,
            rsi_period,
            ma,
            ma_periods,
        };

        result.validate()?;
        Ok(result)
    }
}

/// RSI 전략 데이터
#[derive(Debug)]
pub struct RSIStrategyData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// RSI 지표 값
    pub rsi: RSI,
    /// 이동평균선 집합
    pub mas: MAs,
}

impl<C: Candle> RSIStrategyData<C> {
    /// 새 전략 데이터 생성
    pub fn new(candle: C, mas: MAs, rsi: RSI) -> RSIStrategyData<C> {
        RSIStrategyData { candle, rsi, mas }
    }

    /// 이동평균이 정규 배열(오름차순)인지 확인
    pub fn is_ma_regular_arrangement(&self) -> bool {
        self.is_regular_arrangement(|data| &data.mas, |ma| ma.get())
    }

    /// 이동평균이 역배열(내림차순)인지 확인
    pub fn is_ma_reverse_arrangement(&self) -> bool {
        self.is_reverse_arrangement(|data| &data.mas, |ma| ma.get())
    }
}

impl<C: Candle> GetCandle<C> for RSIStrategyData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> StrategyDataOps<C> for RSIStrategyData<C> {}

/// RSI 전략 컨텍스트
#[derive(Debug)]
pub struct RSIStrategyContext<C: Candle> {
    /// RSI 빌더
    pub rsibuilder: RSIBuilder<C>,
    /// 이동평균 빌더
    pub masbuilder: MAsBuilder<C>,
    /// 전략 데이터 히스토리 (최신 데이터가 인덱스 0)
    pub items: Vec<RSIStrategyData<C>>,
}

impl<C: Candle> Display for RSIStrategyContext<C> {
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

impl<C: Candle + 'static> RSIStrategyContext<C> {
    /// 새 전략 컨텍스트 생성
    pub fn new(
        rsi_period: usize,
        ma_type: &MAType,
        ma_periods: &[usize],
        storage: &CandleStore<C>,
    ) -> RSIStrategyContext<C> {
        let rsibuilder = RSIBuilder::new(rsi_period);
        let masbuilder = MAsBuilderFactory::build::<C>(ma_type, ma_periods);
        let mut ctx = RSIStrategyContext {
            rsibuilder,
            masbuilder,
            items: vec![],
        };

        ctx.init(storage.get_reversed_items());
        ctx
    }

    /// n개의 연속 데이터에서 이동평균이 정규 배열인지 확인
    pub fn is_ma_regular_arrangement(&self, n: usize) -> bool {
        self.is_all(|data| data.is_ma_regular_arrangement(), n)
    }

    /// n개의 연속 데이터에서 이동평균이 역배열인지 확인
    pub fn is_ma_reverse_arrangement(&self, n: usize) -> bool {
        self.is_all(|data| data.is_ma_reverse_arrangement(), n)
    }
}

impl<C: Candle> StrategyContextOps<RSIStrategyData<C>, C> for RSIStrategyContext<C> {
    fn next_data(&mut self, candle: C) -> RSIStrategyData<C> {
        let rsi = self.rsibuilder.next(&candle);
        let mas = self.masbuilder.next(&candle);
        RSIStrategyData::new(candle, mas, rsi)
    }

    fn datum(&self) -> &Vec<RSIStrategyData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<RSIStrategyData<C>> {
        &mut self.items
    }
}

/// RSI 전략을 위한 공통 트레이트
pub trait RSIStrategyCommon<C: Candle + 'static>: Strategy<C> {
    /// 컨텍스트 참조 반환
    fn context(&self) -> &RSIStrategyContext<C>;

    /// 설정의 rsi_lower 반환
    fn config_rsi_lower(&self) -> f64;

    /// 설정의 rsi_upper 반환
    fn config_rsi_upper(&self) -> f64;

    /// 설정의 rsi_count 반환
    fn config_rsi_count(&self) -> usize;

    /// RSI가 과매수 영역인지 확인
    fn is_rsi_overbought(&self) -> bool {
        if self.context().items.len() < self.config_rsi_count() {
            return false;
        }

        // 과매수 판단: RSI가 상단 임계값을 넘어서면 과매수로 판단
        self.context()
            .items
            .iter()
            .take(self.config_rsi_count())
            .all(|item| item.rsi.rsi > self.config_rsi_upper())
    }

    /// RSI가 과매도 영역인지 확인
    fn is_rsi_oversold(&self) -> bool {
        if self.context().items.len() < self.config_rsi_count() {
            return false;
        }

        // 과매도 판단: RSI가 하단 임계값 아래로 내려가면 과매도로 판단
        self.context()
            .items
            .iter()
            .take(self.config_rsi_count())
            .all(|item| item.rsi.rsi < self.config_rsi_lower())
    }
}
