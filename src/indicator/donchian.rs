use crate::candle_store::CandleStore;
use crate::indicator::{IndicatorResult, TABuilder, TAs, TAsBuilder, checked_indicator_capacity};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

#[derive(Clone, Debug)]
struct DonchianInput {
    high: f64,
    low: f64,
}

/// Donchian Channel 계산 빌더
#[derive(Debug)]
pub struct DonchianChannelBuilder<C: Candle> {
    period: usize,
    values: Vec<DonchianInput>,
    _phantom: PhantomData<C>,
}

/// Donchian Channel 지표
#[derive(Clone, Debug)]
pub struct DonchianChannel {
    period: usize,
    sample_count: usize,
    pub upper: f64,
    pub lower: f64,
    pub middle: f64,
    pub width: f64,
}

impl DonchianChannel {
    pub fn period(&self) -> usize {
        self.period
    }

    pub fn sample_count(&self) -> usize {
        self.sample_count
    }

    pub fn upper(&self) -> f64 {
        self.upper
    }

    pub fn lower(&self) -> f64 {
        self.lower
    }

    pub fn middle(&self) -> f64 {
        self.middle
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn contains_price(&self, price: f64) -> bool {
        price >= self.lower && price <= self.upper
    }

    pub fn percent_b(&self, price: f64) -> f64 {
        if self.width.abs() < f64::EPSILON {
            return 0.5;
        }

        (price - self.lower) / self.width
    }
}

impl Display for DonchianChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DonchianChannel({}: {:.2}, {:.2}, {:.2})",
            self.period, self.upper, self.middle, self.lower
        )
    }
}

impl<C> DonchianChannelBuilder<C>
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
            return Err("Donchian Channel 기간은 0보다 커야 합니다".to_string());
        }

        let capacity = checked_indicator_capacity("Donchian Channel", period, 2, 0)?;

        Ok(Self {
            period,
            values: Vec::with_capacity(capacity),
            _phantom: PhantomData,
        })
    }

    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> DonchianChannel {
        self.build(&storage.get_ascending_items())
    }

    pub fn build(&mut self, data: &[C]) -> DonchianChannel {
        self.values.clear();

        if data.is_empty() {
            return self.calculate();
        }

        for item in data {
            self.push(item);
        }

        self.calculate()
    }

    pub fn next(&mut self, data: &C) -> DonchianChannel {
        self.push(data);
        self.calculate()
    }

    fn push(&mut self, data: &C) {
        self.values.push(DonchianInput {
            high: data.high_price(),
            low: data.low_price(),
        });

        if self.values.len() > self.period * 2 {
            let excess = self.values.len() - self.period * 2;
            self.values.drain(0..excess);
        }
    }

    fn calculate(&self) -> DonchianChannel {
        if self.values.is_empty() {
            return DonchianChannel {
                period: self.period,
                sample_count: 0,
                upper: 0.0,
                lower: 0.0,
                middle: 0.0,
                width: 0.0,
            };
        }

        let start_idx = self.values.len().saturating_sub(self.period);
        let slice = &self.values[start_idx..];
        let upper = slice
            .iter()
            .fold(f64::NEG_INFINITY, |current, item| current.max(item.high));
        let lower = slice
            .iter()
            .fold(f64::INFINITY, |current, item| current.min(item.low));
        let middle = (upper + lower) / 2.0;
        let width = upper - lower;

        DonchianChannel {
            period: self.period,
            sample_count: slice.len(),
            upper,
            lower,
            middle,
            width,
        }
    }
}

impl<C> TABuilder<DonchianChannel, C> for DonchianChannelBuilder<C>
where
    C: Candle,
{
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> DonchianChannel {
        self.build_from_storage(storage)
    }

    fn build(&mut self, data: &[C]) -> DonchianChannel {
        self.build(data)
    }

    fn next(&mut self, data: &C) -> DonchianChannel {
        self.next(data)
    }
}

pub type DonchianChannels = TAs<usize, DonchianChannel>;
pub type DonchianChannelsBuilder<C> = TAsBuilder<usize, DonchianChannel, C>;

