use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::atr::{ATRs, ATRsBuilder, ATRsBuilderFactory};
use std::fmt::Display;
use trading_chart::Candle;

/// ATR 분석기 데이터
#[derive(Debug)]
pub struct ATRAnalyzerData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// ATR 지표 집합
    pub atrs: ATRs,
}

impl<C: Candle> ATRAnalyzerData<C> {
    /// 새 분석기 데이터 생성
    pub fn new(candle: C, atrs: ATRs) -> ATRAnalyzerData<C> {
        ATRAnalyzerData { candle, atrs }
    }

    /// 특정 ATR 값 반환
    pub fn get_atr(&self, period: usize) -> f64 {
        self.atrs.get(&period).value
    }
}

impl<C: Candle> GetCandle<C> for ATRAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for ATRAnalyzerData<C> {}

/// ATR 분석기
#[derive(Debug)]
pub struct ATRAnalyzer<C: Candle + 'static> {
    /// ATR 빌더
    pub atrsbuilder: ATRsBuilder<C>,
    /// 분석 데이터 히스토리
    pub items: Vec<ATRAnalyzerData<C>>,
}

impl<C: Candle + 'static> Display for ATRAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ATRAnalyzer {{ items: {} }}", self.items.len())
    }
}

impl<C: Candle + 'static> ATRAnalyzer<C> {
    /// 새 ATR 분석기 생성
    pub fn new(periods: &[usize], storage: &CandleStore<C>) -> ATRAnalyzer<C> {
        let atrsbuilder = ATRsBuilderFactory::build(periods);
        let mut analyzer = ATRAnalyzer {
            atrsbuilder,
            items: Vec::new(),
        };

        analyzer.init_from_storage(storage);
        analyzer
    }

    /// 저장소에서 초기 데이터 계산
    pub fn init_from_storage(&mut self, storage: &CandleStore<C>) {
        for candle in storage.get_time_ordered_items().iter() {
            self.next(candle.clone());
        }
    }

    /// 특정 기간과 배수로 ATR 상단값 계산
    pub fn calculate_upper_band(&self, candle: &C, period: usize, multiplier: f64) -> f64 {
        let atr = self
            .items
            .first()
            .map(|item| item.get_atr(period))
            .unwrap_or(0.0);
        let price = (candle.high_price() + candle.low_price()) / 2.0;
        price + (atr * multiplier)
    }

    /// 특정 기간과 배수로 ATR 하단값 계산
    pub fn calculate_lower_band(&self, candle: &C, period: usize, multiplier: f64) -> f64 {
        let atr = self
            .items
            .first()
            .map(|item| item.get_atr(period))
            .unwrap_or(0.0);
        let price = (candle.high_price() + candle.low_price()) / 2.0;
        price - (atr * multiplier)
    }

    /// 현재 ATR이 특정 임계값을 초과하는지 확인
    pub fn is_above_threshold(&self, period: usize, threshold: f64) -> bool {
        self.items
            .first()
            .map(|item| item.get_atr(period) > threshold)
            .unwrap_or(false)
    }

    /// 현재 ATR이 이전 n개 캔들의 평균 ATR보다 높은지 확인 (변동성 확대)
    pub fn is_volatility_expanding(&self, period: usize, n: usize) -> bool {
        if self.items.len() <= n {
            return false;
        }

        let current_atr = self
            .items
            .first()
            .map(|item| item.get_atr(period))
            .unwrap_or(0.0);
        let avg_atr: f64 = self.items[1..=n]
            .iter()
            .map(|item| item.get_atr(period))
            .sum::<f64>()
            / n as f64;

        current_atr > avg_atr
    }

    /// 현재 ATR이 이전 n개 캔들의 평균 ATR보다 낮은지 확인 (변동성 축소)
    pub fn is_volatility_contracting(&self, period: usize, n: usize) -> bool {
        if self.items.len() <= n {
            return false;
        }

        let current_atr = self
            .items
            .first()
            .map(|item| item.get_atr(period))
            .unwrap_or(0.0);
        let avg_atr: f64 = self.items[1..=n]
            .iter()
            .map(|item| item.get_atr(period))
            .sum::<f64>()
            / n as f64;

        current_atr < avg_atr
    }

    /// ATR 임계값 돌파 신호 확인 (n개 연속 ATR > 임계값, 이전 m개는 아님)
    pub fn is_atr_above_threshold_signal(
        &self,
        n: usize,
        m: usize,
        period: usize,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |data| {
                let atr = data.get_atr(period);
                atr > threshold
            },
            n,
            m,
            p,
        )
    }

    /// ATR 임계값 하향 돌파 신호 확인 (n개 연속 ATR < 임계값, 이전 m개는 아님)
    pub fn is_atr_below_threshold_signal(
        &self,
        n: usize,
        m: usize,
        period: usize,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |data| {
                let atr = data.get_atr(period);
                atr < threshold
            },
            n,
            m,
            p,
        )
    }

    /// 고변동성 신호 확인 (n개 연속 고변동성, 이전 m개는 아님)
    pub fn is_high_volatility_signal(
        &self,
        n: usize,
        m: usize,
        period: usize,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |data| {
                let atr = data.get_atr(period);
                atr > threshold
            },
            n,
            m,
            p,
        )
    }

    /// 저변동성 신호 확인 (n개 연속 저변동성, 이전 m개는 아님)
    pub fn is_low_volatility_signal(
        &self,
        n: usize,
        m: usize,
        period: usize,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |data| {
                let atr = data.get_atr(period);
                atr < threshold
            },
            n,
            m,
            p,
        )
    }

    /// 변동성 증가 신호 확인 (n개 연속 변동성 증가, 이전 m개는 아님)
    pub fn is_volatility_increasing_signal(
        &self,
        n: usize,
        m: usize,
        period: usize,
        p: usize,
    ) -> bool {
        if self.items.len() < n + m + p + 1 {
            return false;
        }

        let recent_increasing = self.is_volatility_increasing(n, period);
        if !recent_increasing {
            return false;
        }

        if m > 0 && p + n < self.items.len() {
            let previous_start = p + n;
            let previous_end = (previous_start + m).min(self.items.len());
            for i in previous_start..previous_end.saturating_sub(1) {
                if let (Some(current), Some(next)) = (self.items.get(i), self.items.get(i + 1)) {
                    let current_atr = current.get_atr(period);
                    let next_atr = next.get_atr(period);
                    if current_atr > next_atr {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// 변동성 감소 신호 확인 (n개 연속 변동성 감소, 이전 m개는 아님)
    pub fn is_volatility_decreasing_signal(
        &self,
        n: usize,
        m: usize,
        period: usize,
        p: usize,
    ) -> bool {
        if self.items.len() < n + m + p + 1 {
            return false;
        }

        let recent_decreasing = self.is_volatility_decreasing(n, period);
        if !recent_decreasing {
            return false;
        }

        if m > 0 && p + n < self.items.len() {
            let previous_start = p + n;
            let previous_end = (previous_start + m).min(self.items.len());
            for i in previous_start..previous_end.saturating_sub(1) {
                if let (Some(current), Some(next)) = (self.items.get(i), self.items.get(i + 1)) {
                    let current_atr = current.get_atr(period);
                    let next_atr = next.get_atr(period);
                    if current_atr < next_atr {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// n개의 연속 데이터에서 고변동성인지 확인
    ///
    /// # Arguments
    /// * `n` - 확인할 데이터 개수
    /// * `period` - ATR 계산 기간
    /// * `threshold` - 변동성 임계값
    /// * `p` - 과거 시점 확인을 위한 오프셋 (기본값: 0)
    ///
    /// # Returns
    /// * `bool` - n개 연속으로 고변동성이면 true
    ///
    pub fn is_high_volatility(&self, n: usize, period: usize, threshold: f64, p: usize) -> bool {
        self.is_all(
            |data| {
                let atr = data.get_atr(period);
                atr > threshold
            },
            n,
            p,
        )
    }

    /// n개의 연속 데이터에서 저변동성인지 확인
    ///
    /// # Arguments
    /// * `n` - 확인할 데이터 개수
    /// * `period` - ATR 계산 기간
    /// * `threshold` - 변동성 임계값
    /// * `p` - 과거 시점 확인을 위한 오프셋 (기본값: 0)
    ///
    /// # Returns
    /// * `bool` - n개 연속으로 저변동성이면 true
    ///
    pub fn is_low_volatility(&self, n: usize, period: usize, threshold: f64, p: usize) -> bool {
        self.is_all(
            |data| {
                let atr = data.get_atr(period);
                atr < threshold
            },
            n,
            p,
        )
    }

    /// n개의 연속 데이터에서 변동성이 증가하는지 확인
    pub fn is_volatility_increasing(&self, n: usize, period: usize) -> bool {
        if n > self.items.len() || n < 2 {
            return false;
        }
        for i in 0..n - 1 {
            if let (Some(current), Some(next)) = (self.items.get(i), self.items.get(i + 1)) {
                let current_atr = current.get_atr(period);
                let next_atr = next.get_atr(period);
                if current_atr <= next_atr {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    /// n개의 연속 데이터에서 변동성이 감소하는지 확인
    pub fn is_volatility_decreasing(&self, n: usize, period: usize) -> bool {
        if n > self.items.len() || n < 2 {
            return false;
        }
        for i in 0..n - 1 {
            if let (Some(current), Some(next)) = (self.items.get(i), self.items.get(i + 1)) {
                let current_atr = current.get_atr(period);
                let next_atr = next.get_atr(period);
                if current_atr >= next_atr {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}

impl<C: Candle + 'static> AnalyzerOps<ATRAnalyzerData<C>, C> for ATRAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> ATRAnalyzerData<C> {
        let atrs = self.atrsbuilder.next(&candle);
        ATRAnalyzerData::new(candle, atrs)
    }

    fn datum(&self) -> &Vec<ATRAnalyzerData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<ATRAnalyzerData<C>> {
        &mut self.items
    }
}
