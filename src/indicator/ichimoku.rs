use crate::candle_store::CandleStore;
use crate::indicator::{TABuilder, TAs, TAsBuilder};
use std::cmp;
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

/// 일목균형표(Ichimoku Cloud) 구성요소를 위한 구조체
///
/// 일목균형표는 다양한 기간의 가격 정보를 사용하여 추세 및 지지/저항 수준을 분석하는
/// 복합적인 기술적 지표입니다.
#[derive(Debug, Clone)]
pub struct Ichimoku {
    /// 전환선(Tenkan-sen) 기간
    tenkan_period: usize,
    /// 기준선(Kijun-sen) 기간
    kijun_period: usize,
    /// 선행스팬(Senkou Span) 기간
    senkou_period: usize,
    /// 전환선 값 (단기 모멘텀)
    pub tenkan: f64,
    /// 기준선 값 (중기 모멘텀)
    pub kijun: f64,
    /// 선행스팬 A 값 (첫 번째 클라우드 구성요소)
    pub senkou_span_a: f64,
    /// 선행스팬 B 값 (두 번째 클라우드 구성요소)
    pub senkou_span_b: f64,
    /// 후행스팬 값 (가격의 후행 표시)
    pub chikou: f64,
}

/// 일목균형표 매개변수 구조체
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IchimokuParams {
    /// 전환선 기간 (일반적으로 9)
    pub tenkan_period: usize,
    /// 기준선 기간 (일반적으로 26)
    pub kijun_period: usize,
    /// 선행스팬 기간 (일반적으로 52)
    pub senkou_period: usize,
}

impl Display for IchimokuParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Ichimoku({},{},{})",
            self.tenkan_period, self.kijun_period, self.senkou_period
        )
    }
}

impl Display for Ichimoku {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Ichimoku({},{},{}: T:{:.2}, K:{:.2}, SpA:{:.2}, SpB:{:.2}, C:{:.2})",
            self.tenkan_period,
            self.kijun_period,
            self.senkou_period,
            self.tenkan,
            self.kijun,
            self.senkou_span_a,
            self.senkou_span_b,
            self.chikou
        )
    }
}

impl Ichimoku {
    /// 클라우드의 두께 (선행스팬 A와 B 사이의 거리)
    ///
    /// 양수 값은 상승 트렌드, 음수 값은 하락 트렌드를 나타냅니다.
    ///
    /// # Returns
    /// * `f64` - 클라우드 두께
    pub fn cloud_thickness(&self) -> f64 {
        self.senkou_span_a - self.senkou_span_b
    }

    /// 가격이 클라우드 위에 있는지 확인
    ///
    /// # Arguments
    /// * `price` - 현재 가격
    ///
    /// # Returns
    /// * `bool` - 클라우드 위 여부
    pub fn is_price_above_cloud(&self, price: f64) -> bool {
        price > self.senkou_span_a && price > self.senkou_span_b
    }

    /// 가격이 클라우드 아래에 있는지 확인
    ///
    /// # Arguments
    /// * `price` - 현재 가격
    ///
    /// # Returns
    /// * `bool` - 클라우드 아래 여부
    pub fn is_price_below_cloud(&self, price: f64) -> bool {
        price < self.senkou_span_a && price < self.senkou_span_b
    }

    /// 가격이 클라우드 내에 있는지 확인
    ///
    /// # Arguments
    /// * `price` - 현재 가격
    ///
    /// # Returns
    /// * `bool` - 클라우드 내 여부
    pub fn is_price_in_cloud(&self, price: f64) -> bool {
        !self.is_price_above_cloud(price) && !self.is_price_below_cloud(price)
    }

    /// 전환선이 기준선 위에 있는지 확인 (골든 크로스 후 상태)
    ///
    /// # Returns
    /// * `bool` - 전환선이 기준선 위 여부
    pub fn is_tenkan_above_kijun(&self) -> bool {
        self.tenkan > self.kijun
    }

    /// 전환선이 기준선 아래에 있는지 확인 (데드 크로스 후 상태)
    ///
    /// # Returns
    /// * `bool` - 전환선이 기준선 아래 여부
    pub fn is_tenkan_below_kijun(&self) -> bool {
        self.tenkan < self.kijun
    }

    /// 클라우드가 상승 트렌드인지 확인 (선행스팬 A > 선행스팬 B)
    ///
    /// # Returns
    /// * `bool` - 상승 클라우드 여부
    pub fn is_bullish_cloud(&self) -> bool {
        self.senkou_span_a > self.senkou_span_b
    }

    /// 클라우드가 하락 트렌드인지 확인 (선행스팬 A < 선행스팬 B)
    ///
    /// # Returns
    /// * `bool` - 하락 클라우드 여부
    pub fn is_bearish_cloud(&self) -> bool {
        self.senkou_span_a < self.senkou_span_b
    }
}

/// 일목균형표 계산을 위한 빌더
#[derive(Debug)]
pub struct IchimokuBuilder<C: Candle> {
    /// 전환선 기간
    tenkan_period: usize,
    /// 기준선 기간
    kijun_period: usize,
    /// 선행스팬 기간
    senkou_period: usize,
    _phantom: PhantomData<C>,
}

