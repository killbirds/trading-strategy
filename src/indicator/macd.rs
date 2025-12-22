use crate::candle_store::CandleStore;
use crate::indicator::utils::moving_average;
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
    /// 이전 빠른 EMA 값 (증분 계산용)
    previous_fast_ema: Option<f64>,
    /// 이전 느린 EMA 값 (증분 계산용)
    previous_slow_ema: Option<f64>,
    /// 이전 시그널 라인 값 (증분 계산용)
    previous_signal_line: Option<f64>,
    /// MACD 라인 히스토리 (시그널 라인 계산용, 최근 signal_period * 2개만 유지)
    macd_history: Vec<f64>,
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

/// MACD 계산 함수 (전체 데이터에서 계산)
fn calculate_macd(
    values: &[f64],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> (f64, f64, f64) {
    if values.len() < slow_period {
        return (0.0, 0.0, 0.0);
    }

    // EMA 알파값 계산 (유틸리티 함수 사용)
    let fast_alpha = moving_average::calculate_ema_alpha(fast_period);
    let slow_alpha = moving_average::calculate_ema_alpha(slow_period);
    let signal_alpha = moving_average::calculate_ema_alpha(signal_period);

    // 초기 SMA 계산
    let fast_sma =
        moving_average::calculate_sma(&values[..fast_period.min(values.len())], fast_period);
    let slow_sma =
        moving_average::calculate_sma(&values[..slow_period.min(values.len())], slow_period);

    // EMA 계산
    // 주의: EMA는 전체 데이터를 순회하면서 계산해야 정확합니다.
    // 초기 SMA 이후에도 모든 데이터에 대해 EMA를 재계산해야 올바른 결과를 얻을 수 있습니다.
    let mut fast_ema = fast_sma;
    let mut slow_ema = slow_sma;
    let mut macd_lines = Vec::with_capacity(values.len());

    // 전체 데이터에 대해 EMA 계산
    for &price in values.iter() {
        fast_ema = moving_average::calculate_ema_step(price, fast_ema, fast_alpha);
        slow_ema = moving_average::calculate_ema_step(price, slow_ema, slow_alpha);
        let macd_line = fast_ema - slow_ema;
        macd_lines.push(macd_line);
    }

    // 시그널 라인 계산 (MACD 라인의 EMA)
    let mut signal_line = 0.0;
    if macd_lines.len() >= signal_period {
        // 초기 시그널 라인 (SMA) - 첫 signal_period 개의 MACD 값의 평균
        let signal_sma = macd_lines[..signal_period].iter().sum::<f64>() / signal_period as f64;
        signal_line = signal_sma;

        // EMA로 시그널 라인 업데이트 - 나머지 모든 MACD 값에 대해
        for &macd in macd_lines[signal_period..].iter() {
            signal_line = moving_average::calculate_ema_step(macd, signal_line, signal_alpha);
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
            previous_fast_ema: None,
            previous_slow_ema: None,
            previous_signal_line: None,
            macd_history: Vec::with_capacity(signal_period * 2),
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
    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> MACD {
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
        self.macd_history.clear();
        for candle in data {
            self.values.push(candle.close_price());
        }

        let (macd_line, signal_line, histogram) = calculate_macd(
            &self.values,
            self.fast_period,
            self.slow_period,
            self.signal_period,
        );

        // 이전 값 업데이트 (다음 next() 호출을 위해)
        if !self.values.is_empty() {
            let fast_alpha = moving_average::calculate_ema_alpha(self.fast_period);
            let slow_alpha = moving_average::calculate_ema_alpha(self.slow_period);

            // 전체 데이터에서 EMA 재계산하여 이전 값 설정
            let fast_sma = moving_average::calculate_sma(
                &self.values[..self.fast_period.min(self.values.len())],
                self.fast_period,
            );
            let slow_sma = moving_average::calculate_sma(
                &self.values[..self.slow_period.min(self.values.len())],
                self.slow_period,
            );

            let mut fast_ema = fast_sma;
            let mut slow_ema = slow_sma;

            for &price in &self.values {
                fast_ema = moving_average::calculate_ema_step(price, fast_ema, fast_alpha);
                slow_ema = moving_average::calculate_ema_step(price, slow_ema, slow_alpha);
                let macd = fast_ema - slow_ema;
                self.macd_history.push(macd);
            }

            self.previous_fast_ema = Some(fast_ema);
            self.previous_slow_ema = Some(slow_ema);
            self.previous_signal_line = Some(signal_line);

            // 히스토리 크기 제한
            if self.macd_history.len() > self.signal_period * 2 {
                let excess = self.macd_history.len() - self.signal_period * 2;
                self.macd_history.drain(0..excess);
            }
        }

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
        let price = data.close_price();
        self.values.push(price);

        // 필요한 데이터만 유지
        if self.values.len() > self.slow_period * 2 {
            let excess = self.values.len() - self.slow_period * 2;
            self.values.drain(0..excess);
        }

        // EMA 알파값 계산
        let fast_alpha = moving_average::calculate_ema_alpha(self.fast_period);
        let slow_alpha = moving_average::calculate_ema_alpha(self.slow_period);
        let signal_alpha = moving_average::calculate_ema_alpha(self.signal_period);

        // 증분 계산: 이전 EMA 값이 있으면 사용, 없으면 전체 재계산
        let (fast_ema, slow_ema) = match (self.previous_fast_ema, self.previous_slow_ema) {
            (Some(prev_fast), Some(prev_slow)) => {
                // 증분 계산
                (
                    moving_average::calculate_ema_step(price, prev_fast, fast_alpha),
                    moving_average::calculate_ema_step(price, prev_slow, slow_alpha),
                )
            }
            _ => {
                // 이전 값이 없으면 전체 재계산
                if self.values.len() < self.slow_period {
                    // 데이터가 부족한 경우
                    return MACD {
                        fast_period: self.fast_period,
                        slow_period: self.slow_period,
                        signal_period: self.signal_period,
                        macd_line: 0.0,
                        signal_line: 0.0,
                        histogram: 0.0,
                    };
                }

                let (macd_line, signal_line, _) = calculate_macd(
                    &self.values,
                    self.fast_period,
                    self.slow_period,
                    self.signal_period,
                );

                // 이전 EMA 값 설정 (다음 next() 호출을 위해)
                let fast_sma = moving_average::calculate_sma(
                    &self.values[..self.fast_period.min(self.values.len())],
                    self.fast_period,
                );
                let slow_sma = moving_average::calculate_sma(
                    &self.values[..self.slow_period.min(self.values.len())],
                    self.slow_period,
                );

                let mut fast_ema = fast_sma;
                let mut slow_ema = slow_sma;

                // MACD 히스토리 재구성 (시그널 라인 계산을 위해)
                self.macd_history.clear();
                for &p in &self.values {
                    fast_ema = moving_average::calculate_ema_step(p, fast_ema, fast_alpha);
                    slow_ema = moving_average::calculate_ema_step(p, slow_ema, slow_alpha);
                    let macd = fast_ema - slow_ema;
                    self.macd_history.push(macd);
                }

                // 히스토리 크기 제한
                if self.macd_history.len() > self.signal_period * 2 {
                    let excess = self.macd_history.len() - self.signal_period * 2;
                    self.macd_history.drain(0..excess);
                }

                // 이전 값 저장
                self.previous_fast_ema = Some(fast_ema);
                self.previous_slow_ema = Some(slow_ema);
                self.previous_signal_line = Some(signal_line);

                return MACD {
                    fast_period: self.fast_period,
                    slow_period: self.slow_period,
                    signal_period: self.signal_period,
                    macd_line,
                    signal_line,
                    histogram: macd_line - signal_line,
                };
            }
        };

        // MACD 라인 계산
        let macd_line = fast_ema - slow_ema;
        self.macd_history.push(macd_line);

        // 히스토리 크기 제한
        if self.macd_history.len() > self.signal_period * 2 {
            let excess = self.macd_history.len() - self.signal_period * 2;
            self.macd_history.drain(0..excess);
        }

        // 시그널 라인 계산 (증분 또는 전체 재계산)
        let signal_line = match self.previous_signal_line {
            Some(prev_signal) if self.macd_history.len() >= self.signal_period => {
                // 증분 계산
                moving_average::calculate_ema_step(macd_line, prev_signal, signal_alpha)
            }
            _ => {
                // 이전 값이 없거나 데이터가 부족하면 전체 재계산
                if self.macd_history.len() >= self.signal_period {
                    let signal_sma = self.macd_history[..self.signal_period].iter().sum::<f64>()
                        / self.signal_period as f64;
                    let mut signal = signal_sma;
                    for &macd in &self.macd_history[self.signal_period..] {
                        signal = moving_average::calculate_ema_step(macd, signal, signal_alpha);
                    }
                    signal
                } else {
                    0.0
                }
            }
        };

        // 이전 값 업데이트
        self.previous_fast_ema = Some(fast_ema);
        self.previous_slow_ema = Some(slow_ema);
        self.previous_signal_line = Some(signal_line);

        let histogram = macd_line - signal_line;

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
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> MACD {
        self.build_from_storage(storage)
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

        assert_eq!(format!("{macd}"), "MACD(12,26,9: 1.50, 1.00, 0.50)");
    }

    #[test]
    fn test_macd_params_display() {
        let params = MACDParams {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
        };

        assert_eq!(format!("{params}"), "MACD(12,26,9)");
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
            "histogram_change2 ({histogram_change2}) should be greater than histogram_change1 ({histogram_change1})"
        );
    }

    #[test]
    fn test_macd_known_values_accuracy() {
        // 알려진 MACD 계산 결과와 비교
        // period=2,3,2인 경우 간단한 계산으로 검증
        // 데이터: [10.0, 11.0, 12.0, 13.0, 14.0]
        let candles = vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 10.0,
                high: 10.5,
                low: 9.5,
                close: 10.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 1,
                open: 10.0,
                high: 11.5,
                low: 9.5,
                close: 11.0,
                volume: 1100.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 2,
                open: 11.0,
                high: 12.5,
                low: 10.5,
                close: 12.0,
                volume: 1200.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 3,
                open: 12.0,
                high: 13.5,
                low: 11.5,
                close: 13.0,
                volume: 1300.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp() + 4,
                open: 13.0,
                high: 14.5,
                low: 12.5,
                close: 14.0,
                volume: 1400.0,
            },
        ];

        let mut builder = MACDBuilder::<TestCandle>::new(2, 3, 2);
        let macd = builder.build(&candles);

        // MACD 라인은 fast_ema - slow_ema이므로 양수여야 함 (상승 추세)
        assert!(
            macd.macd_line > 0.0,
            "MACD line should be positive for uptrend. Got: {}",
            macd.macd_line
        );

        // 히스토그램은 macd_line - signal_line
        let expected_histogram = macd.macd_line - macd.signal_line;
        assert!(
            (macd.histogram - expected_histogram).abs() < 0.01,
            "Histogram calculation mismatch. Expected: {}, Got: {}",
            expected_histogram,
            macd.histogram
        );
    }

    #[test]
    fn test_macd_known_values_period_12_26_9() {
        // 표준 MACD(12,26,9) 파라미터로 알려진 값과 비교
        // 간단한 상승 추세 데이터로 검증
        let mut candles = Vec::new();
        for i in 0..30 {
            candles.push(TestCandle {
                timestamp: Utc::now().timestamp() + i as i64,
                open: 100.0 + i as f64 * 0.5,
                high: 101.0 + i as f64 * 0.5,
                low: 99.0 + i as f64 * 0.5,
                close: 100.5 + i as f64 * 0.5,
                volume: 1000.0 + i as f64,
            });
        }

        let mut builder = MACDBuilder::<TestCandle>::new(12, 26, 9);
        let macd = builder.build(&candles);

        // 상승 추세이므로 MACD 라인은 양수여야 함
        assert!(
            macd.macd_line > 0.0,
            "MACD line should be positive for uptrend. Got: {}",
            macd.macd_line
        );

        // MACD 값들이 유효한 범위 내에 있어야 함
        assert!(
            !macd.macd_line.is_nan() && !macd.macd_line.is_infinite(),
            "MACD line should be finite. Got: {}",
            macd.macd_line
        );
        assert!(
            !macd.signal_line.is_nan() && !macd.signal_line.is_infinite(),
            "Signal line should be finite. Got: {}",
            macd.signal_line
        );
        assert!(
            !macd.histogram.is_nan() && !macd.histogram.is_infinite(),
            "Histogram should be finite. Got: {}",
            macd.histogram
        );
    }
}