pub struct DonchianChannelsBuilderFactory;
impl DonchianChannelsBuilderFactory {
    pub fn build<C: Candle + 'static>(periods: &[usize]) -> DonchianChannelsBuilder<C> {
        match Self::build_checked(periods) {
            Ok(builder) => builder,
            Err(message) => panic!("{message}"),
        }
    }

    pub fn build_checked<C: Candle + 'static>(
        periods: &[usize],
    ) -> IndicatorResult<DonchianChannelsBuilder<C>> {
        for period in periods {
            DonchianChannelBuilder::<C>::new_checked(*period)?;
        }

        Ok(DonchianChannelsBuilder::new(
            "donchian_channels".to_owned(),
            periods,
            |period| Box::new(DonchianChannelBuilder::<C>::new(*period)),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestCandle;

    fn test_candle(timestamp: i64, high: f64, low: f64, close: f64) -> TestCandle {
        TestCandle {
            timestamp,
            open: close,
            high,
            low,
            close,
            volume: 1.0,
        }
    }

    fn test_candles() -> Vec<TestCandle> {
        vec![
            test_candle(1, 110.0, 90.0, 100.0),
            test_candle(2, 115.0, 95.0, 105.0),
            test_candle(3, 120.0, 85.0, 110.0),
            test_candle(4, 108.0, 88.0, 100.0),
        ]
    }

    #[test]
    fn calculates_channel_from_recent_period() {
        let mut builder = DonchianChannelBuilder::<TestCandle>::new(3);
        let channel = builder.build(&test_candles());

        assert_eq!(channel.period(), 3);
        assert_eq!(channel.sample_count(), 3);
        assert_eq!(channel.upper(), 120.0);
        assert_eq!(channel.lower(), 85.0);
        assert_eq!(channel.middle(), 102.5);
        assert_eq!(channel.width(), 35.0);
    }

    #[test]
    fn next_updates_channel_incrementally() {
        let mut builder = DonchianChannelBuilder::<TestCandle>::new(2);
        let candles = test_candles();

        let first = builder.next(&candles[0]);
        assert_eq!(first.sample_count(), 1);
        assert_eq!(first.upper(), 110.0);
        assert_eq!(first.lower(), 90.0);

        let second = builder.next(&candles[1]);
        assert_eq!(second.upper(), 115.0);
        assert_eq!(second.lower(), 90.0);

        let third = builder.next(&candles[2]);
        assert_eq!(third.upper(), 120.0);
        assert_eq!(third.lower(), 85.0);
    }

    #[test]
    fn returns_empty_channel_for_empty_data() {
        let mut builder = DonchianChannelBuilder::<TestCandle>::new(20);
        let channel = builder.build(&[]);

        assert_eq!(channel.sample_count(), 0);
        assert_eq!(channel.upper(), 0.0);
        assert_eq!(channel.lower(), 0.0);
        assert_eq!(channel.middle(), 0.0);
        assert_eq!(channel.width(), 0.0);
    }

    #[test]
    fn calculates_price_position_in_channel() {
        let mut builder = DonchianChannelBuilder::<TestCandle>::new(3);
        let channel = builder.build(&test_candles());

        assert!(channel.contains_price(100.0));
        assert!(!channel.contains_price(130.0));
        assert!((channel.percent_b(102.5) - 0.5).abs() < 1e-10);
    }

    #[test]
    #[should_panic(expected = "Donchian Channel 기간은 0보다 커야 합니다")]
    fn rejects_zero_period() {
        DonchianChannelBuilder::<TestCandle>::new(0);
    }

    #[test]
    fn rejects_excessive_period() {
        assert!(DonchianChannelBuilder::<TestCandle>::new_checked(usize::MAX).is_err());
    }

    #[test]
    fn builds_multiple_channels() {
        let mut builder = DonchianChannelsBuilderFactory::build::<TestCandle>(&[2, 3]);
        let channels = builder.build(&test_candles());

        assert_eq!(channels.keys(), &vec![2, 3]);
        assert_eq!(channels.get(&2).upper(), 120.0);
        assert_eq!(channels.get(&3).lower(), 85.0);
    }
}
