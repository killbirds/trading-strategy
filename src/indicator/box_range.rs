use crate::candle_store::CandleStore;
use crate::indicator::{IndicatorResult, TABuilder, TAs, TAsBuilder};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

#[derive(Clone, Debug, PartialEq)]
pub enum BoxRangeBreakout {
    Above,
    Below,
    Inside,
}

#[derive(Clone, Debug)]
struct BoxRangeInput {
    high: f64,
    low: f64,
}

#[derive(Debug)]
pub struct BoxRangeBuilder<C: Candle> {
    period: usize,
    max_width_ratio: f64,
    values: Vec<BoxRangeInput>,
    _phantom: PhantomData<C>,
}

#[derive(Clone, Debug)]
pub struct BoxRange {
    period: usize,
    max_width_ratio: f64,
    sample_count: usize,
    pub upper: f64,
    pub lower: f64,
    pub middle: f64,
    pub width: f64,
    pub width_ratio: f64,
    pub is_box_range: bool,
}

impl BoxRange {
    pub fn period(&self) -> usize {
        self.period
    }

    pub fn max_width_ratio(&self) -> f64 {
        self.max_width_ratio
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

    pub fn width_ratio(&self) -> f64 {
        self.width_ratio
    }

    pub fn is_box_range(&self) -> bool {
        self.is_box_range
    }

    pub fn contains_price(&self, price: f64) -> bool {
        price >= self.lower && price <= self.upper
    }

    pub fn breakout_direction(&self, price: f64) -> BoxRangeBreakout {
        if price > self.upper {
            BoxRangeBreakout::Above
        } else if price < self.lower {
            BoxRangeBreakout::Below
        } else {
            BoxRangeBreakout::Inside
        }
    }
}

impl Display for BoxRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BoxRange({},{}: {:.2}, {:.2}, {:.4}, {})",
            self.period,
            self.max_width_ratio,
            self.upper,
            self.lower,
            self.width_ratio,
            self.is_box_range
        )
    }
}