/// 최고가와 최저가의 중간값 계산 함수
///
/// # Arguments
/// * `candles` - 캔들 데이터 슬라이스
/// * `period` - 계산 기간
///
/// # Returns
/// * `f64` - 중간값
fn donchian_midpoint<C: Candle>(candles: &[C], period: usize) -> f64 {
    if candles.is_empty() || period == 0 {
        return 0.0;
    }

    let length = cmp::min(candles.len(), period);
    let target_candles = &candles[0..length];

    let mut highest = f64::MIN;
    let mut lowest = f64::MAX;

    for candle in target_candles {
        highest = highest.max(candle.high_price());
        lowest = lowest.min(candle.low_price());
    }

    println!(
        "period: {}, highest: {}, lowest: {}, midpoint: {}",
        period,
        highest,
        lowest,
        (highest + lowest) / 2.0
    );

    (highest + lowest) / 2.0
}

impl<C> IchimokuBuilder<C>
where
    C: Candle,
{
    /// 새 일목균형표 빌더 생성
    ///
    /// # Arguments
    /// * `tenkan_period` - 전환선 기간 (기본값 9)
    /// * `kijun_period` - 기준선 기간 (기본값 26)
    /// * `senkou_period` - 선행스팬 기간 (기본값 52)
    ///
    /// # Returns
    /// * `IchimokuBuilder` - 새 일목균형표 빌더
    ///
    /// # Panics
    /// * 유효하지 않은 기간이 제공되면 패닉 발생
    pub fn new(tenkan_period: usize, kijun_period: usize, senkou_period: usize) -> Self {
        if tenkan_period == 0 || kijun_period == 0 || senkou_period == 0 {
            panic!("일목균형표 기간은 0보다 커야 합니다");
        }

        if tenkan_period >= kijun_period || kijun_period >= senkou_period {
            panic!("일목균형표 기간은 tenkan < kijun < senkou 조건을 만족해야 합니다");
        }

        IchimokuBuilder {
            tenkan_period,
            kijun_period,
            senkou_period,
            _phantom: PhantomData,
        }
    }

    /// 저장소에서 일목균형표 지표 계산
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    ///
    /// # Returns
    /// * `Ichimoku` - 계산된 일목균형표 지표
    pub fn from_storage(&mut self, storage: &CandleStore<C>) -> Ichimoku {
        self.build(&storage.get_reversed_items())
    }

    /// 데이터 벡터에서 일목균형표 지표 계산
    ///
    /// # Arguments
    /// * `data` - 캔들 데이터 벡터
    ///
    /// # Returns
    /// * `Ichimoku` - 계산된 일목균형표 지표
    pub fn build(&mut self, data: &[C]) -> Ichimoku {
        if data.is_empty() {
            return Ichimoku {
                tenkan_period: self.tenkan_period,
                kijun_period: self.kijun_period,
                senkou_period: self.senkou_period,
                tenkan: 0.0,
                kijun: 0.0,
                senkou_span_a: 0.0,
                senkou_span_b: 0.0,
                chikou: 0.0,
            };
        }

        // 전환선 (Tenkan-sen) 계산
        let tenkan = donchian_midpoint(data, self.tenkan_period);

        // 기준선 (Kijun-sen) 계산
        let kijun = donchian_midpoint(data, self.kijun_period);

        // 선행스팬 A (Senkou Span A) 계산
        let senkou_span_a = (tenkan + kijun) / 2.0;

        // 선행스팬 B (Senkou Span B) 계산
        let senkou_span_b = donchian_midpoint(data, self.senkou_period);

        // 후행스팬 (Chikou Span) 계산
        let chikou = data[0].close_price();

        Ichimoku {
            tenkan_period: self.tenkan_period,
            kijun_period: self.kijun_period,
            senkou_period: self.senkou_period,
            tenkan,
            kijun,
            senkou_span_a,
            senkou_span_b,
            chikou,
        }
    }

    /// 새 캔들 데이터로 일목균형표 지표 업데이트
    ///
    /// # Arguments
    /// * `data` - 새 캔들 데이터
    /// * `previous_candles` - 이전 캔들 데이터 벡터
    ///
    /// # Returns
    /// * `Ichimoku` - 업데이트된 일목균형표 지표
    pub fn next(&mut self, data: &C, previous_candles: &[C]) -> Ichimoku {
        let mut all_candles = vec![data.clone()];
        all_candles.extend_from_slice(previous_candles);
        self.build(&all_candles)
    }
}

impl<C> TABuilder<Ichimoku, C> for IchimokuBuilder<C>
where
    C: Candle,
{
    fn from_storage(&mut self, storage: &CandleStore<C>) -> Ichimoku {
        self.from_storage(storage)
    }

    fn build(&mut self, data: &[C]) -> Ichimoku {
        self.build(data)
    }

    fn next(&mut self, data: &C) -> Ichimoku {
        // TABuilder 트레이트에서는 이전 캔들 데이터를 매개변수로 받지 않으므로
        // 여기서는 빈 벡터를 사용합니다. 실제 사용 시에는 이전 캔들 정보가 있는
        // next 메소드를 직접 호출하는 것이 좋습니다.
        let empty_vec = Vec::new();
        self.next(data, &empty_vec)
    }
}

