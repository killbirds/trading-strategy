use crate::candle_store::CandleStore;
use crate::indicator::{TABuilder, TAs, TAsBuilder};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

/// MACD(Moving Average Convergence Divergence) 계산을 위한 빌더
///
/// MACD는 두 개의 이동평균선(빠른 EMA와 느린 EMA)의 차이를 계산하고,
/// 이 값에 대한 시그널 라인(MACD의 EMA)을 제공하는 기술적 지표입니다.
#[derive(Debug)]
pub struct MACDBuilder<C: Candle> {
    /// 빠른 EMA 기간 (일반적으로 12)
    fast_period: usize,
    /// 느린 EMA 기간 (일반적으로 26)
    slow_period: usize,
    /// 시그널 라인 기간 (일반적으로 9)
    signal_period: usize,
    /// MACD 계산을 위한 가격 저장 배열
    values: Vec<f64>,
    _phantom: PhantomData<C>,
}

/// MACD(Moving Average Convergence Divergence) 기술적 지표
///
/// MACD는 추세 추종 모멘텀 지표로, 추세의 방향과 강도를 나타냅니다.
#[derive(Clone, Debug)]
pub struct MACD {
    /// 빠른 EMA 기간
    fast_period: usize,
    /// 느린 EMA 기간
    slow_period: usize,
    /// 시그널 라인 기간
    signal_period: usize,
    /// MACD 라인 (빠른 EMA - 느린 EMA)
    pub macd_line: f64,
    /// 시그널 라인 (MACD의 EMA)
    pub signal_line: f64,
    /// 히스토그램 (MACD - 시그널)
    pub histogram: f64,
}

impl Display for MACD {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MACD({},{},{}: {:.2}, {:.2}, {:.2})",
            self.fast_period,
            self.slow_period,
            self.signal_period,
            self.macd_line,
            self.signal_line,
            self.histogram
        )
    }
}

