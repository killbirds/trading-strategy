use crate::candle_store::CandleStore;
use crate::indicator::{IndicatorResult, TABuilder};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

#[derive(Debug)]
pub struct ParabolicSARBuilder<C: Candle> {
    step: f64,
    max_step: f64,
    initialized: bool,
    is_long: bool,
    sar: f64,
    extreme_point: f64,
    acceleration_factor: f64,
    previous_high: Option<f64>,
    previous_low: Option<f64>,
    second_previous_high: Option<f64>,
    second_previous_low: Option<f64>,
    _phantom: PhantomData<C>,
}

#[derive(Clone, Debug)]
pub struct ParabolicSAR {
    step: f64,
    max_step: f64,
    pub value: f64,
    pub is_long: bool,
    pub acceleration_factor: f64,
    pub extreme_point: f64,
}

impl ParabolicSAR {
    pub fn step(&self) -> f64 {
        self.step
    }
    pub fn max_step(&self) -> f64 {
        self.max_step
    }
    pub fn value(&self) -> f64 {
        self.value
    }
    pub fn is_long(&self) -> bool {
        self.is_long
    }
    pub fn acceleration_factor(&self) -> f64 {
        self.acceleration_factor
    }
    pub fn extreme_point(&self) -> f64 {
        self.extreme_point
    }
}

impl Display for ParabolicSAR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ParabolicSAR({:.2}/{:.2}: {:.2}, long={})",
            self.step, self.max_step, self.value, self.is_long
        )
    }
}

impl<C> ParabolicSARBuilder<C>
where
    C: Candle,
{
    pub fn new(step: f64, max_step: f64) -> Self {
        match Self::new_checked(step, max_step) {
            Ok(builder) => builder,
            Err(message) => panic!("{message}"),
        }
    }
    pub fn new_checked(step: f64, max_step: f64) -> IndicatorResult<Self> {
        if step <= 0.0 || max_step <= 0.0 || !step.is_finite() || !max_step.is_finite() {
            return Err("Parabolic SAR step 값은 유효한 양수여야 합니다".to_string());
        }
        if step > max_step {
            return Err("Parabolic SAR step은 max_step보다 작거나 같아야 합니다".to_string());
        }
        Ok(Self {
            step,
            max_step,
            initialized: false,
            is_long: true,
            sar: 0.0,
            extreme_point: 0.0,
            acceleration_factor: step,
            previous_high: None,
            previous_low: None,
            second_previous_high: None,
            second_previous_low: None,
            _phantom: PhantomData,
        })
    }
    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> ParabolicSAR {
        self.build(&storage.get_ascending_items())
    }
    pub fn build(&mut self, data: &[C]) -> ParabolicSAR {
        self.reset();
        if data.is_empty() {
            return self.current();
        }
        let mut result = self.current();
        for candle in data {
            result = self.next(candle);
        }
        result
    }
    pub fn next(&mut self, data: &C) -> ParabolicSAR {
        let high = data.high_price();
        let low = data.low_price();
        if !self.initialized {
            self.initialized = true;
            self.sar = low;
            self.extreme_point = high;
            self.previous_high = Some(high);
            self.previous_low = Some(low);
            self.second_previous_high = None;
            self.second_previous_low = None;
            return self.current();
        }
        let mut next_sar = self.sar + self.acceleration_factor * (self.extreme_point - self.sar);
        if self.is_long {
            if let Some(previous_low) = self.previous_low {
                next_sar = next_sar.min(previous_low);
            }
            if let Some(second_previous_low) = self.second_previous_low {
                next_sar = next_sar.min(second_previous_low);
            }
            if low < next_sar {
                self.is_long = false;
                self.sar = self.extreme_point;
                self.extreme_point = low;
                self.acceleration_factor = self.step;
            } else {
                self.sar = next_sar;
                if high > self.extreme_point {
                    self.extreme_point = high;
                    self.acceleration_factor =
                        (self.acceleration_factor + self.step).min(self.max_step);
                }
            }
        } else {
            if let Some(previous_high) = self.previous_high {
                next_sar = next_sar.max(previous_high);
            }
            if let Some(second_previous_high) = self.second_previous_high {
                next_sar = next_sar.max(second_previous_high);
            }
            if high > next_sar {
                self.is_long = true;
                self.sar = self.extreme_point;
                self.extreme_point = high;
                self.acceleration_factor = self.step;
            } else {
                self.sar = next_sar;
                if low < self.extreme_point {
                    self.extreme_point = low;
                    self.acceleration_factor =
                        (self.acceleration_factor + self.step).min(self.max_step);
                }
            }
        }
        self.second_previous_high = self.previous_high;
        self.second_previous_low = self.previous_low;
        self.previous_high = Some(high);
        self.previous_low = Some(low);
        self.current()
    }
    fn reset(&mut self) {
        self.initialized = false;
        self.is_long = true;
        self.sar = 0.0;
        self.extreme_point = 0.0;
        self.acceleration_factor = self.step;
        self.previous_high = None;
        self.previous_low = None;
        self.second_previous_high = None;
        self.second_previous_low = None;
    }
    fn current(&self) -> ParabolicSAR {
        ParabolicSAR {
            step: self.step,
            max_step: self.max_step,
            value: self.sar,
            is_long: self.is_long,
            acceleration_factor: self.acceleration_factor,
            extreme_point: self.extreme_point,
        }
    }
}

