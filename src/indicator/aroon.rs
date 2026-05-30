use crate::candle_store::CandleStore;
use crate::indicator::{IndicatorResult, TABuilder, TAs, TAsBuilder, checked_indicator_capacity};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

#[derive(Clone, Debug)]
struct AroonInput {
    high: f64,
    low: f64,
}

#[derive(Debug)]
pub struct AroonBuilder<C: Candle> {
    period: usize,
    values: Vec<AroonInput>,
    _phantom: PhantomData<C>,
}

#[derive(Clone, Debug)]
pub struct Aroon {
    period: usize,
    sample_count: usize,
    pub up: f64,
    pub down: f64,
    pub oscillator: f64,
}

impl Aroon {
    pub fn period(&self) -> usize {
        self.period
    }
    pub fn sample_count(&self) -> usize {
        self.sample_count
    }
    pub fn up(&self) -> f64 {
        self.up
    }
    pub fn down(&self) -> f64 {
        self.down
    }
    pub fn oscillator(&self) -> f64 {
        self.oscillator
    }
}

impl Display for Aroon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Aroon({}: {:.2}, {:.2}, {:.2})",
            self.period, self.up, self.down, self.oscillator
        )
    }
}

impl<C> AroonBuilder<C>
where
    C: Candle,
{
    pub fn new(period: usize) -> Self {
        match Self::new_checked(period) {
            Ok(builder) => builder,
            Err(message) => panic!("{message}"),
        }
    }
    pub fn new_checked(period: usize) -> IndicatorResult<Self> {
        if period == 0 {
            return Err("Aroon 기간은 0보다 커야 합니다".to_string());
        }
        let capacity = checked_indicator_capacity("Aroon", period, 2, 0)?;

        Ok(Self {
            period,
            values: Vec::with_capacity(capacity),
            _phantom: PhantomData,
        })
    }
    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> Aroon {
        self.build(&storage.get_ascending_items())
    }
    pub fn build(&mut self, data: &[C]) -> Aroon {
        self.values.clear();
        if data.is_empty() {
            return self.calculate();
        }
        for candle in data {
            self.push(candle);
        }
        self.calculate()
    }
    pub fn next(&mut self, data: &C) -> Aroon {
        self.push(data);
        self.calculate()
    }
    fn push(&mut self, data: &C) {
        self.values.push(AroonInput {
            high: data.high_price(),
            low: data.low_price(),
        });
        if self.values.len() > self.period * 2 {
            let excess = self.values.len() - self.period * 2;
            self.values.drain(0..excess);
        }
    }
    fn calculate(&self) -> Aroon {
        if self.values.is_empty() {
            return Aroon {
                period: self.period,
                sample_count: 0,
                up: 0.0,
                down: 0.0,
                oscillator: 0.0,
            };
        }
        // 표준 Aroon은 period+1개의 봉(현재부터 period봉 전까지)을 본다.
        // 캔들이 부족하면 가용한 만큼만 보며, days_since 분모는 (slice.len() - 1)을 사용한다.
        let start = self.values.len().saturating_sub(self.period + 1);
        let slice = &self.values[start..];
        let mut high_idx = 0usize;
        let mut low_idx = 0usize;
        for (idx, item) in slice.iter().enumerate() {
            // `>=` / `<=` 로 가장 최근(=높은 인덱스)을 선호 — 동률 시 표준과 일치한다.
            if item.high >= slice[high_idx].high {
                high_idx = idx;
            }
            if item.low <= slice[low_idx].low {
                low_idx = idx;
            }
        }
        // last_idx 가 가장 최근 봉(days_since = 0). high_idx 가 last_idx 면 100, 가장 오래된이면 0.
        let last_idx = slice.len().saturating_sub(1);
        let days_since_high = last_idx - high_idx;
        let days_since_low = last_idx - low_idx;
        // 분모는 윈도우의 봉 간격 수. period 만큼 채워졌으면 period, 미만이면 last_idx.
        let denominator = last_idx.max(1) as f64;
        let up = ((denominator - days_since_high as f64) / denominator) * 100.0;
        let down = ((denominator - days_since_low as f64) / denominator) * 100.0;
        Aroon {
            period: self.period,
            sample_count: slice.len(),
            up,
            down,
            oscillator: up - down,
        }
    }
}

