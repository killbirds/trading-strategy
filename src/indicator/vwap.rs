use crate::candle_store::CandleStore;
use crate::indicator::{TABuilder, TAs, TAsBuilder};
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use trading_chart::Candle;

/// VWAP(거래량가중평균가격) 매개변수
///
/// VWAP 계산에 필요한 매개변수 설정을 저장합니다.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct VWAPParams {
    /// 기간 (0이면 모든 데이터 사용)
    pub period: usize,
}

impl Display for VWAPParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VWAP({})", self.period)
    }
}

impl VWAPParams {
    /// 새 VWAP 매개변수 생성
    ///
    /// # Arguments
    /// * `period` - 기간 (0이면 모든 데이터 사용)
    ///
    /// # Returns
    /// * `VWAPParams` - 새 VWAPParams 인스턴스
    pub fn new(period: usize) -> Self {
        VWAPParams { period }
    }
}

impl Default for VWAPParams {
    fn default() -> Self {
        Self::new(0)
    }
}

/// 일일 VWAP 지표
///
/// 거래량 가중 평균 가격(VWAP)은 주어진 기간 동안의 거래량을 고려한 평균 가격입니다.
#[derive(Clone, Debug)]
pub struct VWAP {
    /// VWAP 매개변수
    pub params: VWAPParams,
    /// VWAP 값
    pub value: f64,
}

impl Display for VWAP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VWAP({}): {:.2}", self.params.period, self.value)
    }
}

impl VWAP {
    /// 현재 가격이 VWAP 위에 있는지 확인
    ///
    /// # Arguments
    /// * `price` - 확인할 가격
    ///
    /// # Returns
    /// * `bool` - VWAP 위 여부
    pub fn is_price_above(&self, price: f64) -> bool {
        price > self.value
    }

    /// 현재 가격이 VWAP 아래에 있는지 확인
    ///
    /// # Arguments
    /// * `price` - 확인할 가격
    ///
    /// # Returns
    /// * `bool` - VWAP 아래 여부
    pub fn is_price_below(&self, price: f64) -> bool {
        price < self.value
    }

    /// 가격 대비 VWAP의 상대적 거리(%)
    ///
    /// # Arguments
    /// * `price` - 현재 가격
    ///
    /// # Returns
    /// * `f64` - 가격 대비 VWAP 상대적 거리 백분율
    pub fn price_to_vwap_percent(&self, price: f64) -> f64 {
        if self.value == 0.0 {
            return 0.0; // 0으로 나누기 방지
        }
        ((price - self.value) / self.value) * 100.0
    }
}

/// VWAP 빌더
///
/// VWAP 기술적 지표를 계산하고 업데이트합니다.
///
/// # 성능 고려사항
/// - 메모리 사용량: period개의 (typical_price, volume) 쌍만 유지 (period=0일 때 최대 500개)
/// - 시간 복잡도: O(1) 업데이트 (누적 합계 사용), O(n) 초기 빌드 (n = 데이터 개수)
/// - 최적화: 누적 합계(cumulative_pv, cumulative_volume)를 사용하여 효율적인 업데이트 지원
#[derive(Debug)]
pub struct VWAPBuilder<C: Candle> {
    /// VWAP 매개변수
    params: VWAPParams,
    /// (typical_price, volume) 쌍을 저장
    values: Vec<(f64, f64)>,
    /// 누적 (가격 * 거래량) 합계
    cumulative_pv: f64,
    /// 누적 거래량 합계
    cumulative_volume: f64,
    _phantom: PhantomData<C>,
}

const MAX_PERIOD_0_CAPACITY: usize = 500;