/// MACD 계산 함수
fn calculate_macd(
    values: &[f64],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> (f64, f64, f64) {
    if values.len() < slow_period {
        return (0.0, 0.0, 0.0);
    }

    // 초기 SMA 계산
    let fast_sma: f64 = values[..fast_period].iter().sum::<f64>() / fast_period as f64;
    let slow_sma: f64 = values[..slow_period].iter().sum::<f64>() / slow_period as f64;

    // EMA 승수 계산
    let fast_multiplier = 2.0 / (fast_period + 1) as f64;
    let slow_multiplier = 2.0 / (slow_period + 1) as f64;
    let signal_multiplier = 2.0 / (signal_period + 1) as f64;

    // EMA 계산을 위한 벡터
    let mut fast_emas = Vec::with_capacity(values.len());
    let mut slow_emas = Vec::with_capacity(values.len());
    let mut macd_lines = Vec::with_capacity(values.len());

    // 초기값 설정
    let mut fast_ema = fast_sma;
    let mut slow_ema = slow_sma;

    // EMA 계산
    for &price in values.iter() {
        // 빠른 EMA 업데이트
        fast_ema = (price - fast_ema) * fast_multiplier + fast_ema;
        fast_emas.push(fast_ema);

        // 느린 EMA 업데이트
        slow_ema = (price - slow_ema) * slow_multiplier + slow_ema;
        slow_emas.push(slow_ema);

        // MACD 라인 계산
        let macd_line = fast_ema - slow_ema;
        macd_lines.push(macd_line);
    }

    // 시그널 라인 계산
    let mut signal_line = 0.0;
    if macd_lines.len() >= signal_period {
        // 초기 시그널 라인 (SMA)
        let signal_sma = macd_lines[macd_lines.len() - signal_period..]
            .iter()
            .sum::<f64>()
            / signal_period as f64;
        signal_line = signal_sma;

        // EMA로 시그널 라인 업데이트
        for &macd in macd_lines[macd_lines.len() - signal_period..].iter() {
            signal_line = (macd - signal_line) * signal_multiplier + signal_line;
        }
    }

    let macd_line = *macd_lines.last().unwrap_or(&0.0);
    let histogram = macd_line - signal_line;

    (macd_line, signal_line, histogram)
}

impl<C> MACDBuilder<C>
where
    C: Candle,
{
    /// 새 MACD 빌더 생성
    ///
    /// # Arguments
    /// * `fast_period` - 빠른 EMA 기간 (기본값 12)
    /// * `slow_period` - 느린 EMA 기간 (기본값 26)
    /// * `signal_period` - 시그널 라인 기간 (기본값 9)
    ///
    /// # Returns
    /// * `MACDBuilder` - 새 MACD 빌더 인스턴스
    ///
    /// # Panics
    /// * 유효하지 않은 기간이 제공되면 패닉 발생
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> Self {
        if fast_period == 0 || slow_period == 0 || signal_period == 0 {
            panic!("MACD 기간은 0보다 커야 합니다");
        }

        if fast_period >= slow_period {
            panic!("빠른 기간은 느린 기간보다 작아야 합니다");
        }

        Self {
            fast_period,
            slow_period,
            signal_period,
            values: Vec::with_capacity(slow_period * 2),
            _phantom: PhantomData,
        }
    }

    /// 저장소에서 MACD 지표 생성
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    ///
    /// # Returns
    /// * `MACD` - 계산된 MACD 지표
    pub fn from_storage(&mut self, storage: &CandleStore<C>) -> MACD {
        self.build(&storage.get_time_ordered_items())
    }

    /// 데이터 벡터에서 MACD 지표 생성
    ///
    /// # Arguments
    /// * `data` - 캔들 데이터 벡터
    ///
    /// # Returns
    /// * `MACD` - 계산된 MACD 지표
    pub fn build(&mut self, data: &[C]) -> MACD {
        self.values.clear();
        for candle in data {
            self.values.push(candle.close_price());
        }

        let (macd_line, signal_line, histogram) = calculate_macd(
            &self.values,
            self.fast_period,
            self.slow_period,
            self.signal_period,
        );

        MACD {
            fast_period: self.fast_period,
            slow_period: self.slow_period,
            signal_period: self.signal_period,
            macd_line,
            signal_line,
            histogram,
        }
    }

    /// 새 캔들 데이터로 MACD 지표 업데이트
    ///
    /// # Arguments
    /// * `data` - 새 캔들 데이터
    ///
    /// # Returns
    /// * `MACD` - 업데이트된 MACD 지표
    pub fn next(&mut self, data: &C) -> MACD {
        self.values.push(data.close_price());

        // 필요한 데이터만 유지
        if self.values.len() > self.slow_period * 2 {
            self.values.remove(0);
        }

        let (macd_line, signal_line, histogram) = calculate_macd(
            &self.values,
            self.fast_period,
            self.slow_period,
            self.signal_period,
        );

        MACD {
            fast_period: self.fast_period,
            slow_period: self.slow_period,
            signal_period: self.signal_period,
            macd_line,
            signal_line,
            histogram,
        }
    }
}

impl<C> TABuilder<MACD, C> for MACDBuilder<C>
where
    C: Candle,
{
    fn from_storage(&mut self, storage: &CandleStore<C>) -> MACD {
        self.from_storage(storage)
    }

    fn build(&mut self, data: &[C]) -> MACD {
        self.build(data)
    }

    fn next(&mut self, data: &C) -> MACD {
        self.next(data)
    }
}

/// MACD 매개변수를 정의하는 구조체
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct MACDParams {
    /// 빠른 EMA 기간
    pub fast_period: usize,
    /// 느린 EMA 기간
    pub slow_period: usize,
    /// 시그널 라인 기간
    pub signal_period: usize,
}

impl Display for MACDParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MACD({},{},{})",
            self.fast_period, self.slow_period, self.signal_period
        )
    }
}

