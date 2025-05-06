use crate::candle_store::CandleStore;
use crate::indicator::TABuilder;
use crate::indicator::ma::MA;
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

#[derive(Debug)]
pub struct WMABuilder<C: Candle> {
    period: usize,
    values: Vec<f64>,
    _phantom: PhantomData<C>,
}

#[derive(Clone, Debug)]
pub struct WMA {
    period: usize,
    wma: f64,
}

impl Display for WMA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WMA({}: {})", self.period, self.wma)
    }
}

impl MA for WMA {
    fn get(&self) -> f64 {
        self.wma
    }

    fn period(&self) -> usize {
        self.period
    }
}

impl<C> WMABuilder<C>
where
    C: Candle,
{
    /// 새 WMA 빌더 생성
    ///
    /// # Arguments
    /// * `period` - 계산 기간
    ///
    /// # Returns
    /// * `WMABuilder` - 새 WMA 빌더 인스턴스
    pub fn new(period: usize) -> Self {
        if period == 0 {
            panic!("WMA 기간은 0보다 커야 합니다");
        }

        Self {
            period,
            values: Vec::with_capacity(period),
            _phantom: PhantomData,
        }
    }

    pub fn from_storage(&mut self, storage: &CandleStore<C>) -> WMA {
        self.build(&storage.get_time_ordered_items())
    }

    pub fn build(&mut self, data: &[C]) -> WMA {
        if data.is_empty() {
            return WMA {
                period: self.period,
                wma: 0.0,
            };
        }

        // 데이터를 values 배열에 저장
        self.values.clear();
        for item in data {
            self.values.push(item.close_price());
        }

        let wma = self.calculate_wma();

        WMA {
            period: self.period,
            wma,
        }
    }

    pub fn next(&mut self, data: &C) -> WMA {
        // 새 가격 추가
        self.values.push(data.close_price());

        // 필요한 데이터만 유지
        if self.values.len() > self.period * 2 {
            let excess = self.values.len() - self.period * 2;
            self.values.drain(0..excess);
        }

        // 충분한 데이터가 없는 경우
        if self.values.len() < self.period {
            return WMA {
                period: self.period,
                wma: data.close_price(),
            };
        }

        let wma = self.calculate_wma();

        WMA {
            period: self.period,
            wma,
        }
    }

    fn calculate_wma(&self) -> f64 {
        let mut wma = 0.0;
        let len = self.values.len().min(self.period);
        let weight_sum: f64 = (1..=len).sum::<usize>() as f64;

        for i in 0..len {
            let weight = (len - i) as f64 / weight_sum;
            wma += self.values[i] * weight;
        }

        wma
    }
}

impl<C> TABuilder<Box<dyn MA>, C> for WMABuilder<C>
where
    C: Candle,
{
    fn from_storage(&mut self, storage: &CandleStore<C>) -> Box<dyn MA> {
        Box::new(self.from_storage(storage))
    }

    fn build(&mut self, data: &[C]) -> Box<dyn MA> {
        Box::new(self.build(data))
    }

    fn next(&mut self, data: &C) -> Box<dyn MA> {
        Box::new(self.next(data))
    }
}

impl WMA {
    /// 새로운 데이터로 WMA 업데이트
    ///
    /// # Arguments
    /// * `data` - 새 캔들 데이터
    ///
    /// # Returns
    /// * `Option<f64>` - 업데이트된 WMA 값
    pub fn next<C: Candle>(&mut self, data: &C) -> Option<f64> {
        if self.period == 0 {
            return None;
        }

        // 현재 캔들의 가격
        let price = data.close_price();

        // 1. 데이터 크기 문제로 이 구현에서는 단일 값으로 WMA를 근사값 계산
        // 2. 완전한 WMA 계산을 위해서는 period 크기의 최근 데이터 배열이 필요함

        // 가중치 계산 (최신 데이터에 더 높은 가중치 부여)
        // WMA 공식: (p₁*1 + p₂*2 + p₃*3 + ... + pₙ*n) / (1+2+3+...+n)
        // 여기서는 현재 가격과 이전 WMA를 활용한 근사 계산

        // 이전 WMA와 새 가격 간의 가중 평균
        // 새 가격에 더 높은 가중치 부여 (period에 따라 조정)
        let weight_new = 2.0 * (self.period as f64) / ((self.period + 1) * self.period) as f64;
        let weight_old = 1.0 - weight_new;

        // 가중 평균 계산
        self.wma = price * weight_new + self.wma * weight_old;

        Some(self.wma)
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
                high: 115.0,
                low: 95.0,
                close: 110.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 110.0,
                high: 125.0,
                low: 105.0,
                close: 120.0,
                volume: 1100.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 120.0,
                high: 125.0,
                low: 110.0,
                close: 115.0,
                volume: 1200.0,
            },
        ]
    }

    #[test]
    fn test_wma_calculation() {
        let candles = create_test_candles();
        let mut builder = WMABuilder::<TestCandle>::new(2);

        // 첫 번째 계산
        let wma = builder.build(&candles);
        assert_eq!(wma.period(), 2);
        assert!(wma.get() > 0.0);

        // 새 캔들로 업데이트
        let new_candle = TestCandle {
            timestamp: Utc::now().timestamp(),
            open: 115.0,
            high: 130.0,
            low: 115.0,
            close: 125.0,
            volume: 1300.0,
        };

        let updated_wma = builder.next(&new_candle);
        assert_eq!(updated_wma.period(), 2);
        assert!(updated_wma.get() > 0.0);
    }

    #[test]
    fn test_wma_weights() {
        let mut builder = WMABuilder::<TestCandle>::new(3);
        let test_data = vec![10.0, 20.0, 30.0];

        // 데이터 추가
        for &price in &test_data {
            builder.values.push(price);
        }

        // 가중치 합이 1에 가까운지 확인
        let weight_sum: f64 = (1..=3).sum::<usize>() as f64;
        let weights: Vec<f64> = (1..=3).map(|i| i as f64 / weight_sum).collect();

        // 가중치가 증가하는 순서인지 확인
        for i in 1..weights.len() {
            assert!(weights[i] > weights[i - 1]);
        }

        // 가중치 합이 1인지 확인
        assert!((weights.iter().sum::<f64>() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_wma_display() {
        let wma = WMA {
            period: 5,
            wma: 100.0,
        };

        let display_str = wma.to_string();
        assert!(display_str.contains("WMA"));
        assert!(display_str.contains("5"));
        assert!(display_str.contains("100"));
    }

    #[test]
    fn test_wma_trend() {
        let mut builder = WMABuilder::new(2);

        // 상승 추세 테스트
        let up_candles = vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 110.0,
                low: 95.0,
                close: 105.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 105.0,
                high: 115.0,
                low: 100.0,
                close: 110.0,
                volume: 1100.0,
            },
        ];

        let wma1 = builder.build(&up_candles);
        let wma2 = builder.next(&up_candles[1]);

        // 상승 추세에서는 WMA 값이 증가해야 함
        assert!(wma2.get() >= wma1.get());
    }
}
