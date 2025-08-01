use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::volume::{Volumes, VolumesBuilder, VolumesBuilderFactory};
use std::fmt::Display;
use trading_chart::Candle;

/// 볼륨 전략 데이터
#[derive(Debug)]
pub struct VolumeAnalyzerData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// 볼륨 지표 집합
    pub volumes: Volumes,
}

impl<C: Candle> VolumeAnalyzerData<C> {
    /// 새 전략 데이터 생성
    pub fn new(candle: C, volumes: Volumes) -> VolumeAnalyzerData<C> {
        VolumeAnalyzerData { candle, volumes }
    }

    /// 특정 기간의 볼륨 비율 확인
    pub fn get_volume_ratio(&self, period: usize) -> f64 {
        self.volumes.get(&period).volume_ratio
    }

    /// 모든 볼륨 비율이 기준(1.0) 이상인지 확인
    pub fn is_all_volume_ratio_above_average(&self) -> bool {
        self.volumes.get_all().iter().all(|v| v.volume_ratio >= 1.0)
    }

    /// 모든 볼륨 비율이 기준보다 크게 증가했는지 확인
    pub fn is_all_volume_ratio_significantly_above(&self, threshold: f64) -> bool {
        self.volumes
            .get_all()
            .iter()
            .all(|v| v.volume_ratio >= threshold)
    }

    /// 모든 볼륨 비율이 기준(1.0) 미만인지 확인
    pub fn is_all_volume_ratio_below_average(&self) -> bool {
        self.volumes.get_all().iter().all(|v| v.volume_ratio < 1.0)
    }

    /// 현재 볼륨이 평균 볼륨보다 많은지 확인
    pub fn is_current_volume_above_average(&self, period: usize) -> bool {
        let volume = self.volumes.get(&period);
        volume.current_volume > volume.average_volume
    }

    /// 현재 캔들이 양봉이고 볼륨이 증가했는지 확인
    pub fn is_bullish_with_increased_volume(&self, period: usize) -> bool {
        let is_bullish = self.candle.close_price() > self.candle.open_price();
        is_bullish && self.is_current_volume_above_average(period)
    }

    /// 현재 캔들이 음봉이고 볼륨이 증가했는지 확인
    pub fn is_bearish_with_increased_volume(&self, period: usize) -> bool {
        let is_bearish = self.candle.close_price() < self.candle.open_price();
        is_bearish && self.is_current_volume_above_average(period)
    }
}

impl<C: Candle> GetCandle<C> for VolumeAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for VolumeAnalyzerData<C> {}

/// 볼륨 전략 컨텍스트
#[derive(Debug)]
pub struct VolumeAnalyzer<C: Candle> {
    /// 볼륨 빌더
    pub volumesbuilder: VolumesBuilder<C>,
    /// 전략 데이터 히스토리 (최신 데이터가 인덱스 0)
    pub items: Vec<VolumeAnalyzerData<C>>,
}

impl<C: Candle> Display for VolumeAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(f, "캔들: {}, Volumes: {}", first.candle, first.volumes),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> VolumeAnalyzer<C> {
    /// 새 전략 컨텍스트 생성
    pub fn new(periods: &[usize], storage: &CandleStore<C>) -> VolumeAnalyzer<C> {
        let volumesbuilder = VolumesBuilderFactory::build::<C>(periods);
        let mut ctx = VolumeAnalyzer {
            volumesbuilder,
            items: vec![],
        };
        ctx.init_from_storage(storage);
        ctx
    }

    /// 기본 기간으로 새 전략 컨텍스트 생성
    pub fn default(storage: &CandleStore<C>) -> VolumeAnalyzer<C> {
        let periods = vec![10, 20, 50];
        Self::new(&periods, storage)
    }