impl<C> BoxRangeBuilder<C>
where
    C: Candle,
{
    pub fn new(period: usize, max_width_ratio: f64) -> Self {
        match Self::new_checked(period, max_width_ratio) {
            Ok(builder) => builder,
            Err(message) => panic!("{message}"),
        }
    }

    pub fn new_checked(period: usize, max_width_ratio: f64) -> IndicatorResult<Self> {
        if period == 0 {
            return Err("박스권 기간은 0보다 커야 합니다".to_string());
        }

        if max_width_ratio <= 0.0 || max_width_ratio.is_nan() || max_width_ratio.is_infinite() {
            return Err("박스권 최대 폭 비율은 유한한 양수여야 합니다".to_string());
        }

        Ok(Self {
            period,
            max_width_ratio,
            values: Vec::with_capacity(period * 2),
            _phantom: PhantomData,
        })
    }

    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> BoxRange {
        self.build(&storage.get_ascending_items())
    }

    pub fn build(&mut self, data: &[C]) -> BoxRange {
        self.values.clear();

        if data.is_empty() {
            return self.calculate();
        }

        for item in data {
            self.push(item);
        }

        self.calculate()
    }

    pub fn next(&mut self, data: &C) -> BoxRange {
        self.push(data);
        self.calculate()
    }

    fn push(&mut self, data: &C) {
        self.values.push(BoxRangeInput {
            high: data.high_price(),
            low: data.low_price(),
        });

        if self.values.len() > self.period * 2 {
            let excess = self.values.len() - self.period * 2;
            self.values.drain(0..excess);
        }
    }

    fn calculate(&self) -> BoxRange {
        if self.values.is_empty() {
            return BoxRange {
                period: self.period,
                max_width_ratio: self.max_width_ratio,
                sample_count: 0,
                upper: 0.0,
                lower: 0.0,
                middle: 0.0,
                width: 0.0,
                width_ratio: 0.0,
                is_box_range: false,
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
        let width_ratio = if middle.abs() < f64::EPSILON {
            0.0
        } else {
            width / middle.abs()
        };
        let has_enough_samples = slice.len() >= self.period;
        let is_box_range = has_enough_samples && width_ratio <= self.max_width_ratio;

        BoxRange {
            period: self.period,
            max_width_ratio: self.max_width_ratio,
            sample_count: slice.len(),
            upper,
            lower,
            middle,
            width,
            width_ratio,
            is_box_range,
        }
    }
}

impl<C> TABuilder<BoxRange, C> for BoxRangeBuilder<C>
where
    C: Candle,
{
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> BoxRange {
        self.build_from_storage(storage)
    }

    fn build(&mut self, data: &[C]) -> BoxRange {
        self.build(data)
    }

    fn next(&mut self, data: &C) -> BoxRange {
        self.next(data)
    }
}

pub type BoxRanges = TAs<usize, BoxRange>;
pub type BoxRangesBuilder<C> = TAsBuilder<usize, BoxRange, C>;

pub struct BoxRangesBuilderFactory;
impl BoxRangesBuilderFactory {
    pub fn build<C: Candle + 'static>(
        periods: &[usize],
        max_width_ratio: f64,
    ) -> BoxRangesBuilder<C> {
        match Self::build_checked(periods, max_width_ratio) {
            Ok(builder) => builder,
            Err(message) => panic!("{message}"),
        }
    }

    pub fn build_checked<C: Candle + 'static>(
        periods: &[usize],
        max_width_ratio: f64,
    ) -> IndicatorResult<BoxRangesBuilder<C>> {
        for period in periods {
            BoxRangeBuilder::<C>::new_checked(*period, max_width_ratio)?;
        }

        Ok(BoxRangesBuilder::new(
            "box_ranges".to_owned(),
            periods,
            move |period| Box::new(BoxRangeBuilder::<C>::new(*period, max_width_ratio)),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestCandle;

    fn candle(timestamp: i64, high: f64, low: f64, close: f64) -> TestCandle {
        TestCandle {
            timestamp,
            open: close,
            high,
            low,
            close,
            volume: 1000.0,
        }
    }

    fn box_candles() -> Vec<TestCandle> {
        vec![
            candle(1, 101.0, 99.0, 100.0),
            candle(2, 102.0, 98.5, 101.0),
            candle(3, 101.5, 99.5, 100.5),
            candle(4, 102.0, 98.0, 99.5),
            candle(5, 101.0, 99.0, 100.0),
        ]
    }

    #[test]
    fn test_box_range_builder_new() {
        let builder = BoxRangeBuilder::<TestCandle>::new(20, 0.05);
        assert_eq!(builder.period, 20);
        assert_eq!(builder.max_width_ratio, 0.05);
    }

    #[test]
    #[should_panic(expected = "박스권 기간은 0보다 커야 합니다")]
    fn test_box_range_builder_invalid_period() {
        BoxRangeBuilder::<TestCandle>::new(0, 0.05);
    }

    #[test]
    #[should_panic(expected = "박스권 최대 폭 비율은 유한한 양수여야 합니다")]
    fn test_box_range_builder_invalid_width_ratio() {
        BoxRangeBuilder::<TestCandle>::new(20, 0.0);
    }

    #[test]
    fn test_box_range_build_empty_data() {
        let mut builder = BoxRangeBuilder::<TestCandle>::new(5, 0.05);
        let range = builder.build(&[]);

        assert_eq!(range.period(), 5);
        assert_eq!(range.sample_count(), 0);
        assert_eq!(range.upper(), 0.0);
        assert_eq!(range.lower(), 0.0);
        assert!(!range.is_box_range());
    }

    #[test]
    fn test_box_range_detects_sideways_range() {
        let mut builder = BoxRangeBuilder::<TestCandle>::new(5, 0.05);
        let range = builder.build(&box_candles());

        assert_eq!(range.sample_count(), 5);
        assert_eq!(range.upper(), 102.0);
        assert_eq!(range.lower(), 98.0);
        assert_eq!(range.middle(), 100.0);
        assert_eq!(range.width(), 4.0);
        assert_eq!(range.width_ratio(), 0.04);
        assert!(range.is_box_range());
        assert!(range.contains_price(100.0));
        assert_eq!(range.breakout_direction(103.0), BoxRangeBreakout::Above);
        assert_eq!(range.breakout_direction(97.0), BoxRangeBreakout::Below);
        assert_eq!(range.breakout_direction(100.0), BoxRangeBreakout::Inside);
    }

    #[test]
    fn test_box_range_rejects_wide_range() {
        let mut builder = BoxRangeBuilder::<TestCandle>::new(5, 0.05);
        let mut candles = box_candles();
        candles.push(candle(6, 130.0, 80.0, 110.0));
        let range = builder.build(&candles);

        assert_eq!(range.sample_count(), 5);
        assert_eq!(range.upper(), 130.0);
        assert_eq!(range.lower(), 80.0);
        assert!(!range.is_box_range());
    }

    #[test]
    fn test_box_range_waits_until_period_is_filled() {
        let mut builder = BoxRangeBuilder::<TestCandle>::new(5, 0.05);
        let candles = box_candles();

        for item in candles.iter().take(4) {
            let range = builder.next(item);
            assert!(!range.is_box_range());
        }

        let range = builder.next(&candles[4]);
        assert!(range.is_box_range());
    }

    #[test]
    fn test_box_ranges_builder_factory() {
        let mut builder = BoxRangesBuilderFactory::build::<TestCandle>(&[3, 5], 0.05);
        let candles = box_candles();
        let ranges = builder.build(&candles);

        assert_eq!(ranges.len(), 2);
        assert!(ranges.get(&3).is_box_range());
        assert!(ranges.get(&5).is_box_range());
    }
}
