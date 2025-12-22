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
        ctx.init_from_storage(storage);
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
    pub fn is_strong_trend(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_all_adx_strong_trend(), n, p)
    }

    /// 모든 ADX 값이 n개의 연속 데이터에서 매우 강한 추세(50 이상)인지 확인
    pub fn is_very_strong_trend(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_all_adx_very_strong_trend(), n, p)
    }

    /// 모든 ADX 값이 n개의 연속 데이터에서 약한 추세(25 미만)인지 확인
    pub fn is_weak_trend(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_all_adx_weak_trend(), n, p)
    }

    /// 추세 강도가 증가하는지 확인 (현재 ADX가 이전 ADX보다 큰지)
    pub fn is_trend_strengthening(&self, period: usize, n: usize) -> bool {
        if self.items.len() < n + 1 {
            return false;
        }

        // 최근 n+1개 기간 동안의 ADX와 +DI, -DI 값들
        let mut adx_values = Vec::new();
        let mut pdi_values = Vec::new();
        let mut ndi_values = Vec::new();

        for item in self.items.iter().take(n + 1) {
            let adx = item.adxs.get(&period).adx;
            let pdi = item.adxs.get(&period).plus_di;
            let ndi = item.adxs.get(&period).minus_di;
            adx_values.push(adx);
            pdi_values.push(pdi);
            ndi_values.push(ndi);
        }

        // ADX 증가 추세 또는 높은 값에서 안정적 유지 확인
        // ADX가 이미 매우 높은 값(80 이상)인 경우, 작은 감소에도 불구하고 높은 값을 유지하는 것도 추세 강화로 간주
        let mut adx_increasing = true;
        let mut adx_stable_high = false;

        // 평균 ADX 값 계산
        let avg_adx: f64 = adx_values.iter().sum::<f64>() / adx_values.len() as f64;

        // ADX가 매우 높은 값에서 시작하는 경우 (80 이상)
        if avg_adx >= 80.0 {
            // 높은 값에서 안정적으로 유지되는지 확인 (변동폭이 작고 최소값이 80 이상)
            let min_adx = adx_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_adx = adx_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let adx_range = max_adx - min_adx;

            // 높은 값에서 안정적으로 유지되거나 (변동폭이 작음) 또는 증가하는 경우
            adx_stable_high =
                min_adx >= 80.0 && (adx_range < 5.0 || adx_values[0] >= adx_values[n]);
        } else {
            // 일반적인 경우: ADX가 증가하는지 확인
            for i in 1..=n {
                // 현재 값이 이전 값보다 작고, 이전 값이 100이 아닌 경우 증가 추세가 아님
                if adx_values[i] < adx_values[i - 1] && adx_values[i - 1] < 100.0 {
                    adx_increasing = false;
                    break;
                }
            }
        }

        // +DI와 -DI의 상대적 강도 확인
        let mut di_strength = true;
        for i in 0..=n {
            if pdi_values[i] <= ndi_values[i] {
                di_strength = false;
                break;
            }
        }

        // ADX 최소값 확인 (25 이상)
        let adx_strong = adx_values.iter().all(|&adx| adx >= 25.0);

        // 모든 조건을 만족해야 추세 강화로 판단
        // 높은 값에서 안정적으로 유지되는 경우도 추세 강화로 간주
        (adx_increasing || adx_stable_high) && di_strength && adx_strong
    }

    /// 추세 강도가 감소하는지 확인 (현재 ADX가 이전 ADX보다 작은지)
    pub fn is_trend_weakening(&self, period: usize, n: usize) -> bool {
        if self.items.len() < n + 1 {
            return false;
        }

        // 최근 n개 기간 동안의 ADX 값들
        let adx_values: Vec<f64> = self
            .items
            .iter()
            .take(n + 1)
            .map(|data| data.get_adx(period))
            .collect();

        // 연속된 감소 횟수 확인
        let mut decreasing_count = 0;
        for i in 0..n {
            if adx_values[i] > adx_values[i + 1] {
                decreasing_count += 1;
            }
        }

        // 첫 번째와 마지막 값의 차이로 전체적인 감소 추세 확인
        let total_change = adx_values[n] - adx_values[0];

        // n개 중 최소 50% 이상이 감소하고, 전체적으로도 감소했다면 추세 약화로 판단
        decreasing_count >= (n as f64 * 0.5).ceil() as usize && total_change > 0.0
    }

    /// 추세 전환점 확인 (추세 강도가 약해졌다가 다시 강해지는 패턴)
    pub fn is_trend_reversal(&self, period: usize, n: usize, m: usize) -> bool {
        if self.items.len() < n + m + 1 {
            return false;
        }

        // 최근 n개 기간 동안의 ADX 값들
        let recent_adx: Vec<f64> = self
            .items
            .iter()
            .take(n)
            .map(|data| data.get_adx(period))
            .collect();

        // 이전 m개 기간 동안의 ADX 값들
        let previous_adx: Vec<f64> = self
            .items
            .iter()
            .skip(n)
            .take(m)
            .map(|data| data.get_adx(period))
            .collect();

        // 최근 n개 기간의 평균 ADX
        let recent_avg = recent_adx.iter().sum::<f64>() / n as f64;

        // 이전 m개 기간의 평균 ADX
        let previous_avg = previous_adx.iter().sum::<f64>() / m as f64;

        // 최근 ADX 값의 변화율 계산
        let recent_change = (recent_adx[0] - recent_adx[n - 1]) / recent_adx[n - 1] * 100.0;

        // 이전 ADX 값의 변화율 계산
        let previous_change = (previous_adx[0] - previous_adx[m - 1]) / previous_adx[m - 1] * 100.0;

        // ADX 최소값 확인 (20 이상)
        let recent_min_adx = recent_adx.iter().fold(f64::INFINITY, |a, &b| a.min(b));

        // 조건:
        // 1. 최근 ADX 평균이 이전 ADX 평균보다 높음
        // 2. 최근 ADX가 이전 ADX보다 증가하는 추세를 보임 (변화율 기준)
        // 3. 이전 ADX가 증가하는 추세를 보였음 (변화율 기준)
        // 4. 최근 ADX의 최소값이 20 이상
        recent_avg > previous_avg
            && recent_change > 0.0
            && previous_change > 0.0
            && recent_min_adx >= 20.0
    }

    /// n개의 연속 데이터에서 ADX가 횡보 상태인지 확인
    pub fn is_adx_sideways(&self, period: usize, n: usize, p: usize, threshold: f64) -> bool {
        self.is_sideways(
            |data: &ADXAnalyzerData<C>| data.get_adx(period),
            n,
            p,
            threshold,
        )
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
