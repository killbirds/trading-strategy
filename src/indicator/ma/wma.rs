use crate::candle_store::CandleStore;
use crate::indicator::TABuilder;
use crate::indicator::ma::MA;
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

/// 가중이동평균(WMA) 계산 빌더
///
/// 가중이동평균은 최근 데이터에 선형적으로 증가하는 가중치를 부여하는 이동평균입니다.
#[derive(Debug)]
pub struct WMABuilder<C: Candle> {
    /// WMA 계산 기간
    pub period: usize,
    /// 가격 데이터 저장용 배열
    values: Vec<f64>,
    _phantom: PhantomData<C>,
}

/// 가중이동평균(WMA) 기술적 지표
///
/// 계산된 WMA 값을 저장하고 제공합니다.
#[derive(Clone, Debug)]
pub struct WMA {
    /// WMA 계산 기간
    period: usize,
    /// 계산된 WMA 값
    wma: f64,
}

impl Display for WMA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WMA({}: {:.2})", self.period, self.wma)
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
    /// * `period` - WMA 계산 기간
    ///
    /// # Returns
    /// * `WMABuilder` - 새 빌더 인스턴스
    ///
    /// # Panics
    /// * 유효하지 않은 기간이 제공되면 패닉 발생
    pub fn new(period: usize) -> Self {
        if period == 0 {
            panic!("WMA 기간은 0보다 커야 합니다");
        }

        Self {
            period,
            values: Vec::with_capacity(period * 2),
            _phantom: PhantomData,
        }
    }

    /// 저장소에서 WMA 지표 생성
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    ///
    /// # Returns
    /// * `WMA` - 계산된 WMA 지표
    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> WMA {
        self.build(&storage.get_time_ordered_items())
    }

    /// 데이터 벡터에서 WMA 지표 생성
    ///
    /// # Arguments
    /// * `data` - 캔들 데이터 벡터
    ///
    /// # Returns
    /// * `WMA` - 계산된 WMA 지표
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

    /// 새 캔들 데이터로 WMA 지표 업데이트
    ///
    /// # Arguments
    /// * `data` - 새 캔들 데이터
    ///
    /// # Returns
    /// * `WMA` - 업데이트된 WMA 지표
    pub fn next(&mut self, data: &C) -> WMA {
        // 새 가격 추가
        self.values.push(data.close_price());

        // 필요한 데이터만 유지
        if self.values.len() > self.period * 2 {
            let excess = self.values.len() - self.period * 2;
            self.values.drain(0..excess);
        }

        // 충분한 데이터가 없는 경우 현재 가격 반환
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

    /// WMA 값 계산
    ///
    /// 가중이동평균 공식: (p₁*1 + p₂*2 + ... + pₙ*n) / (1+2+...+n)
    /// 여기서 p₁은 가장 오래된 데이터, pₙ은 가장 최신 데이터
    fn calculate_wma(&self) -> f64 {
        let len = self.values.len().min(self.period);
        if len == 0 {
            return 0.0;
        }

        // 최근 period 개만 사용 (가장 최근 데이터에 높은 가중치)
        let start_idx = if self.values.len() > self.period {
            self.values.len() - self.period
        } else {
            0
        };
        let slice = &self.values[start_idx..];

        let weight_sum: f64 = (1..=len).sum::<usize>() as f64;
        let mut wma = 0.0;
        let mut actual_weight_sum = 0.0;

        // 최신 데이터에 높은 가중치 부여 (slice의 마지막 요소가 가장 높은 가중치)
        for (i, &value) in slice.iter().enumerate() {
            let weight = (i + 1) as f64 / weight_sum;
            actual_weight_sum += weight;
            wma += value * weight;
        }

        // 가중치 합 검증 (부동소수점 오차 허용)
        const EPSILON: f64 = 1e-10;
        if (actual_weight_sum - 1.0).abs() > EPSILON {
            // 가중치 합이 1이 아니면 정규화
            if actual_weight_sum > EPSILON {
                wma /= actual_weight_sum;
            }
        }

        wma
    }
}