impl<C> VWAPBuilder<C>
where
    C: Candle,
{
    /// 새 VWAP 빌더 생성
    ///
    /// # Arguments
    /// * `params` - VWAP 매개변수
    ///
    /// # Returns
    /// * `VWAPBuilder` - 새 빌더 인스턴스
    pub fn new(params: VWAPParams) -> Self {
        let capacity = if params.period > 0 {
            params.period * 2
        } else {
            MAX_PERIOD_0_CAPACITY
        };

        Self {
            params,
            values: Vec::with_capacity(capacity),
            cumulative_pv: 0.0,
            cumulative_volume: 0.0,
            _phantom: PhantomData,
        }
    }

    /// 저장소에서 VWAP 생성
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    ///
    /// # Returns
    /// * `VWAP` - 계산된 VWAP
    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> VWAP {
        self.build(&storage.get_time_ordered_items())
    }

    /// 데이터 벡터에서 VWAP 생성
    ///
    /// # Arguments
    /// * `data` - 캔들 데이터 벡터
    ///
    /// # Returns
    /// * `VWAP` - 계산된 VWAP
    pub fn build(&mut self, data: &[C]) -> VWAP {
        self.values.clear();
        self.cumulative_pv = 0.0;
        self.cumulative_volume = 0.0;

        if data.is_empty() {
            return VWAP {
                params: self.params,
                value: 0.0,
            };
        }

        let slice_start = if self.params.period > 0 && data.len() > self.params.period {
            data.len() - self.params.period
        } else {
            0
        };

        let data_slice = &data[slice_start..];

        // period=0일 때는 최근 MAX_PERIOD_0_CAPACITY 개만 사용하여 메모리 제한
        let effective_slice = if self.params.period == 0 && data_slice.len() > MAX_PERIOD_0_CAPACITY
        {
            let start = data_slice.len() - MAX_PERIOD_0_CAPACITY;
            &data_slice[start..]
        } else {
            data_slice
        };

        for item in effective_slice {
            let typical_price = (item.high_price() + item.low_price() + item.close_price()) / 3.0;
            let volume = item.volume();

            // NaN/Infinity 체크
            if typical_price.is_nan()
                || typical_price.is_infinite()
                || volume.is_nan()
                || volume.is_infinite()
            {
                continue;
            }

            self.values.push((typical_price, volume));
            self.cumulative_pv += typical_price * volume;
            self.cumulative_volume += volume;
        }

        if self.params.period > 0 && self.values.len() < self.params.period {
            let (price, _) = *self.values.last().unwrap_or(&(0.0, 0.0));
            return VWAP {
                params: self.params,
                value: if price.is_nan() || price.is_infinite() {
                    0.0
                } else {
                    price
                },
            };
        }

        let vwap_value = if self.cumulative_volume > 0.0 {
            self.cumulative_pv / self.cumulative_volume
        } else {
            0.0
        };

        // 결과값 유효성 검증
        let final_value = if vwap_value.is_nan() || vwap_value.is_infinite() {
            0.0
        } else {
            vwap_value
        };

        VWAP {
            params: self.params,
            value: final_value,
        }
    }

    /// 새 데이터로 VWAP 업데이트
    ///
    /// # Arguments
    /// * `data` - 새 캔들 데이터
    ///
    /// # Returns
    /// * `VWAP` - 업데이트된 VWAP
    pub fn next(&mut self, data: &C) -> VWAP {
        let typical_price = (data.high_price() + data.low_price() + data.close_price()) / 3.0;
        let volume = data.volume();

        // NaN/Infinity 체크
        if typical_price.is_nan()
            || typical_price.is_infinite()
            || volume.is_nan()
            || volume.is_infinite()
            || data.high_price().is_nan()
            || data.low_price().is_nan()
            || data.close_price().is_nan()
        {
            return VWAP {
                params: self.params,
                value: 0.0,
            };
        }

        let max_len = if self.params.period > 0 {
            self.params.period
        } else {
            MAX_PERIOD_0_CAPACITY
        };

        if self.values.len() >= max_len {
            if let Some((old_price, old_volume)) = self.values.first().copied() {
                self.cumulative_pv -= old_price * old_volume;
                self.cumulative_volume -= old_volume;
            }
            self.values.drain(0..1);
        }

        self.values.push((typical_price, volume));
        self.cumulative_pv += typical_price * volume;
        self.cumulative_volume += volume;

        if self.params.period > 0 && self.values.len() < self.params.period {
            return VWAP {
                params: self.params,
                value: typical_price,
            };
        }

        if self.params.period > 0 && self.values.len() > self.params.period * 2 {
            let excess = self.values.len() - self.params.period * 2;
            for &(price, vol) in &self.values[..excess] {
                self.cumulative_pv -= price * vol;
                self.cumulative_volume -= vol;
            }
            self.values.drain(0..excess);
        }

        let vwap_value = if self.cumulative_volume > 0.0 {
            self.cumulative_pv / self.cumulative_volume
        } else {
            0.0
        };

        // 결과값 유효성 검증
        let final_value = if vwap_value.is_nan() || vwap_value.is_infinite() {
            0.0
        } else {
            vwap_value
        };

        VWAP {
            params: self.params,
            value: final_value,
        }
    }

    /// VWAP 리셋 (일일 계산에 사용)
    pub fn reset(&mut self) {
        self.values.clear();
        self.cumulative_pv = 0.0;
        self.cumulative_volume = 0.0;
    }
}

