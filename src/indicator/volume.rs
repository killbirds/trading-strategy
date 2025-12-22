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

impl Volume {
    /// 볼륨 기간 반환
    ///
    /// # Returns
    /// * `usize` - 볼륨 계산 기간
    pub fn period(&self) -> usize {
        self.period
    }

    /// 고거래량 상태인지 확인
    ///
    /// # Arguments
    /// * `threshold` - 고거래량 기준값 (기본값 1.5)
    ///
    /// # Returns
    /// * `bool` - 고거래량 여부
    pub fn is_high_volume(&self, threshold: Option<f64>) -> bool {
        let threshold_value = threshold.unwrap_or(1.5);
        self.volume_ratio >= threshold_value
    }

    /// 저거래량 상태인지 확인
    ///
    /// # Arguments
    /// * `threshold` - 저거래량 기준값 (기본값 0.5)
    ///
    /// # Returns
    /// * `bool` - 저거래량 여부
    pub fn is_low_volume(&self, threshold: Option<f64>) -> bool {
        let threshold_value = threshold.unwrap_or(0.5);
        self.volume_ratio <= threshold_value
    }

    /// 평균 거래량 이상인지 확인
    ///
    /// # Returns
    /// * `bool` - 평균 이상 여부
    pub fn is_above_average(&self) -> bool {
        self.volume_ratio >= 1.0
    }

    /// 평균 거래량 이하인지 확인
    ///
    /// # Returns
    /// * `bool` - 평균 이하 여부
    pub fn is_below_average(&self) -> bool {
        self.volume_ratio <= 1.0
    }
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
            data_buffer: Vec::with_capacity(period * 2),
            _phantom: PhantomData,
        }
    }

    /// 평균 거래량 계산
    ///
    /// # Arguments
    /// * `count` - 데이터 개수
    ///
    /// # Returns
    /// * `f64` - 평균 거래량
    fn calculate_average_volume(&self, count: usize) -> f64 {
        if count > 0 {
            self.accumulated_volume / count as f64
        } else {
            0.0
        }
    }

    /// 볼륨 비율 계산
    ///
    /// # Arguments
    /// * `current_volume` - 현재 거래량
    /// * `average_volume` - 평균 거래량
    ///
    /// # Returns
    /// * `f64` - 볼륨 비율
    fn calculate_volume_ratio(&self, current_volume: f64, average_volume: f64) -> f64 {
        if average_volume > 0.0 {
            current_volume / average_volume
        } else {
            1.0
        }
    }

    /// Volume 구조체 생성
    ///
    /// # Arguments
    /// * `current_volume` - 현재 거래량
    ///
    /// # Returns
    /// * `Volume` - 생성된 볼륨 지표
    fn create_volume(&self, current_volume: f64) -> Volume {
        let average_volume = self.calculate_average_volume(self.data_buffer.len());
        let volume_ratio = self.calculate_volume_ratio(current_volume, average_volume);

        Volume {
            period: self.period,
            average_volume,
            current_volume,
            volume_ratio,
        }
    }

    /// 저장소에서 볼륨 지표 생성
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    ///
    /// # Returns
    /// * `Volume` - 계산된 볼륨 지표
    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> Volume {
        self.build(&storage.get_time_ordered_items())
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
                volume_ratio: 1.0,
            };
        }

        // 최대 period 개수만큼만 처리
        let slice_start = if data.len() > self.period {
            data.len() - self.period
        } else {
            0
        };

        // 필요한 데이터만 버퍼에 추가 (next() 호출을 위해)
        for candle in &data[slice_start..] {
            let volume = candle.volume();
            self.data_buffer.push(volume);
            self.accumulated_volume += volume;
        }

        let current_volume = data.last().map(|c| c.volume()).unwrap_or(0.0);
        self.create_volume(current_volume)
    }

    /// 새 캔들 데이터로 볼륨 지표 업데이트
    ///
    /// # Arguments
    /// * `data` - 새 캔들 데이터
    ///
    /// # Returns
    /// * `Volume` - 업데이트된 볼륨 지표
    pub fn next(&mut self, data: &C) -> Volume {
        let current_volume = data.volume();

        // period를 초과하는 경우 가장 오래된 데이터 제거
        if self.data_buffer.len() >= self.period {
            if let Some(oldest) = self.data_buffer.first().copied() {
                self.accumulated_volume -= oldest;
            }
            self.data_buffer.drain(0..1);
        }

        // 새 데이터 추가
        self.data_buffer.push(current_volume);
        self.accumulated_volume += current_volume;

        // 버퍼 크기 제한 (period * 2로 제한하여 효율성 유지)
        if self.data_buffer.len() > self.period * 2 {
            let excess = self.data_buffer.len() - self.period * 2;
            for &volume in &self.data_buffer[..excess] {
                self.accumulated_volume -= volume;
            }
            self.data_buffer.drain(0..excess);
        }

        self.create_volume(current_volume)
    }
}