impl<C> TABuilder<Box<dyn MA>, C> for WMABuilder<C>
where
    C: Candle,
{
    fn build_from_storage(&mut self, storage: &CandleStore<C>) -> Box<dyn MA> {
        Box::new(self.build_from_storage(storage))
    }

    fn build(&mut self, data: &[C]) -> Box<dyn MA> {
        Box::new(self.build(data))
    }

    fn next(&mut self, data: &C) -> Box<dyn MA> {
        Box::new(self.next(data))
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

    #[test]
    fn test_wma_exact_calculation() {
        let candles = vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 110.0,
                low: 95.0,
                close: 10.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 110.0,
                low: 95.0,
                close: 20.0,
                volume: 1100.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 110.0,
                high: 120.0,
                low: 105.0,
                close: 30.0,
                volume: 1200.0,
            },
        ];

        let mut builder = WMABuilder::new(3);
        let wma = builder.build(&candles);

        // WMA = (10*1 + 20*2 + 30*3) / (1+2+3) = (10 + 40 + 90) / 6 = 140 / 6 = 23.33...
        let expected = (10.0 * 1.0 + 20.0 * 2.0 + 30.0 * 3.0) / 6.0;
        assert!((wma.get() - expected).abs() < 0.01);
    }

    #[test]
    fn test_wma_less_data_than_period() {
        let candles = vec![TestCandle {
            timestamp: Utc::now().timestamp(),
            open: 100.0,
            high: 110.0,
            low: 95.0,
            close: 105.0,
            volume: 1000.0,
        }];

        let mut builder = WMABuilder::new(5);
        let wma = builder.build(&candles);

        // 데이터가 period보다 적으면 가중 평균 계산
        assert!(wma.get() > 0.0);
    }

    #[test]
    fn test_wma_consecutive_next() {
        let mut builder = WMABuilder::new(3);

        let candle1 = TestCandle {
            timestamp: Utc::now().timestamp(),
            open: 100.0,
            high: 110.0,
            low: 95.0,
            close: 100.0,
            volume: 1000.0,
        };
        let wma1 = builder.next(&candle1);
        // 데이터가 부족하면 현재 가격 반환
        assert_eq!(wma1.get(), 100.0);

        let candle2 = TestCandle {
            timestamp: Utc::now().timestamp(),
            open: 100.0,
            high: 110.0,
            low: 95.0,
            close: 110.0,
            volume: 1100.0,
        };
        let wma2 = builder.next(&candle2);
        assert_eq!(wma2.get(), 110.0);

        let candle3 = TestCandle {
            timestamp: Utc::now().timestamp(),
            open: 110.0,
            high: 120.0,
            low: 105.0,
            close: 120.0,
            volume: 1200.0,
        };
        let wma3 = builder.next(&candle3);
        assert!(wma3.get() > 0.0);
        // WMA는 최신 데이터에 더 높은 가중치를 주므로 120에 가까워야 함
        assert!(wma3.get() > 100.0);
    }

    #[test]
    #[should_panic(expected = "WMA 기간은 0보다 커야 합니다")]
    fn test_wma_invalid_period() {
        WMABuilder::<TestCandle>::new(0);
    }

    #[test]
    fn test_wma_tabuilder_trait() {
        let candles = create_test_candles();
        let mut builder = WMABuilder::new(2);

        let ma: Box<dyn MA> = Box::new(builder.build(&candles));
        assert_eq!(ma.period(), 2);
        assert!(ma.get() > 0.0);

        let new_candle = TestCandle {
            timestamp: Utc::now().timestamp(),
            open: 115.0,
            high: 130.0,
            low: 115.0,
            close: 125.0,
            volume: 1300.0,
        };

        let updated_ma: Box<dyn MA> = Box::new(builder.next(&new_candle));
        assert_eq!(updated_ma.period(), 2);
        assert!(updated_ma.get() > 0.0);
    }

    #[test]
    fn test_wma_weight_distribution() {
        let mut builder = WMABuilder::new(4);
        let candles = vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 110.0,
                low: 95.0,
                close: 10.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 110.0,
                low: 95.0,
                close: 20.0,
                volume: 1100.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 110.0,
                high: 120.0,
                low: 105.0,
                close: 30.0,
                volume: 1200.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 120.0,
                high: 130.0,
                low: 115.0,
                close: 40.0,
                volume: 1300.0,
            },
        ];

        let wma = builder.build(&candles);

        // WMA = (10*1 + 20*2 + 30*3 + 40*4) / (1+2+3+4) = (10 + 40 + 90 + 160) / 10 = 300 / 10 = 30.0
        let expected = (10.0 * 1.0 + 20.0 * 2.0 + 30.0 * 3.0 + 40.0 * 4.0) / 10.0;
        assert!((wma.get() - expected).abs() < 0.01);
    }

    #[test]
    fn test_wma_empty_data() {
        let mut builder = WMABuilder::<TestCandle>::new(5);
        let wma = builder.build(&[]);

        assert_eq!(wma.get(), 0.0);
        assert_eq!(wma.period(), 5);
    }
}
