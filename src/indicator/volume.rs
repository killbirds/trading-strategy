use crate::candle_store::CandleStore;
use crate::indicator::{TABuilder, TAs, TAsBuilder};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

/// 볼륨 기반 지표 빌더
///
/// 특정 기간의 거래량 데이터를 기반으로 한 지표를 계산합니다.
#[derive(Debug)]
pub struct VolumeBuilder<C: Candle> {
    /// 계산 기간
    period: usize,
    /// 누적 거래량
    accumulated_volume: f64,
    /// 데이터 저장 버퍼
    data_buffer: Vec<f64>,
    _phantom: PhantomData<C>,
}

/// 볼륨 분석 결과
#[derive(Clone, Debug)]
pub struct Volume {
    /// 볼륨 계산 기간
    period: usize,
    /// 평균 거래량
    pub average_volume: f64,
    /// 현재 거래량
    pub current_volume: f64,
    /// 볼륨 비율 (현재/평균)
    pub volume_ratio: f64,
}

impl Display for Volume {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Volume({}: avg={:.2}, current={:.2}, ratio={:.2})",
            self.period, self.average_volume, self.current_volume, self.volume_ratio
        )
    }
}

impl<C> VolumeBuilder<C>
where
    C: Candle,
{
    /// 새 볼륨 빌더 생성
    ///
    /// # Arguments
    /// * `period` - 볼륨 계산 기간
    ///
    /// # Returns
    /// * `VolumeBuilder` - 새 빌더 인스턴스
    pub fn new(period: usize) -> Self {
        if period == 0 {
            panic!("볼륨 계산 기간은 0보다 커야 합니다");
        }

        VolumeBuilder {
            period,
            accumulated_volume: 0.0,
            data_buffer: Vec::with_capacity(period),
            _phantom: PhantomData,
        }
    }

    /// 저장소에서 볼륨 지표 생성
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    ///
    /// # Returns
    /// * `Volume` - 계산된 볼륨 지표
    pub fn from_storage(&mut self, storage: &CandleStore<C>) -> Volume {
        self.build(&storage.get_reversed_items())
    }

    /// 데이터 벡터에서 볼륨 지표 생성
    ///
    /// # Arguments
    /// * `data` - 캔들 데이터 벡터
    ///
    /// # Returns
    /// * `Volume` - 계산된 볼륨 지표
    pub fn build(&mut self, data: &[C]) -> Volume {
        // 데이터 버퍼 리셋
        self.data_buffer.clear();
        self.accumulated_volume = 0.0;

        if data.is_empty() {
            return Volume {
                period: self.period,
                average_volume: 0.0,
                current_volume: 0.0,
                volume_ratio: 1.0, // 기본값
            };
        }

        // 최대 period 개수만큼만 처리
        let slice_start = if data.len() > self.period {
            data.len() - self.period
        } else {
            0
        };

        for candle in &data[slice_start..] {
            self.data_buffer.push(candle.acc_trade_volume());
            self.accumulated_volume += candle.acc_trade_volume();
        }

        let current_volume = if let Some(last) = data.last() {
            last.acc_trade_volume()
        } else {
            0.0
        };

        let average_volume = if !self.data_buffer.is_empty() {
            self.accumulated_volume / self.data_buffer.len() as f64
        } else {
            0.0
        };

        let volume_ratio = if average_volume > 0.0 {
            current_volume / average_volume
        } else {
            1.0
        };

        Volume {
            period: self.period,
            average_volume,
            current_volume,
            volume_ratio,
        }
    }

    /// 새 캔들 데이터로 볼륨 지표 업데이트
    ///
    /// # Arguments
    /// * `data` - 새 캔들 데이터
    ///
    /// # Returns
    /// * `Volume` - 업데이트된 볼륨 지표
    pub fn next(&mut self, data: &C) -> Volume {
        if self.data_buffer.len() >= self.period {
            // 가장 오래된 데이터 제거
            if let Some(oldest) = self.data_buffer.first().cloned() {
                self.accumulated_volume -= oldest;
            }
            self.data_buffer.remove(0);
        }

        let current_volume = data.acc_trade_volume();
        self.data_buffer.push(current_volume);
        self.accumulated_volume += current_volume;

        let average_volume = if !self.data_buffer.is_empty() {
            self.accumulated_volume / self.data_buffer.len() as f64
        } else {
            0.0
        };

        let volume_ratio = if average_volume > 0.0 {
            current_volume / average_volume
        } else {
            1.0
        };

        Volume {
            period: self.period,
            average_volume,
            current_volume,
            volume_ratio,
        }
    }
}

impl<C> TABuilder<Volume, C> for VolumeBuilder<C>
where
    C: Candle,
{
    fn from_storage(&mut self, storage: &CandleStore<C>) -> Volume {
        self.from_storage(storage)
    }

    fn build(&mut self, data: &[C]) -> Volume {
        self.build(data)
    }

    fn next(&mut self, data: &C) -> Volume {
        self.next(data)
    }
}

/// 여러 기간의 볼륨 지표 컬렉션 타입
pub type Volumes = TAs<usize, Volume>;

/// 여러 기간의 볼륨 지표 빌더 타입
pub type VolumesBuilder<C> = TAsBuilder<usize, Volume, C>;

/// 볼륨 컬렉션 빌더 팩토리
pub struct VolumesBuilderFactory;

impl VolumesBuilderFactory {
    /// 여러 기간의 볼륨 빌더 생성
    ///
    /// # Arguments
    /// * `periods` - 볼륨 계산 기간 목록
    ///
    /// # Returns
    /// * `VolumesBuilder` - 여러 기간의 볼륨 빌더
    pub fn build<C: Candle + 'static>(periods: &[usize]) -> VolumesBuilder<C> {
        VolumesBuilder::new("volumes".to_owned(), periods, |period| {
            Box::new(VolumeBuilder::<C>::new(*period))
        })
    }

    /// 기본 볼륨 빌더 생성 (20기간)
    ///
    /// # Returns
    /// * `VolumesBuilder` - 기본 볼륨 빌더
    pub fn build_default<C: Candle + 'static>() -> VolumesBuilder<C> {
        let default_periods = vec![20];
        Self::build(&default_periods)
    }

    /// 일반적인 볼륨 빌더 세트 생성 (10, 20, 50 기간)
    ///
    /// # Returns
    /// * `VolumesBuilder` - 일반적인 기간 세트의 볼륨 빌더
    pub fn build_common<C: Candle + 'static>() -> VolumesBuilder<C> {
        let common_periods = vec![10, 20, 50];
        Self::build(&common_periods)
    }
}