impl<C> TABuilder<Volume, C> for VolumeBuilder<C>
where
    C: Candle,
{
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> Volume {
        self.build_from_storage(storage)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestCandle;
    use chrono::Utc;

    fn create_test_candles() -> Vec<TestCandle> {
        vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 110.0,
                low: 90.0,
                close: 105.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 105.0,
                high: 115.0,
                low: 95.0,
                close: 110.0,
                volume: 1100.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 110.0,
                high: 120.0,
                low: 100.0,
                close: 115.0,
                volume: 1200.0,
            },
        ]
    }

    #[test]
    fn test_volume_builder_new() {
        let builder = VolumeBuilder::<TestCandle>::new(20);
        assert_eq!(builder.period, 20);
    }

    #[test]
    #[should_panic(expected = "볼륨 계산 기간은 0보다 커야 합니다")]
    fn test_volume_builder_new_invalid_period() {
        VolumeBuilder::<TestCandle>::new(0);
    }

    #[test]
    fn test_volume_build_empty_data() {
        let mut builder = VolumeBuilder::<TestCandle>::new(20);
        let volume = builder.build(&[]);
        assert_eq!(volume.period, 20);
        assert_eq!(volume.average_volume, 0.0);
        assert_eq!(volume.current_volume, 0.0);
        assert_eq!(volume.volume_ratio, 1.0); // 기본값
    }

    #[test]
    fn test_volume_build_with_data() {
        let mut builder = VolumeBuilder::<TestCandle>::new(3);
        let candles = create_test_candles();
        let volume = builder.build(&candles);

        assert_eq!(volume.period, 3);
        assert_eq!(volume.average_volume, 1100.0); // (1000 + 1100 + 1200) / 3
        assert_eq!(volume.current_volume, 1200.0); // 마지막 캔들의 거래량
        assert!(volume.volume_ratio > 1.0); // 현재 거래량이 평균보다 높음
    }

    #[test]
    fn test_volume_next() {
        let mut builder = VolumeBuilder::<TestCandle>::new(3);
        let candles = create_test_candles();
        let volume = builder.next(&candles[0]);

        assert_eq!(volume.period, 3);
        assert_eq!(volume.average_volume, 1000.0); // 첫 번째 캔들만 있음
        assert_eq!(volume.current_volume, 1000.0);
        assert_eq!(volume.volume_ratio, 1.0); // 현재 = 평균
    }

    #[test]
    fn test_volume_display() {
        let volume = Volume {
            period: 20,
            average_volume: 1000.0,
            current_volume: 1200.0,
            volume_ratio: 1.2,
        };

        assert_eq!(
            format!("{volume}"),
            "Volume(20: avg=1000.00, current=1200.00, ratio=1.20)"
        );
    }

    #[test]
    fn test_volume_data_buffer() {
        let mut builder = VolumeBuilder::<TestCandle>::new(2);
        let candles = create_test_candles();

        // 첫 번째 데이터
        let vol1 = builder.next(&candles[0]);
        assert_eq!(vol1.average_volume, 1000.0);
        assert_eq!(vol1.current_volume, 1000.0);

        // 두 번째 데이터
        let vol2 = builder.next(&candles[1]);
        assert_eq!(vol2.average_volume, 1050.0); // (1000 + 1100) / 2
        assert_eq!(vol2.current_volume, 1100.0);

        // 세 번째 데이터 (첫 번째 데이터가 제거됨)
        let vol3 = builder.next(&candles[2]);
        assert_eq!(vol3.average_volume, 1150.0); // (1100 + 1200) / 2
        assert_eq!(vol3.current_volume, 1200.0);
    }

    #[test]
    fn test_volume_period() {
        let volume = Volume {
            period: 20,
            average_volume: 1000.0,
            current_volume: 1200.0,
            volume_ratio: 1.2,
        };
        assert_eq!(volume.period(), 20);
    }

    #[test]
    fn test_volume_is_high_volume() {
        let volume = Volume {
            period: 20,
            average_volume: 1000.0,
            current_volume: 1600.0,
            volume_ratio: 1.6,
        };
        assert!(volume.is_high_volume(None));
        assert!(volume.is_high_volume(Some(1.5)));
        assert!(!volume.is_high_volume(Some(2.0)));
    }

    #[test]
    fn test_volume_is_low_volume() {
        let volume = Volume {
            period: 20,
            average_volume: 1000.0,
            current_volume: 400.0,
            volume_ratio: 0.4,
        };
        assert!(volume.is_low_volume(None));
        assert!(volume.is_low_volume(Some(0.5)));
        assert!(!volume.is_low_volume(Some(0.3)));
    }

    #[test]
    fn test_volume_is_above_average() {
        let volume = Volume {
            period: 20,
            average_volume: 1000.0,
            current_volume: 1200.0,
            volume_ratio: 1.2,
        };
        assert!(volume.is_above_average());
        assert!(!volume.is_below_average());
    }

    #[test]
    fn test_volume_is_below_average() {
        let volume = Volume {
            period: 20,
            average_volume: 1000.0,
            current_volume: 800.0,
            volume_ratio: 0.8,
        };
        assert!(!volume.is_above_average());
        assert!(volume.is_below_average());
    }

    #[test]
    fn test_volume_known_values_accuracy() {
        // 알려진 Volume 계산 결과와 비교
        // period=2인 경우 간단한 계산으로 검증
        // 평균 거래량 = (v1 + v2) / 2
        // 거래량 비율 = 현재 거래량 / 평균 거래량
        let candles = vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 110.0,
                low: 90.0,
                close: 105.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 1,
                open: 105.0,
                high: 115.0,
                low: 95.0,
                close: 110.0,
                volume: 1500.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 2,
                open: 110.0,
                high: 120.0,
                low: 100.0,
                close: 115.0,
                volume: 2000.0,
            },
        ];

        let mut builder = VolumeBuilder::<TestCandle>::new(2);
        let volume = builder.build(&candles);

        // 평균 거래량 = (1500 + 2000) / 2 = 1750
        let expected_avg = (1500.0 + 2000.0) / 2.0;
        assert!(
            (volume.average_volume - expected_avg).abs() < 0.01,
            "Average volume calculation mismatch. Expected: {}, Got: {}",
            expected_avg,
            volume.average_volume
        );

        // 현재 거래량 = 2000
        assert_eq!(
            volume.current_volume, 2000.0,
            "Current volume should be 2000. Got: {}",
            volume.current_volume
        );

        // 거래량 비율 = 2000 / 1750 = 1.143
        let expected_ratio = 2000.0 / expected_avg;
        assert!(
            (volume.volume_ratio - expected_ratio).abs() < 0.01,
            "Volume ratio calculation mismatch. Expected: {}, Got: {}",
            expected_ratio,
            volume.volume_ratio
        );
    }

    #[test]
    fn test_volume_known_values_period_2() {
        // period=2인 경우 정확한 계산 검증
        let candles = vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 110.0,
                low: 90.0,
                close: 105.0,
                volume: 500.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 1,
                open: 105.0,
                high: 115.0,
                low: 95.0,
                close: 110.0,
                volume: 1000.0,
            },
        ];

        let mut builder = VolumeBuilder::<TestCandle>::new(2);
        let volume = builder.build(&candles);

        // 평균 거래량 = (500 + 1000) / 2 = 750
        let expected_avg = (500.0 + 1000.0) / 2.0;
        assert!(
            (volume.average_volume - expected_avg).abs() < 0.01,
            "Average volume calculation mismatch. Expected: {}, Got: {}",
            expected_avg,
            volume.average_volume
        );

        // 현재 거래량 = 1000
        assert_eq!(
            volume.current_volume, 1000.0,
            "Current volume should be 1000. Got: {}",
            volume.current_volume
        );

        // 거래량 비율 = 1000 / 750 = 1.333
        let expected_ratio = 1000.0 / expected_avg;
        assert!(
            (volume.volume_ratio - expected_ratio).abs() < 0.01,
            "Volume ratio calculation mismatch. Expected: {}, Got: {}",
            expected_ratio,
            volume.volume_ratio
        );

        // 거래량이 평균보다 높으므로 is_above_average는 true
        assert!(
            volume.is_above_average(),
            "Volume should be above average. Ratio: {}",
            volume.volume_ratio
        );
    }
}