impl<C> TABuilder<ParabolicSAR, C> for ParabolicSARBuilder<C>
where
    C: Candle,
{
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> ParabolicSAR {
        self.build_from_storage(storage)
    }
    fn build(&mut self, data: &[C]) -> ParabolicSAR {
        self.build(data)
    }
    fn next(&mut self, data: &C) -> ParabolicSAR {
        self.next(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestCandle;
    fn candle(timestamp: i64, high: f64, low: f64) -> TestCandle {
        TestCandle {
            timestamp,
            open: low,
            high,
            low,
            close: high,
            volume: 1.0,
        }
    }
    #[test]
    fn rejects_invalid_steps() {
        assert!(ParabolicSARBuilder::<TestCandle>::new_checked(0.0, 0.2).is_err());
        assert!(ParabolicSARBuilder::<TestCandle>::new_checked(0.3, 0.2).is_err());
    }
    #[test]
    fn updates_state_machine_and_reverses() {
        let data = vec![
            candle(1, 10.0, 8.0),
            candle(2, 11.0, 9.0),
            candle(3, 7.0, 6.0),
        ];
        let mut builder = ParabolicSARBuilder::<TestCandle>::new(0.02, 0.2);
        let sar = builder.build(&data);
        assert!(!sar.is_long());
        assert_eq!(sar.value(), 11.0);
        assert_eq!(sar.extreme_point(), 6.0);
    }

    #[test]
    fn clamps_long_sar_to_prior_two_lows() {
        let data = vec![
            candle(1, 100.0, 50.0),
            candle(2, 200.0, 150.0),
            candle(3, 210.0, 90.0),
        ];
        let mut builder = ParabolicSARBuilder::<TestCandle>::new(0.2, 0.2);
        let sar = builder.build(&data);

        assert!(sar.is_long());
        assert_eq!(sar.value(), 50.0);
    }

    #[test]
    fn clamps_short_sar_to_prior_two_highs() {
        let data = vec![
            candle(1, 10.0, 8.0),
            candle(2, 11.0, 9.0),
            candle(3, 7.0, 6.0),
            candle(4, 10.95, 5.5),
        ];
        let mut builder = ParabolicSARBuilder::<TestCandle>::new(0.02, 0.2);
        let sar = builder.build(&data);

        assert!(!sar.is_long());
        assert_eq!(sar.value(), 11.0);
    }
}
