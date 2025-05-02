use super::Strategy;
use super::context::{GetCandle, StrategyContextOps, StrategyDataOps};
use crate::indicator::bband::{BBand, BBandBuilder};
use crate::indicator::ma::{MAs, MAsBuilder};
use crate::indicator::rsi::{RSI, RSIBuilder};
use serde::Deserialize;
use serde_json;
use std::fmt::Display;
use trading_chart::Candle;

/// Copys 전략 공통 설정 기본 구조체
#[derive(Debug, Deserialize)]
pub struct CopysStrategyConfigBase {
    /// RSI 계산 기간
    pub rsi_period: usize,
    /// RSI 상한값
    pub rsi_upper: f64,
    /// RSI 하한값
    pub rsi_lower: f64,
    /// 볼린저밴드 계산 기간
    pub bband_period: usize,
    /// 볼린저밴드 표준편차 승수
    pub bband_multiplier: f64,
}

impl CopysStrategyConfigBase {
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

        if self.bband_period < 2 {
            return Err("볼린저밴드 기간은 2 이상이어야 합니다".to_string());
        }

        if self.bband_multiplier <= 0.0 {
            return Err("볼린저밴드 승수는 0보다 커야 합니다".to_string());
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
}

/// Copys 전략 데이터
#[derive(Debug)]
pub struct CopysStrategyData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// RSI 지표 값
    pub rsi: RSI,
    /// 이동평균선 집합
    pub mas: MAs,
    /// 볼린저밴드 지표 값
    pub bband: BBand,
}

impl<C: Candle> CopysStrategyData<C> {
    /// 새 전략 데이터 생성
    pub fn new(candle: C, rsi: RSI, mas: MAs, bband: BBand) -> CopysStrategyData<C> {
        CopysStrategyData {
            candle,
            rsi,
            mas,
            bband,
        }
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

impl<C: Candle> GetCandle<C> for CopysStrategyData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> StrategyDataOps<C> for CopysStrategyData<C> {}

/// Copys 전략 컨텍스트
#[derive(Debug)]
pub struct CopysStrategyContext<C: Candle> {
    /// RSI 빌더
    pub rsibuilder: RSIBuilder<C>,
    /// 이동평균 빌더
    pub masbuilder: MAsBuilder<C>,
    /// 볼린저밴드 빌더
    pub bbandbuilder: BBandBuilder<C>,
    /// 전략 데이터 히스토리 (최신 데이터가 인덱스 0)
    pub items: Vec<CopysStrategyData<C>>,
}

impl<C: Candle> Display for CopysStrategyContext<C> {
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

impl<C: Candle> StrategyContextOps<CopysStrategyData<C>, C> for CopysStrategyContext<C> {
    fn next_data(&mut self, candle: C) -> CopysStrategyData<C> {
        let rsi = self.rsibuilder.next(&candle);
        let mas = self.masbuilder.next(&candle);
        let bband = self.bbandbuilder.next(&candle);
        CopysStrategyData::new(candle, rsi, mas, bband)
    }

    fn datum(&self) -> &Vec<CopysStrategyData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<CopysStrategyData<C>> {
        &mut self.items
    }
}

impl<C: Candle + 'static> CopysStrategyContext<C> {
    /// 새 전략 컨텍스트 생성
    pub fn new(
        rsi_period: usize,
        ma_builder: MAsBuilder<C>,
        bband_builder: BBandBuilder<C>,
    ) -> Self {
        CopysStrategyContext {
            rsibuilder: RSIBuilder::new(rsi_period),
            masbuilder: ma_builder,
            bbandbuilder: bband_builder,
            items: vec![],
        }
    }

    /// n개의 연속 데이터에서 이동평균이 정규 배열인지 확인
    pub fn is_ma_regular_arrangement(&self, n: usize) -> bool {
        self.is_all(|data| data.is_ma_regular_arrangement(), n)
    }

    /// n개의 연속 데이터에서 이동평균이 역배열인지 확인
    pub fn is_ma_reverse_arrangement(&self, n: usize) -> bool {
        self.is_all(|data| data.is_ma_reverse_arrangement(), n)
    }

    /// 조건을 만족하는 돌파 패턴 확인
    pub fn is_break_through_by_satisfying<F>(&self, is_fn: F, n: usize, m: usize) -> bool
    where
        F: Fn(&CopysStrategyData<C>) -> bool + Copy,
    {
        if self.items.len() < n + m {
            false
        } else {
            let (heads, tails) = self.items.split_at(n);
            let result = heads.iter().all(is_fn);
            result && tails.iter().take(m).all(|data| !is_fn(data))
        }
    }
}

/// Copys 전략 공통 트레이트
pub trait CopysStrategyCommon<C: Candle + 'static>: Strategy<C> {
    /// 컨텍스트 참조 반환
    fn context(&self) -> &CopysStrategyContext<C>;

    /// 설정의 rsi_lower 반환
    fn config_rsi_lower(&self) -> f64;

    /// 설정의 rsi_upper 반환
    fn config_rsi_upper(&self) -> f64;

    /// RSI 판정 횟수 반환
    fn config_rsi_count(&self) -> usize;
}
