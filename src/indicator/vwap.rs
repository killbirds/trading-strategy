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
#[derive(Debug)]
pub struct VWAPBuilder<C: Candle> {
    /// VWAP 매개변수
    params: VWAPParams,
    /// 총 (가격 * 거래량) 값
    values: Vec<(f64, f64)>, // (typical_price, volume) 쌍을 저장
    _phantom: PhantomData<C>,
}

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
            100 // 기본 용량
        };

        Self {
            params,
            values: Vec::with_capacity(capacity),
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
    pub fn from_storage(&mut self, storage: &CandleStore<C>) -> VWAP {
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
        if data.is_empty() {
            return VWAP {
                params: self.params,
                value: 0.0,
            };
        }

        // 데이터를 values 배열에 저장
        self.values.clear();
        for item in data {
            let typical_price = (item.high_price() + item.low_price() + item.close_price()) / 3.0;
            self.values.push((typical_price, item.quote_volume()));
        }

        // 충분한 데이터가 없는 경우
        if self.params.period > 0 && self.values.len() < self.params.period {
            let (price, _) = *self.values.last().unwrap_or(&(0.0, 0.0));
            return VWAP {
                params: self.params,
                value: price,
            };
        }

        // VWAP 계산
        let (cumulative_pv, cumulative_volume) = self
            .values
            .iter()
            .fold((0.0, 0.0), |(pv, vol), (price, volume)| {
                (pv + price * volume, vol + volume)
            });

        VWAP {
            params: self.params,
            value: if cumulative_volume > 0.0 {
                cumulative_pv / cumulative_volume
            } else {
                0.0
            },
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
        // 새 데이터 추가
        let typical_price = (data.high_price() + data.low_price() + data.close_price()) / 3.0;
        self.values.push((typical_price, data.quote_volume()));

        // 필요한 데이터만 유지
        if self.params.period > 0 && self.values.len() > self.params.period * 2 {
            let excess = self.values.len() - self.params.period * 2;
            self.values.drain(0..excess);
        }

        // 충분한 데이터가 없는 경우
        if self.params.period > 0 && self.values.len() < self.params.period {
            return VWAP {
                params: self.params,
                value: typical_price,
            };
        }

        // VWAP 계산
        let (cumulative_pv, cumulative_volume) = self
            .values
            .iter()
            .fold((0.0, 0.0), |(pv, vol), (price, volume)| {
                (pv + price * volume, vol + volume)
            });

        VWAP {
            params: self.params,
            value: if cumulative_volume > 0.0 {
                cumulative_pv / cumulative_volume
            } else {
                0.0
            },
        }
    }

    /// VWAP 리셋 (일일 계산에 사용)
    pub fn reset(&mut self) {
        self.values.clear();
    }
}

impl<C> TABuilder<VWAP, C> for VWAPBuilder<C>
where
    C: Candle,
{
    fn from_storage(&mut self, storage: &CandleStore<C>) -> VWAP {
        self.from_storage(storage)
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
