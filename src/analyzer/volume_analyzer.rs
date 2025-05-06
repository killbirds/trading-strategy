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

    /// n개의 연속 데이터에서 모든 볼륨 비율이 평균 이상인지 확인
    pub fn is_volume_above_average(&self, n: usize) -> bool {
        self.is_all(|data| data.is_all_volume_ratio_above_average(), n)
    }

    /// n개의 연속 데이터에서 모든 볼륨 비율이 지정된 임계값 이상인지 확인
    pub fn is_volume_significantly_above(&self, threshold: f64, n: usize) -> bool {
        self.is_all(
            |data| data.is_all_volume_ratio_significantly_above(threshold),
            n,
        )
    }

    /// n개의 연속 데이터에서 모든 볼륨 비율이 평균 미만인지 확인
    pub fn is_volume_below_average(&self, n: usize) -> bool {
        self.is_all(|data| data.is_all_volume_ratio_below_average(), n)
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

    /// 양봉 볼륨 증가 패턴 확인
    pub fn is_bullish_with_increased_volume(&self, period: usize, n: usize) -> bool {
        self.is_all(|data| data.is_bullish_with_increased_volume(period), n)
    }

    /// 음봉 볼륨 증가 패턴 확인
    pub fn is_bearish_with_increased_volume(&self, period: usize, n: usize) -> bool {
        self.is_all(|data| data.is_bearish_with_increased_volume(period), n)
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