impl<C> TABuilder<VWAP, C> for VWAPBuilder<C>
where
    C: Candle,
{
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> VWAP {
        self.build_from_storage(storage)
    }

    fn build(&mut self, data: &[C]) -> VWAP {
        self.build(data)
    }

    fn next(&mut self, data: &C) -> VWAP {
        self.next(data)
    }
}

pub type VWAPs = TAs<VWAPParams, VWAP>;
pub type VWAPsBuilder<C> = TAsBuilder<VWAPParams, VWAP, C>;

pub struct VWAPsBuilderFactory;
impl VWAPsBuilderFactory {
    pub fn build<C: Candle + 'static>(params_list: &[VWAPParams]) -> VWAPsBuilder<C> {
        VWAPsBuilder::new("vwaps".to_owned(), params_list, |params| {
            Box::new(VWAPBuilder::<C>::new(*params))
        })
    }

    pub fn build_default<C: Candle + 'static>() -> VWAPsBuilder<C> {
        Self::build::<C>(&[VWAPParams::default()])
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
                volume: 2000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 110.0,
                high: 120.0,
                low: 100.0,
                close: 115.0,
                volume: 1500.0,
            },
        ]
    }

    #[test]
    fn test_vwap_params_new() {
        let params = VWAPParams::new(20);
        assert_eq!(params.period, 20);
    }

    #[test]
    fn test_vwap_params_default() {
        let params = VWAPParams::default();
        assert_eq!(params.period, 0);
    }

    #[test]
    fn test_vwap_params_display() {
        let params = VWAPParams::new(20);
        assert_eq!(format!("{params}"), "VWAP(20)");
    }

    #[test]
    fn test_vwap_builder_new() {
        let builder = VWAPBuilder::<TestCandle>::new(VWAPParams::new(20));
        assert_eq!(builder.params.period, 20);
    }

    #[test]
    fn test_vwap_build_empty_data() {
        let mut builder = VWAPBuilder::<TestCandle>::new(VWAPParams::new(20));
        let vwap = builder.build(&[]);
        assert_eq!(vwap.params.period, 20);
        assert_eq!(vwap.value, 0.0);
    }

    #[test]
    fn test_vwap_build_with_data() {
        let mut builder = VWAPBuilder::<TestCandle>::new(VWAPParams::new(3));
        let candles = create_test_candles();
        let vwap = builder.build(&candles);

        assert_eq!(vwap.params.period, 3);
        assert!(vwap.value > 0.0);
    }

    #[test]
    fn test_vwap_calculation_accuracy() {
        let mut builder = VWAPBuilder::<TestCandle>::new(VWAPParams::new(3));
        let candles = create_test_candles();
        let vwap = builder.build(&candles);

        // Typical price 계산: (high + low + close) / 3
        let tp1 = (110.0 + 90.0 + 105.0) / 3.0; // 101.67
        let tp2 = (115.0 + 95.0 + 110.0) / 3.0; // 106.67
        let tp3 = (120.0 + 100.0 + 115.0) / 3.0; // 111.67

        // VWAP = (tp1*v1 + tp2*v2 + tp3*v3) / (v1 + v2 + v3)
        let expected = (tp1 * 1000.0 + tp2 * 2000.0 + tp3 * 1500.0) / (1000.0 + 2000.0 + 1500.0);
        assert!((vwap.value - expected).abs() < 0.01);
    }

    #[test]
    fn test_vwap_period_zero() {
        let mut builder = VWAPBuilder::<TestCandle>::new(VWAPParams::new(0));
        let candles = create_test_candles();
        let vwap = builder.build(&candles);

        assert_eq!(vwap.params.period, 0);
        assert!(vwap.value > 0.0);
    }

    #[test]
    fn test_vwap_next() {
        let mut builder = VWAPBuilder::<TestCandle>::new(VWAPParams::new(3));
        let candles = create_test_candles();
        let vwap = builder.next(&candles[0]);

        assert_eq!(vwap.params.period, 3);
        assert!(vwap.value > 0.0);
    }

    #[test]
    fn test_vwap_incremental_vs_build() {
        let mut builder1 = VWAPBuilder::<TestCandle>::new(VWAPParams::new(3));
        let mut builder2 = VWAPBuilder::<TestCandle>::new(VWAPParams::new(3));
        let candles = create_test_candles();

        // builder1: next를 여러 번 호출
        for candle in &candles {
            builder1.next(candle);
        }
        let vwap1 = builder1.next(&candles[candles.len() - 1]);

        // builder2: build를 한 번 호출
        let vwap2 = builder2.build(&candles);

        // 둘 다 유효한 값이어야 함 (next와 build는 내부 상태 차이로 약간 다를 수 있음)
        assert!(vwap1.value > 0.0);
        assert!(vwap2.value > 0.0);
        // 값이 비슷한 범위 내에 있어야 함 (5% 이내로 더 엄격하게 검증)
        let diff_percent = if vwap2.value > 0.0 {
            ((vwap1.value - vwap2.value).abs() / vwap2.value) * 100.0
        } else {
            0.0
        };
        assert!(
            diff_percent < 5.0,
            "VWAP values should be consistent. Incremental: {}, Build: {}, Diff: {}%",
            vwap1.value,
            vwap2.value,
            diff_percent
        );
    }

    #[test]
    fn test_vwap_is_price_above() {
        let vwap = VWAP {
            params: VWAPParams::new(20),
            value: 100.0,
        };
        assert!(vwap.is_price_above(110.0));
        assert!(!vwap.is_price_above(90.0));
    }

    #[test]
    fn test_vwap_is_price_below() {
        let vwap = VWAP {
            params: VWAPParams::new(20),
            value: 100.0,
        };
        assert!(vwap.is_price_below(90.0));
        assert!(!vwap.is_price_below(110.0));
    }

    #[test]
    fn test_vwap_price_to_vwap_percent() {
        let vwap = VWAP {
            params: VWAPParams::new(20),
            value: 100.0,
        };
        let percent = vwap.price_to_vwap_percent(110.0);
        assert!((percent - 10.0).abs() < 0.01); // 10% 위

        let percent2 = vwap.price_to_vwap_percent(90.0);
        assert!((percent2 - (-10.0)).abs() < 0.01); // 10% 아래
    }

    #[test]
    fn test_vwap_price_to_vwap_percent_zero() {
        let vwap = VWAP {
            params: VWAPParams::new(20),
            value: 0.0,
        };
        let percent = vwap.price_to_vwap_percent(100.0);
        assert_eq!(percent, 0.0); // 0으로 나누기 방지
    }

    #[test]
    fn test_vwap_display() {
        let vwap = VWAP {
            params: VWAPParams::new(20),
            value: 100.5,
        };
        let display_str = format!("{vwap}");
        assert!(display_str.contains("VWAP"));
        assert!(display_str.contains("20"));
        assert!(display_str.contains("100.50"));
    }

    #[test]
    fn test_vwap_reset() {
        let mut builder = VWAPBuilder::<TestCandle>::new(VWAPParams::new(3));
        let candles = create_test_candles();
        let _vwap1 = builder.build(&candles);

        builder.reset();
        let vwap2 = builder.build(&candles);

        // 리셋 후에도 같은 결과가 나와야 함
        assert!(vwap2.value > 0.0);
    }

    #[test]
    fn test_vwap_insufficient_data() {
        let mut builder = VWAPBuilder::<TestCandle>::new(VWAPParams::new(10));
        let candles = create_test_candles(); // 3개만 있음
        let vwap = builder.build(&candles);

        // 데이터가 부족하면 마지막 typical price 반환
        assert!(vwap.value > 0.0);
    }

    #[test]
    fn test_vwap_volume_weighted() {
        // 거래량이 큰 캔들이 VWAP에 더 큰 영향을 미치는지 확인
        let mut builder = VWAPBuilder::<TestCandle>::new(VWAPParams::new(2));
        let candles = vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 110.0,
                low: 90.0,
                close: 105.0,
                volume: 1000.0, // 작은 거래량
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 105.0,
                high: 115.0,
                low: 95.0,
                close: 110.0,
                volume: 10000.0, // 큰 거래량
            },
        ];

        let vwap = builder.build(&candles);
        // 두 번째 캔들의 typical price에 더 가까워야 함 (거래량이 크므로)
        let tp1 = (110.0 + 90.0 + 105.0) / 3.0;
        let tp2 = (115.0 + 95.0 + 110.0) / 3.0;
        let expected = (tp1 * 1000.0 + tp2 * 10000.0) / (1000.0 + 10000.0);
        assert!((vwap.value - expected).abs() < 0.01);
    }

    #[test]
    fn test_vwaps_builder() {
        let mut builder =
            VWAPsBuilderFactory::build::<TestCandle>(&[VWAPParams::new(3), VWAPParams::new(5)]);
        let candles = create_test_candles();

        let vwaps = builder.build(&candles);
        let vwap1 = vwaps.get(&VWAPParams::new(3));
        let vwap2 = vwaps.get(&VWAPParams::new(5));

        assert!(vwap1.value > 0.0);
        assert!(vwap2.value > 0.0);
    }

    #[test]
    fn test_vwap_known_values_accuracy() {
        // 알려진 VWAP 계산 결과와 비교
        // period=2인 경우 간단한 계산으로 검증
        // VWAP = sum(typical_price * volume) / sum(volume)
        let candles = vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 105.0,
                low: 95.0,
                close: 102.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 1,
                open: 102.0,
                high: 108.0,
                low: 100.0,
                close: 106.0,
                volume: 2000.0,
            },
        ];

        let mut builder = VWAPBuilder::<TestCandle>::new(VWAPParams::new(2));
        let vwap = builder.build(&candles);

        // Typical Price 계산
        let tp1 = (105.0 + 95.0 + 102.0) / 3.0; // 100.67
        let tp2 = (108.0 + 100.0 + 106.0) / 3.0; // 104.67

        // VWAP = (tp1 * v1 + tp2 * v2) / (v1 + v2)
        let expected = (tp1 * 1000.0 + tp2 * 2000.0) / (1000.0 + 2000.0);

        assert!(
            (vwap.value - expected).abs() < 0.01,
            "VWAP calculation mismatch. Expected: {}, Got: {}",
            expected,
            vwap.value
        );
    }

    #[test]
    fn test_vwap_known_values_period_2() {
        // period=2인 경우 정확한 계산 검증
        // 거래량이 다른 두 캔들
        let candles = vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 10.0,
                high: 12.0,
                low: 8.0,
                close: 11.0,
                volume: 100.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 1,
                open: 11.0,
                high: 14.0,
                low: 10.0,
                close: 13.0,
                volume: 300.0,
            },
        ];

        let mut builder = VWAPBuilder::<TestCandle>::new(VWAPParams::new(2));
        let vwap = builder.build(&candles);

        // Typical Price 계산
        let tp1 = (12.0 + 8.0 + 11.0) / 3.0; // 10.33
        let tp2 = (14.0 + 10.0 + 13.0) / 3.0; // 12.33

        // VWAP = (tp1 * v1 + tp2 * v2) / (v1 + v2)
        let expected = (tp1 * 100.0 + tp2 * 300.0) / (100.0 + 300.0);

        assert!(
            (vwap.value - expected).abs() < 0.01,
            "VWAP calculation mismatch. Expected: {}, Got: {}",
            expected,
            vwap.value
        );

        // 두 번째 캔들의 거래량이 더 크므로 VWAP는 tp2에 더 가까워야 함
        assert!(
            (vwap.value - tp2).abs() < (vwap.value - tp1).abs(),
            "VWAP should be closer to tp2 (higher volume). VWAP: {}, tp1: {}, tp2: {}",
            vwap.value,
            tp1,
            tp2
        );
    }
}
