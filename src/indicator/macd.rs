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
    prices: Vec<f64>,
    /// 빠른 EMA 값 저장
    fast_ema_values: Vec<f64>,
    /// 느린 EMA 값 저장
    slow_ema_values: Vec<f64>,
    /// MACD 값 저장
    macd_values: Vec<f64>,
    /// 마지막 MACD 값
    last_macd: f64,
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
    pub macd: f64,
    /// 시그널 라인 (MACD의 EMA)
    pub signal: f64,
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
            self.macd,
            self.signal,
            self.histogram
        )
    }
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

        MACDBuilder {
            fast_period,
            slow_period,
            signal_period,
            prices: Vec::with_capacity(slow_period * 3),
            fast_ema_values: Vec::with_capacity(slow_period),
            slow_ema_values: Vec::with_capacity(slow_period),
            macd_values: Vec::with_capacity(slow_period),
            last_macd: 0.0,
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
        self.build(&storage.get_reversed_items())
    }

    /// EMA 계산 헬퍼 함수
    fn calculate_ema(values: &[f64], period: usize, prev_ema: Option<f64>) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        // 데이터가 충분하지 않으면 단순 평균 반환
        if values.len() < period {
            let sum: f64 = values.iter().sum();
            return sum / values.len() as f64;
        }

        // 새 값
        let current = values.last().unwrap();

        // 이전 EMA가 있으면 계속 계산
        if let Some(prev) = prev_ema {
            let multiplier = 2.0 / (period as f64 + 1.0);
            return current * multiplier + prev * (1.0 - multiplier);
        }

        // 이전 EMA가 없으면 처음부터 계산
        let first_period_values = &values[values.len() - period..];
        let sum: f64 = first_period_values.iter().sum();
        let sma = sum / period as f64;

        // 나머지 값에 대해 EMA 계산
        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = sma;

        for value in values.iter().skip(period) {
            ema = value * multiplier + ema * (1.0 - multiplier);
        }

        ema
    }

    /// 데이터 벡터에서 MACD 지표 생성
    ///
    /// # Arguments
    /// * `data` - 캔들 데이터 벡터
    ///
    /// # Returns
    /// * `MACD` - 계산된 MACD 지표
    pub fn build(&mut self, data: &[C]) -> MACD {
        if data.is_empty() {
            return MACD {
                fast_period: self.fast_period,
                slow_period: self.slow_period,
                signal_period: self.signal_period,
                macd: 0.0,
                signal: 0.0,
                histogram: 0.0,
            };
        }

        // 데이터를 prices 배열에 저장
        self.prices.clear();
        self.fast_ema_values.clear();
        self.slow_ema_values.clear();
        self.macd_values.clear();

        for item in data {
            self.prices.push(item.close_price());
        }

        // 충분한 데이터가 없는 경우
        if self.prices.len() < self.slow_period {
            return MACD {
                fast_period: self.fast_period,
                slow_period: self.slow_period,
                signal_period: self.signal_period,
                macd: 0.0,
                signal: 0.0,
                histogram: 0.0,
            };
        }

        // 빠른/느린 EMA 계산
        let mut fast_ema = None;
        let mut slow_ema = None;

        for i in 0..self.prices.len() {
            let price_slice = &self.prices[0..=i];

            // 빠른 EMA 계산
            fast_ema = Some(Self::calculate_ema(price_slice, self.fast_period, fast_ema));
            self.fast_ema_values.push(fast_ema.unwrap());

            // 느린 EMA 계산
            slow_ema = Some(Self::calculate_ema(price_slice, self.slow_period, slow_ema));
            self.slow_ema_values.push(slow_ema.unwrap());

            // MACD 값 계산
            let macd_value = fast_ema.unwrap() - slow_ema.unwrap();
            self.macd_values.push(macd_value);
        }

        // 마지막 MACD 값
        let macd_value = *self.macd_values.last().unwrap_or(&0.0);

        // 시그널 라인 (MACD의 EMA) 계산
        let signal_value = Self::calculate_ema(&self.macd_values, self.signal_period, None);

        // 히스토그램 계산
        let histogram_value = macd_value - signal_value;

        self.last_macd = macd_value;

        MACD {
            fast_period: self.fast_period,
            slow_period: self.slow_period,
            signal_period: self.signal_period,
            macd: macd_value,
            signal: signal_value,
            histogram: histogram_value,
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
        // 새 가격 추가
        let price = data.close_price();
        self.prices.push(price);

        // 필요한 데이터만 유지
        if self.prices.len() > self.slow_period * 3 {
            let excess = self.prices.len() - self.slow_period * 3;
            self.prices.drain(0..excess);
        }

        // 빠른 EMA 계산
        let prev_fast_ema = self.fast_ema_values.last().copied();
        let fast_ema = Self::calculate_ema(&self.prices, self.fast_period, prev_fast_ema);
        self.fast_ema_values.push(fast_ema);

        // 느린 EMA 계산
        let prev_slow_ema = self.slow_ema_values.last().copied();
        let slow_ema = Self::calculate_ema(&self.prices, self.slow_period, prev_slow_ema);
        self.slow_ema_values.push(slow_ema);

        // MACD 값 계산
        let macd_value = fast_ema - slow_ema;
        self.macd_values.push(macd_value);

        // 필요한 데이터만 유지
        if self.fast_ema_values.len() > self.slow_period * 3 {
            let excess = self.fast_ema_values.len() - self.slow_period * 3;
            self.fast_ema_values.drain(0..excess);
            self.slow_ema_values.drain(0..excess);
            self.macd_values.drain(0..excess);
        }

        // 시그널 라인 (MACD의 EMA) 계산
        let signal_value = Self::calculate_ema(&self.macd_values, self.signal_period, None);

        // 히스토그램 계산
        let histogram_value = macd_value - signal_value;

        self.last_macd = macd_value;

        MACD {
            fast_period: self.fast_period,
            slow_period: self.slow_period,
            signal_period: self.signal_period,
            macd: macd_value,
            signal: signal_value,
            histogram: histogram_value,
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