impl<C> TABuilder<Aroon, C> for AroonBuilder<C>
where
    C: Candle,
{
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> Aroon {
        self.build_from_storage(storage)
    }
    fn build(&mut self, data: &[C]) -> Aroon {
        self.build(data)
    }
    fn next(&mut self, data: &C) -> Aroon {
        self.next(data)
    }
}

pub type Aroons = TAs<usize, Aroon>;
pub type AroonsBuilder<C> = TAsBuilder<usize, Aroon, C>;
pub struct AroonsBuilderFactory;
impl AroonsBuilderFactory {
    pub fn build<C: Candle + 'static>(periods: &[usize]) -> AroonsBuilder<C> {
        match Self::build_checked(periods) {
            Ok(builder) => builder,
            Err(message) => panic!("{message}"),
        }
    }
    pub fn build_checked<C: Candle + 'static>(
        periods: &[usize],
    ) -> IndicatorResult<AroonsBuilder<C>> {
        for period in periods {
            AroonBuilder::<C>::new_checked(*period)?;
        }
        Ok(AroonsBuilder::new("aroons".to_owned(), periods, |period| {
            Box::new(AroonBuilder::<C>::new(*period))
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestCandle;
    fn candle(timestamp: i64, high: f64, low: f64) -> TestCandle {
        TestCandle {
            timestamp,
            open: 0.0,
            high,
            low,
            close: 0.0,
            volume: 1.0,
        }
    }
    #[test]
    fn rejects_zero_period() {
        assert!(AroonBuilder::<TestCandle>::new_checked(0).is_err());
        assert!(AroonBuilder::<TestCandle>::new_checked(usize::MAX).is_err());
    }
    #[test]
    fn calculates_aroon_with_most_recent_tie() {
        let data = vec![
            candle(1, 10.0, 5.0),
            candle(2, 12.0, 6.0),
            candle(3, 12.0, 4.0),
        ];
        let mut builder = AroonBuilder::<TestCandle>::new(3);
        let aroon = builder.build(&data);
        assert_eq!(aroon.up(), 100.0);
        assert_eq!(aroon.down(), 100.0);
        assert_eq!(aroon.oscillator(), 0.0);
    }

    #[test]
    fn standard_aroon_when_oldest_is_extreme() {
        // 표준 Aroon: period=4 윈도우(5봉)에서 가장 오래된 봉이 high면
        // days_since = 4, up = (4-4)/4 * 100 = 0
        // 가장 최근 봉이 low면 down = 100
        let data = vec![
            candle(1, 50.0, 30.0), // 가장 오래된이 최고가
            candle(2, 40.0, 32.0),
            candle(3, 38.0, 31.0),
            candle(4, 36.0, 29.0),
            candle(5, 35.0, 25.0), // 가장 최근이 최저가
        ];
        let mut builder = AroonBuilder::<TestCandle>::new(4);
        let aroon = builder.build(&data);
        assert!((aroon.up() - 0.0).abs() < 1e-9, "got up={}", aroon.up());
        assert!((aroon.down() - 100.0).abs() < 1e-9, "got down={}", aroon.down());
    }

    #[test]
    fn standard_aroon_intermediate_position() {
        // period=4 윈도우(5봉), high가 인덱스 2(=가운데)에 있음 → days_since = 2
        // up = (4-2)/4 * 100 = 50
        let data = vec![
            candle(1, 30.0, 20.0),
            candle(2, 32.0, 22.0),
            candle(3, 50.0, 24.0), // 인덱스 2: 최고가
            candle(4, 40.0, 23.0),
            candle(5, 35.0, 21.0), // 가장 최근
        ];
        let mut builder = AroonBuilder::<TestCandle>::new(4);
        let aroon = builder.build(&data);
        assert!((aroon.up() - 50.0).abs() < 1e-9, "expected 50.0, got {}", aroon.up());
    }
}