/// 여러 MACD 지표 컬렉션 타입
pub type MACDs = TAs<MACDParams, MACD>;

/// 여러 MACD 지표 빌더 타입
pub type MACDsBuilder<C> = TAsBuilder<MACDParams, MACD, C>;

/// MACD 컬렉션 빌더 팩토리
pub struct MACDsBuilderFactory;

impl MACDsBuilderFactory {
    /// 여러 MACD 매개변수 조합에 대한 빌더 생성
    ///
    /// # Arguments
    /// * `params` - MACD 매개변수 조합 목록
    ///
    /// # Returns
    /// * `MACDsBuilder` - 여러 MACD 빌더
    pub fn build<C: Candle + 'static>(params: &[MACDParams]) -> MACDsBuilder<C> {
        MACDsBuilder::new("macds".to_owned(), params, |param| {
            Box::new(MACDBuilder::new(
                param.fast_period,
                param.slow_period,
                param.signal_period,
            ))
        })
    }

    /// 기본 MACD 매개변수로 빌더 생성 (12, 26, 9)
    ///
    /// # Returns
    /// * `MACDsBuilder` - 기본 MACD 빌더
    pub fn build_default<C: Candle + 'static>() -> MACDsBuilder<C> {
        let default_params = vec![MACDParams {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
        }];

        Self::build(&default_params)
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
    fn test_macd_builder_new() {
        let builder = MACDBuilder::<TestCandle>::new(12, 26, 9);
        assert_eq!(builder.fast_period, 12);
        assert_eq!(builder.slow_period, 26);
        assert_eq!(builder.signal_period, 9);
    }

    #[test]
    #[should_panic(expected = "MACD 기간은 0보다 커야 합니다")]
    fn test_macd_builder_new_invalid_period() {
        MACDBuilder::<TestCandle>::new(0, 26, 9);
    }

    #[test]
    #[should_panic(expected = "빠른 기간은 느린 기간보다 작아야 합니다")]
    fn test_macd_builder_new_invalid_period_order() {
        MACDBuilder::<TestCandle>::new(26, 12, 9);
    }

    #[test]
    fn test_macd_build_empty_data() {
        let mut builder = MACDBuilder::<TestCandle>::new(12, 26, 9);
        let macd = builder.build(&[]);
        assert_eq!(macd.fast_period, 12);
        assert_eq!(macd.slow_period, 26);
        assert_eq!(macd.signal_period, 9);
        assert_eq!(macd.macd_line, 0.0);
        assert_eq!(macd.signal_line, 0.0);
        assert_eq!(macd.histogram, 0.0);
    }

    #[test]
    fn test_macd_build_with_data() {
        let mut builder = MACDBuilder::<TestCandle>::new(2, 3, 2);
        let candles = create_test_candles();
        let macd = builder.build(&candles);

        assert_eq!(macd.fast_period, 2);
        assert_eq!(macd.slow_period, 3);
        assert_eq!(macd.signal_period, 2);
        assert!(macd.macd_line != 0.0);
        assert!(macd.signal_line != 0.0);
        assert!(macd.histogram != 0.0);
    }

    #[test]
    fn test_macd_next() {
        let mut builder = MACDBuilder::<TestCandle>::new(12, 26, 9);
        let candles = create_test_candles();

        let macd = builder.next(&candles[0]);
        assert!(macd.macd_line >= -100.0 && macd.macd_line <= 100.0);
        assert!(macd.signal_line >= -100.0 && macd.signal_line <= 100.0);
        assert!(macd.histogram >= -100.0 && macd.histogram <= 100.0);
    }

    #[test]
    fn test_macd_display() {
        let macd = MACD {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
            macd_line: 1.5,
            signal_line: 1.0,
            histogram: 0.5,
        };

        assert_eq!(format!("{}", macd), "MACD(12,26,9: 1.50, 1.00, 0.50)");
    }

    #[test]
    fn test_macd_params_display() {
        let params = MACDParams {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
        };

        assert_eq!(format!("{}", params), "MACD(12,26,9)");
    }

    #[test]
    fn test_macd_calculation() {
        let mut builder = MACDBuilder::<TestCandle>::new(12, 26, 9);
        let candles = create_test_candles();

        // 첫 번째 MACD 계산
        let macd1 = builder.next(&candles[0]);
        assert_eq!(macd1.fast_period, 12);
        assert_eq!(macd1.slow_period, 26);
        assert_eq!(macd1.signal_period, 9);
        assert!(macd1.macd_line >= -100.0 && macd1.macd_line <= 100.0); // MACD 값 범위 검증
        assert!(macd1.signal_line >= -100.0 && macd1.signal_line <= 100.0); // 시그널 값 범위 검증
        assert!(macd1.histogram >= -100.0 && macd1.histogram <= 100.0); // 히스토그램 값 범위 검증
    }

    #[test]
    fn test_macd_trend_signals() {
        let mut builder = MACDBuilder::<TestCandle>::new(12, 26, 9);
        let candles = create_test_candles();

        let macd = builder.build(&candles);

        // MACD 값 범위 검증
        assert!(macd.macd_line >= -100.0 && macd.macd_line <= 100.0);
        assert!(macd.signal_line >= -100.0 && macd.signal_line <= 100.0);
        assert!(macd.histogram >= -100.0 && macd.histogram <= 100.0);

        // MACD와 시그널 라인의 관계 확인
        if macd.macd_line > macd.signal_line {
            assert!(macd.histogram > 0.0); // 상승 신호
        } else {
            assert!(macd.histogram <= 0.0); // 하락 신호
        }
    }

    #[test]
    fn test_macd_crossovers() {
        let mut builder = MACDBuilder::<TestCandle>::new(2, 3, 2);
        let candles = create_test_candles();

        // 첫 번째 MACD
        let macd1 = builder.next(&candles[0]);

        // 두 번째 MACD
        let macd2 = builder.next(&candles[1]);

        // 세 번째 MACD
        let macd3 = builder.next(&candles[2]);

        // MACD가 시그널선을 상향 돌파하는 경우
        if macd1.macd_line < macd1.signal_line && macd2.macd_line > macd2.signal_line {
            assert!(macd2.histogram > macd1.histogram); // 히스토그램이 증가
        }

        // MACD가 시그널선을 하향 돌파하는 경우
        if macd2.macd_line > macd2.signal_line && macd3.macd_line < macd3.signal_line {
            assert!(macd3.histogram < macd2.histogram); // 히스토그램이 감소
        }
    }

    #[test]
    fn test_macd_divergence() {
        let mut builder = MACDBuilder::<TestCandle>::new(2, 3, 2);

        // 상승 다이버전스 데이터 (가격은 하락하지만 MACD는 상승)
        let divergence_candles = vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 100.0,
                low: 90.0,
                close: 95.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 95.0,
                high: 95.0,
                low: 85.0,
                close: 90.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 90.0,
                high: 90.0,
                low: 80.0,
                close: 85.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 85.0,
                high: 85.0,
                low: 75.0,
                close: 80.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 80.0,
                high: 80.0,
                low: 70.0,
                close: 75.0,
                volume: 1000.0,
            },
        ];

        let mut macd_values = Vec::new();
        for candle in &divergence_candles {
            macd_values.push(builder.next(candle));
        }

        // 가격은 하락하지만 MACD 히스토그램의 하락폭이 감소하는지 확인
        let histogram_change1 = macd_values[2].histogram - macd_values[1].histogram;
        let histogram_change2 = macd_values[4].histogram - macd_values[3].histogram;

        // 두 번째 하락에서 히스토그램의 하락폭이 첫 번째 하락보다 작아야 함
        assert!(
            histogram_change2 > histogram_change1,
            "histogram_change2 ({}) should be greater than histogram_change1 ({})",
            histogram_change2,
            histogram_change1
        );
    }
}