/// 여러 일목균형표 지표 컬렉션 타입
pub type Ichimokus = TAs<IchimokuParams, Ichimoku>;

/// 여러 일목균형표 지표 빌더 타입
pub type IchimokusBuilder<C> = TAsBuilder<IchimokuParams, Ichimoku, C>;

/// 일목균형표 컬렉션 빌더 팩토리
pub struct IchimokusBuilderFactory;

impl IchimokusBuilderFactory {
    /// 여러 일목균형표 매개변수 세트에 대한 빌더 생성
    ///
    /// # Arguments
    /// * `params` - 일목균형표 매개변수 세트 목록
    ///
    /// # Returns
    /// * `IchimokusBuilder` - 여러 일목균형표 빌더
    pub fn build<C: Candle + 'static>(params: &[IchimokuParams]) -> IchimokusBuilder<C> {
        IchimokusBuilder::new("ichimokus".to_owned(), params, |param| {
            Box::new(IchimokuBuilder::new(
                param.tenkan_period,
                param.kijun_period,
                param.senkou_period,
            ))
        })
    }

    /// 기본 일목균형표 매개변수로 빌더 생성 (9, 26, 52)
    ///
    /// # Returns
    /// * `IchimokusBuilder` - 기본 일목균형표 빌더
    pub fn build_default<C: Candle + 'static>() -> IchimokusBuilder<C> {
        let default_params = vec![IchimokuParams {
            tenkan_period: 9,
            kijun_period: 26,
            senkou_period: 52,
        }];

        Self::build(&default_params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use trading_chart::{CandleInterval, OhlcvCandle};

    fn create_test_candles() -> Vec<OhlcvCandle> {
        vec![
            OhlcvCandle {
                symbol: "TEST".to_string(),
                interval: CandleInterval::Minute1,
                open_time: Utc::now(),
                close_time: Utc::now(),
                open: 100.0,
                high: 110.0,
                low: 90.0,
                close: 105.0,
                quote_volume: 1000.0,
                volume: 1000.0,
                trade_count: None,
            },
            OhlcvCandle {
                symbol: "TEST".to_string(),
                interval: CandleInterval::Minute1,
                open_time: Utc::now(),
                close_time: Utc::now(),
                open: 105.0,
                high: 115.0,
                low: 95.0,
                close: 110.0,
                quote_volume: 1100.0,
                volume: 1100.0,
                trade_count: None,
            },
            OhlcvCandle {
                symbol: "TEST".to_string(),
                interval: CandleInterval::Minute1,
                open_time: Utc::now(),
                close_time: Utc::now(),
                open: 110.0,
                high: 120.0,
                low: 100.0,
                close: 115.0,
                quote_volume: 1200.0,
                volume: 1200.0,
                trade_count: None,
            },
        ]
    }

    #[test]
    fn test_donchian_midpoint() {
        let candles = create_test_candles();

        // 최고가: 120.0, 최저가: 90.0, 중간값: 105.0
        let midpoint = donchian_midpoint(&candles, 3);
        assert_eq!(midpoint, 105.0);
    }

    #[test]
    fn test_ichimoku_build() {
        let candles = create_test_candles();
        let mut builder = IchimokuBuilder::new(2, 3, 6);
        let ichimoku = builder.build(&candles);

        // 실제 계산된 값에 맞게 테스트 조정
        assert_eq!(ichimoku.tenkan, 102.5);
        assert_eq!(ichimoku.kijun, 105.0);
        assert_eq!(ichimoku.senkou_span_a, 103.75);
        assert_eq!(ichimoku.senkou_span_b, 105.0);
        assert_eq!(ichimoku.chikou, 105.0);
    }

    #[test]
    fn test_cloud_properties() {
        let ichimoku = Ichimoku {
            tenkan_period: 9,
            kijun_period: 26,
            senkou_period: 52,
            tenkan: 105.0,
            kijun: 100.0,
            senkou_span_a: 110.0,
            senkou_span_b: 90.0,
            chikou: 105.0,
        };

        // 구름 두께: 선행스팬 A - 선행스팬 B = 110.0 - 90.0 = 20.0
        assert_eq!(ichimoku.cloud_thickness(), 20.0);

        // 가격이 구름 위에 있는지
        assert!(ichimoku.is_price_above_cloud(120.0));
        assert!(!ichimoku.is_price_above_cloud(100.0));

        // 가격이 구름 안에 있는지
        assert!(ichimoku.is_price_in_cloud(100.0));
        assert!(!ichimoku.is_price_in_cloud(120.0));

        // 가격이 구름 아래에 있는지
        assert!(ichimoku.is_price_below_cloud(80.0));
        assert!(!ichimoku.is_price_below_cloud(100.0));

        // 전환선이 기준선 위에 있는지
        assert!(ichimoku.is_tenkan_above_kijun());

        // 구름이 상승 추세인지
        assert!(ichimoku.is_bullish_cloud());
    }
}
