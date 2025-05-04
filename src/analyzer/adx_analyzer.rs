use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::adx::{ADXs, ADXsBuilder, ADXsBuilderFactory};
use std::fmt::Display;
use trading_chart::Candle;

/// ADX 전략 데이터
#[derive(Debug)]
pub struct ADXAnalyzerData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// ADX 지표 집합
    pub adxs: ADXs,
}

impl<C: Candle> ADXAnalyzerData<C> {
    /// 새 전략 데이터 생성
    pub fn new(candle: C, adxs: ADXs) -> ADXAnalyzerData<C> {
        ADXAnalyzerData { candle, adxs }
    }

    /// 특정 ADX 값 반환
    pub fn get_adx(&self, period: usize) -> f64 {
        self.adxs.get(&period).adx
    }

    /// 모든 ADX 값이 강한 추세(25 이상)인지 확인
    pub fn is_all_adx_strong_trend(&self) -> bool {
        self.adxs.get_all().iter().all(|adx| adx.adx >= 25.0)
    }

    /// 모든 ADX 값이 매우 강한 추세(50 이상)인지 확인
    pub fn is_all_adx_very_strong_trend(&self) -> bool {
        self.adxs.get_all().iter().all(|adx| adx.adx >= 50.0)
    }

    /// 모든 ADX 값이 약한 추세(25 미만)인지 확인
    pub fn is_all_adx_weak_trend(&self) -> bool {
        self.adxs.get_all().iter().all(|adx| adx.adx < 25.0)
    }
}

impl<C: Candle> GetCandle<C> for ADXAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for ADXAnalyzerData<C> {}

/// ADX 전략 컨텍스트
#[derive(Debug)]
pub struct ADXAnalyzer<C: Candle> {
    /// ADX 빌더
    pub adxsbuilder: ADXsBuilder<C>,
    /// 전략 데이터 히스토리 (최신 데이터가 인덱스 0)
    pub items: Vec<ADXAnalyzerData<C>>,
}

impl<C: Candle> Display for ADXAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(f, "캔들: {}, ADXs: {}", first.candle, first.adxs),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> ADXAnalyzer<C> {
    /// 새 전략 컨텍스트 생성
    pub fn new(adx_periods: &[usize], storage: &CandleStore<C>) -> ADXAnalyzer<C> {
        let adxsbuilder = ADXsBuilderFactory::build::<C>(adx_periods);
        let mut ctx = ADXAnalyzer {
            adxsbuilder,
            items: vec![],
        };
        ctx.init(storage.get_reversed_items());
        ctx
    }

    /// 특정 ADX 값 반환
    pub fn get_adx(&self, period: usize) -> f64 {
        match self.items.first() {
            Some(data) => data.get_adx(period),
            None => 0.0,
        }
    }

    /// 모든 ADX 값이 n개의 연속 데이터에서 강한 추세(25 이상)인지 확인
    pub fn is_strong_trend(&self, n: usize) -> bool {
        self.is_all(|data| data.is_all_adx_strong_trend(), n)
    }

    /// 모든 ADX 값이 n개의 연속 데이터에서 매우 강한 추세(50 이상)인지 확인
    pub fn is_very_strong_trend(&self, n: usize) -> bool {
        self.is_all(|data| data.is_all_adx_very_strong_trend(), n)
    }

    /// 모든 ADX 값이 n개의 연속 데이터에서 약한 추세(25 미만)인지 확인
    pub fn is_weak_trend(&self, n: usize) -> bool {
        self.is_all(|data| data.is_all_adx_weak_trend(), n)
    }

    /// 추세 강도가 증가하는지 확인 (현재 ADX가 이전 ADX보다 큰지)
    pub fn is_trend_strengthening(&self, period: usize, n: usize) -> bool {
        if self.items.len() < n + 1 {
            return false;
        }

        for i in 0..n {
            let current = self.items[i].get_adx(period);
            let previous = self.items[i + 1].get_adx(period);
            if current <= previous {
                return false;
            }
        }

        true
    }

    /// 추세 강도가 감소하는지 확인 (현재 ADX가 이전 ADX보다 작은지)
    pub fn is_trend_weakening(&self, period: usize, n: usize) -> bool {
        if self.items.len() < n + 1 {
            return false;
        }

        for i in 0..n {
            let current = self.items[i].get_adx(period);
            let previous = self.items[i + 1].get_adx(period);
            if current >= previous {
                return false;
            }
        }

        true
    }

    /// 추세 전환점 확인 (추세 강도가 약해졌다가 다시 강해지는 패턴)
    pub fn is_trend_reversal(&self, period: usize, n: usize, m: usize) -> bool {
        if self.items.len() < n + m + 1 {
            return false;
        }

        // 최근 n개 기간 동안 ADX 증가
        let is_increasing = (0..n).all(|i| {
            let current = self.items[i].get_adx(period);
            let previous = self.items[i + 1].get_adx(period);
            current > previous
        });

        // 이전 m개 기간 동안 ADX 감소
        let was_decreasing = (n..n + m).all(|i| {
            let current = self.items[i].get_adx(period);
            let previous = self.items[i + 1].get_adx(period);
            current < previous
        });

        is_increasing && was_decreasing
    }
}

impl<C: Candle> AnalyzerOps<ADXAnalyzerData<C>, C> for ADXAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> ADXAnalyzerData<C> {
        let adxs = self.adxsbuilder.next(&candle);
        ADXAnalyzerData::new(candle, adxs)
    }

    fn datum(&self) -> &Vec<ADXAnalyzerData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<ADXAnalyzerData<C>> {
        &mut self.items
    }
}