    /// 평균 이상 볼륨 신호 확인 (n개 연속 평균 이상 볼륨, 이전 m개는 아님)
    pub fn is_volume_above_average_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.is_all_volume_ratio_above_average(),
            n,
            m,
            p,
        )
    }

    /// 평균 이하 볼륨 신호 확인 (n개 연속 평균 이하 볼륨, 이전 m개는 아님)
    pub fn is_volume_below_average_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.is_all_volume_ratio_below_average(),
            n,
            m,
            p,
        )
    }

    /// 볼륨 급증 신호 확인 (n개 연속 볼륨 급증, 이전 m개는 아님)
    pub fn is_volume_surge_signal(
        &self,
        n: usize,
        m: usize,
        period: usize,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |_| {
                if self.items.is_empty() {
                    return false;
                }

                let current_ratio = self.items[0].get_volume_ratio(period);
                current_ratio > threshold
            },
            n,
            m,
            p,
        )
    }

    /// 볼륨 감소 신호 확인 (n개 연속 볼륨 감소, 이전 m개는 아님)
    pub fn is_volume_decline_signal(
        &self,
        n: usize,
        m: usize,
        period: usize,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |_| {
                if self.items.is_empty() {
                    return false;
                }

                let current_ratio = self.items[0].get_volume_ratio(period);
                current_ratio < threshold
            },
            n,
            m,
            p,
        )
    }

    /// 특정 볼륨 비율 임계값 돌파 신호 확인 (n개 연속 임계값 초과, 이전 m개는 아님)
    pub fn is_volume_ratio_breakthrough(
        &self,
        n: usize,
        m: usize,
        period: usize,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.get_volume_ratio(period) > threshold,
            n,
            m,
            p,
        )
    }

    /// 강한 볼륨 신호 확인 (n개 연속 강한 볼륨, 이전 m개는 아님)
    pub fn is_significantly_above_volume_signal(
        &self,
        n: usize,
        m: usize,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.is_all_volume_ratio_significantly_above(threshold),
            n,
            m,
            p,
        )
    }

    /// 불리시 볼륨 증가 신호 확인 (n개 연속 양봉과 볼륨 증가, 이전 m개는 아님)
    pub fn is_bullish_with_increased_volume_signal(
        &self,
        n: usize,
        m: usize,
        period: usize,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.is_bullish_with_increased_volume(period),
            n,
            m,
            p,
        )
    }

    /// 베어리시 볼륨 증가 신호 확인 (n개 연속 음봉과 볼륨 증가, 이전 m개는 아님)
    pub fn is_bearish_with_increased_volume_signal(
        &self,
        n: usize,
        m: usize,
        period: usize,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.is_bearish_with_increased_volume(period),
            n,
            m,
            p,
        )
    }

    /// 상승 추세에서 볼륨 증가 신호 확인 (n개 연속 상승 추세 볼륨 증가, 이전 m개는 아님)
    pub fn is_increasing_volume_in_uptrend_signal(
        &self,
        n: usize,
        m: usize,
        period: usize,
        trend_period: usize,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |_| {
                // 연속적인 양봉에서 점진적으로 볼륨이 증가하는지 확인
                if self.items.len() < trend_period {
                    return false;
                }

                // 모두 양봉인지 확인
                for i in 0..trend_period {
                    if self.items[i].candle.close_price() <= self.items[i].candle.open_price() {
                        return false;
                    }
                }

                // 볼륨이 점진적으로 증가하는지 확인
                for i in 0..trend_period - 1 {
                    let current_ratio = self.items[i].get_volume_ratio(period);
                    let next_ratio = self.items[i + 1].get_volume_ratio(period);
                    if current_ratio <= next_ratio {
                        return false;
                    }
                }

                true
            },
            n,
            m,
            p,
        )
    }

    /// 하락 추세에서 볼륨 감소 신호 확인 (n개 연속 하락 추세 볼륨 감소, 이전 m개는 아님)
    pub fn is_decreasing_volume_in_downtrend_signal(
        &self,
        n: usize,
        m: usize,
        period: usize,
        trend_period: usize,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |_| {
                // 연속적인 음봉에서 점진적으로 볼륨이 감소하는지 확인
                if self.items.len() < trend_period {
                    return false;
                }

                // 모두 음봉인지 확인
                for i in 0..trend_period {
                    if self.items[i].candle.close_price() >= self.items[i].candle.open_price() {
                        return false;
                    }
                }

                // 볼륨이 점진적으로 감소하는지 확인
                for i in 0..trend_period - 1 {
                    let current_ratio = self.items[i].get_volume_ratio(period);
                    let next_ratio = self.items[i + 1].get_volume_ratio(period);
                    if current_ratio >= next_ratio {
                        return false;
                    }
                }

                true
            },
            n,
            m,
            p,
        )
    }

    /// 볼륨 급증 여부 확인 (이전 대비 갑자기 높은 볼륨)
    pub fn is_volume_surge(&self, period: usize, threshold: f64) -> bool {
        if self.items.len() < 2 {
            return false;
        }

        let current_ratio = self.items[0].get_volume_ratio(period);
        let previous_ratio = self.items[1].get_volume_ratio(period);

        current_ratio > threshold && current_ratio > previous_ratio * 1.5
    }

    /// 볼륨 급감 여부 확인 (이전 대비 갑자기 낮은 볼륨)
    pub fn is_volume_decline(&self, period: usize, threshold: f64) -> bool {
        if self.items.len() < 2 {
            return false;
        }

        let current_ratio = self.items[0].get_volume_ratio(period);
        let previous_ratio = self.items[1].get_volume_ratio(period);

        current_ratio < threshold && current_ratio < previous_ratio * 0.5
    }

    /// 연속적인 양봉에서 점진적으로 볼륨이 증가하는지 확인
    pub fn is_increasing_volume_in_uptrend(&self, period: usize, n: usize) -> bool {
        if self.items.len() < n {
            return false;
        }

        // 모두 양봉인지 확인
        for i in 0..n {
            if self.items[i].candle.close_price() <= self.items[i].candle.open_price() {
                return false;
            }
        }

        // 볼륨이 점진적으로 증가하는지 확인
        for i in 0..n - 1 {
            let current_ratio = self.items[i].get_volume_ratio(period);
            let next_ratio = self.items[i + 1].get_volume_ratio(period);
            if current_ratio <= next_ratio {
                return false;
            }
        }

        true
    }

    /// 연속적인 음봉에서 점진적으로 볼륨이 감소하는지 확인
    pub fn is_decreasing_volume_in_downtrend(&self, period: usize, n: usize) -> bool {
        if self.items.len() < n {
            return false;
        }

        // 모두 음봉인지 확인
        for i in 0..n {
            if self.items[i].candle.close_price() >= self.items[i].candle.open_price() {
                return false;
            }
        }

        // 볼륨이 점진적으로 감소하는지 확인
        for i in 0..n - 1 {
            let current_ratio = self.items[i].get_volume_ratio(period);
            let next_ratio = self.items[i + 1].get_volume_ratio(period);
            if current_ratio >= next_ratio {
                return false;
            }
        }

        true
    }

    /// 현재 볼륨이 평균 이상인 신호 확인 (n개 연속 현재 볼륨 > 평균, 이전 m개는 아님)
    pub fn is_current_volume_above_average_signal(
        &self,
        n: usize,
        m: usize,
        period: usize,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.is_current_volume_above_average(period),
            n,
            m,
            p,
        )
    }

    /// n개의 연속 데이터에서 볼륨이 평균 이상인지 확인
    pub fn is_volume_above_average(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_all_volume_ratio_above_average(), n, p)
    }

    /// n개의 연속 데이터에서 볼륨이 평균 이하인지 확인
    pub fn is_volume_below_average(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.is_all_volume_ratio_below_average(), n, p)
    }

    /// n개의 연속 데이터에서 볼륨이 임계값 이상인지 확인
    pub fn is_volume_significantly_above(&self, n: usize, threshold: f64, p: usize) -> bool {
        self.is_all(
            |data| data.is_all_volume_ratio_significantly_above(threshold),
            n,
            p,
        )
    }

    /// n개의 연속 데이터에서 불리시 볼륨 증가인지 확인
    pub fn is_bullish_with_increased_volume(&self, n: usize, period: usize, p: usize) -> bool {
        self.is_all(|data| data.is_bullish_with_increased_volume(period), n, p)
    }

    /// n개의 연속 데이터에서 베어리시 볼륨 증가인지 확인
    pub fn is_bearish_with_increased_volume(&self, n: usize, period: usize, p: usize) -> bool {
        self.is_all(|data| data.is_bearish_with_increased_volume(period), n, p)
    }
}

impl<C: Candle> AnalyzerOps<VolumeAnalyzerData<C>, C> for VolumeAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> VolumeAnalyzerData<C> {
        let volumes = self.volumesbuilder.next(&candle);
        VolumeAnalyzerData::new(candle, volumes)
    }

    fn datum(&self) -> &Vec<VolumeAnalyzerData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<VolumeAnalyzerData<C>> {
        &mut self.items
    }
}
