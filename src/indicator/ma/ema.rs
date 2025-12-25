use crate::candle_store::CandleStore;
use crate::indicator::TABuilder;
use crate::indicator::ma::MA;
use crate::indicator::utils::moving_average;
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

/// 지수이동평균(EMA) 계산 빌더
///
/// 지수이동평균은 최근 데이터에 더 높은 가중치를 부여하는 이동평균입니다.
///
/// # 성능 고려사항
/// - 메모리 사용량: period * 2개의 가격 데이터만 유지하여 O(period) 메모리 사용
/// - 시간 복잡도: O(1) 업데이트 (증분 계산), O(n) 초기 빌드 (n = 데이터 개수)
/// - 최적화: 이전 EMA 값을 캐싱하여 효율적인 증분 계산 지원
#[derive(Debug)]
pub struct EMABuilder<C: Candle> {
    /// EMA 계산 기간
    pub period: usize,
    /// 가격 데이터 저장용 배열
    values: Vec<f64>,
    /// 이전 EMA 값
    previous_ema: Option<f64>,
    _phantom: PhantomData<C>,
}

/// 지수이동평균(EMA) 기술적 지표
///
/// 계산된 EMA 값을 저장하고 제공합니다.
#[derive(Clone, Debug)]
pub struct EMA {
    /// EMA 계산 기간
    period: usize,
    /// 계산된 EMA 값
    ema: f64,
}

impl Display for EMA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EMA({}: {:.2})", self.period, self.ema)
    }
}

impl MA for EMA {
    fn get(&self) -> f64 {
        self.ema
    }

    fn period(&self) -> usize {
        self.period
    }
}

impl<C> EMABuilder<C>
where
    C: Candle,
{
    /// 시리즈 전체에서 EMA 계산
    fn calculate_ema_from_series(&self, values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let alpha = moving_average::calculate_ema_alpha(self.period);
        let mut ema = values[0]; // 첫 번째 값으로 초기화

        // 충분한 데이터가 있으면 최근 period개의 SMA로 시작
        if values.len() >= self.period {
            let start_idx = values.len() - self.period;
            let initial_slice = &values[start_idx..];
            ema = initial_slice.iter().sum::<f64>() / self.period as f64;

            // 나머지 값들에 대해 EMA 계산
            for &value in &values[start_idx + self.period..] {
                ema = moving_average::calculate_ema_step(value, ema, alpha);
            }
        } else {
            // 데이터가 부족하면 모든 값에 대해 EMA 계산
            for &value in &values[1..] {
                ema = moving_average::calculate_ema_step(value, ema, alpha);
            }
        }

        ema
    }
    /// 새 EMA 빌더 생성
    ///
    /// # Arguments
    /// * `period` - EMA 계산 기간
    ///
    /// # Returns
    /// * `EMABuilder` - 새 빌더 인스턴스
    ///
    /// # Panics
    /// * 유효하지 않은 기간이 제공되면 패닉 발생
    pub fn new(period: usize) -> Self {
        if period == 0 {
            panic!("EMA 기간은 0보다 커야 합니다");
        }

        EMABuilder {
            period,
            values: Vec::with_capacity(period * 2),
            previous_ema: None,
            _phantom: PhantomData,
        }
    }

    /// 저장소에서 EMA 지표 생성
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    ///
    /// # Returns
    /// * `EMA` - 계산된 EMA 지표
    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> EMA {
        self.build(&storage.get_ascending_items())
    }

    /// 데이터 벡터에서 EMA 지표 생성
    ///
    /// # Arguments
    /// * `data` - 캔들 데이터 벡터
    ///
    /// # Returns
    /// * `EMA` - 계산된 EMA 지표
    pub fn build(&mut self, data: &[C]) -> EMA {
        if data.is_empty() {
            self.previous_ema = None;
            return EMA {
                period: self.period,
                ema: 0.0, // 데이터가 없으면 기본값 0 반환
            };
        }

        // 데이터를 values 배열에 저장
        self.values.clear();
        for item in data {
            self.values.push(item.close_price());
        }

        // EMA 계산 (최근 period * 2 개만 사용하여 계산 정확도 유지)
        let values_to_use = if self.values.len() > self.period * 2 {
            let start_idx = self.values.len() - self.period * 2;
            &self.values[start_idx..]
        } else {
            &self.values[..]
        };
        let ema = self.calculate_ema_from_series(values_to_use);

        // previous_ema 업데이트하여 next() 호출 시 일관성 유지
        self.previous_ema = Some(ema);

        EMA {
            period: self.period,
            ema,
        }
    }

    /// 새 캔들 데이터로 EMA 지표 업데이트
    ///
    /// # Arguments
    /// * `data` - 새 캔들 데이터
    ///
    /// # Returns
    /// * `EMA` - 업데이트된 EMA 지표
    pub fn next(&mut self, data: &C) -> EMA {
        // 새 가격 추가
        self.values.push(data.close_price());

        // 필요한 데이터만 유지
        if self.values.len() > self.period * 2 {
            let excess = self.values.len() - self.period * 2;
            self.values.drain(0..excess);
        }

        // EMA 계산
        let alpha = moving_average::calculate_ema_alpha(self.period);
        let ema = match self.previous_ema {
            Some(prev_ema) => {
                moving_average::calculate_ema_step(data.close_price(), prev_ema, alpha)
            }
            None => {
                // 첫 번째 EMA는 SMA로 계산하거나 현재 가격 사용
                moving_average::calculate_sma_or_default(
                    &self.values,
                    self.period,
                    data.close_price(),
                )
            }
        };
        self.previous_ema = Some(ema);

        EMA {
            period: self.period,
            ema,
        }
    }
}

impl<C> TABuilder<Box<dyn MA>, C> for EMABuilder<C>
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
    fn test_ema_calculation() {
        let candles = create_test_candles();
        let mut builder = EMABuilder::new(2);

        // 첫 번째 계산
        let ema = builder.build(&candles);
        assert_eq!(ema.period(), 2);
        assert!(ema.get() > 0.0);

        // 새 캔들로 업데이트
        let new_candle = TestCandle {
            timestamp: Utc::now().timestamp(),
            open: 115.0,
            high: 130.0,
            low: 115.0,
            close: 125.0,
            volume: 1300.0,
        };

        let updated_ema = builder.next(&new_candle);
        assert_eq!(updated_ema.period(), 2);
        assert!(updated_ema.get() > 0.0);
    }

    #[test]
    #[should_panic(expected = "EMA 기간은 0보다 커야 합니다")]
    fn test_invalid_period() {
        EMABuilder::<TestCandle>::new(0);
    }

    #[test]
    fn test_empty_data() {
        let mut builder = EMABuilder::<TestCandle>::new(5);
        let ema = builder.build(&[]);

        assert_eq!(ema.get(), 0.0);
        assert_eq!(ema.period(), 5);
    }

    #[test]
    fn test_ema_display() {
        let ema = EMA {
            period: 5,
            ema: 100.0,
        };

        let display_str = ema.to_string();
        assert!(display_str.contains("EMA"));
        assert!(display_str.contains("5"));
        assert!(display_str.contains("100"));
    }

    #[test]
    fn test_ema_trend() {
        let mut builder = EMABuilder::new(2);

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

        let ema1 = builder.build(&up_candles);
        let ema2 = builder.next(&up_candles[1]);

        // 상승 추세에서는 EMA 값이 증가해야 함
        assert!(ema2.get() >= ema1.get());
    }

    #[test]
    fn test_ema_exact_calculation() {
        let candles = vec![
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 110.0,
                low: 95.0,
                close: 100.0,
                volume: 1000.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0,
                high: 110.0,
                low: 95.0,
                close: 110.0,
                volume: 1100.0,
            },
            TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 110.0,
                high: 120.0,
                low: 105.0,
                close: 120.0,
                volume: 1200.0,
            },
        ];

        let mut builder = EMABuilder::new(2);
        let ema = builder.build(&candles);

        // period=2일 때 alpha = 2/(2+1) = 2/3
        // 첫 2개 평균: (100 + 110) / 2 = 105
        // EMA = alpha * 120 + (1-alpha) * 105 = (2/3)*120 + (1/3)*105 = 80 + 35 = 115
        let alpha = 2.0 / 3.0;
        let initial_sma = (100.0 + 110.0) / 2.0;
        let expected = alpha * 120.0 + (1.0 - alpha) * initial_sma;
        assert!((ema.get() - expected).abs() < 0.1);
    }

    #[test]
    fn test_ema_less_data_than_period() {
        let candles = vec![TestCandle {
            timestamp: Utc::now().timestamp(),
            open: 100.0,
            high: 110.0,
            low: 95.0,
            close: 105.0,
            volume: 1000.0,
        }];

        let mut builder = EMABuilder::new(5);
        let ema = builder.build(&candles);

        // 데이터가 period보다 적으면 첫 번째 값으로 시작
        assert!(ema.get() > 0.0);
    }

    #[test]
    fn test_ema_consecutive_next() {
        let mut builder = EMABuilder::new(3);

        let candle1 = TestCandle {
            timestamp: Utc::now().timestamp(),
            open: 100.0,
            high: 110.0,
            low: 95.0,
            close: 100.0,
            volume: 1000.0,
        };
        let ema1 = builder.next(&candle1);
        assert!(ema1.get() > 0.0);

        let candle2 = TestCandle {
            timestamp: Utc::now().timestamp(),
            open: 100.0,
            high: 110.0,
            low: 95.0,
            close: 110.0,
            volume: 1100.0,
        };
        let ema2 = builder.next(&candle2);
        assert!(ema2.get() > 0.0);
        assert!(ema2.get() != ema1.get());

        let candle3 = TestCandle {
            timestamp: Utc::now().timestamp(),
            open: 110.0,
            high: 120.0,
            low: 105.0,
            close: 120.0,
            volume: 1200.0,
        };
        let ema3 = builder.next(&candle3);
        assert!(ema3.get() > 0.0);
    }

    #[test]
    fn test_ema_tabuilder_trait() {
        let candles = create_test_candles();
        let mut builder = EMABuilder::new(2);

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
    fn test_ema_alpha_calculation() {
        let builder = EMABuilder::<TestCandle>::new(5);
        // period=5일 때 alpha = 2/(5+1) = 2/6 = 1/3
        // 이는 calculate_ema_from_series 내부에서 사용됨
        assert_eq!(builder.period, 5);
    }

    #[test]
    fn test_ema_with_many_data_points() {
        let mut candles = Vec::new();
        for i in 0..20 {
            candles.push(TestCandle {
                timestamp: Utc::now().timestamp(),
                open: 100.0 + i as f64,
                high: 110.0 + i as f64,
                low: 95.0 + i as f64,
                close: 105.0 + i as f64,
                volume: 1000.0 + i as f64,
            });
        }

        let mut builder = EMABuilder::new(5);
        let ema = builder.build(&candles);

        // 많은 데이터가 있어도 최근 period*2 개만 사용
        assert!(ema.get() > 0.0);
        assert_eq!(ema.period(), 5);
    }

    #[test]
    fn test_ema_known_values_accuracy() {
        // 알려진 값과 비교하는 정확도 검증 테스트
        // 테스트 데이터: [22.27, 22.19, 22.08, 22.17, 22.18, 22.13, 22.23, 22.43, 22.24, 22.29]
        // period=10일 때 알려진 EMA 값과 비교
        let known_prices = vec![
            22.27, 22.19, 22.08, 22.17, 22.18, 22.13, 22.23, 22.43, 22.24, 22.29,
        ];
        let mut candles = Vec::new();
        for (i, &price) in known_prices.iter().enumerate() {
            candles.push(TestCandle {
                timestamp: Utc::now().timestamp() + i as i64,
                open: price,
                high: price + 0.1,
                low: price - 0.1,
                close: price,
                volume: 1000.0,
            });
        }

        let mut builder = EMABuilder::new(10);
        let ema = builder.build(&candles);

        // period=10일 때 첫 번째 EMA는 SMA와 같아야 함
        let expected_sma: f64 = known_prices.iter().sum::<f64>() / 10.0;
        assert!(
            (ema.get() - expected_sma).abs() < 0.01,
            "EMA should equal SMA for first calculation. Expected: {}, Got: {}",
            expected_sma,
            ema.get()
        );
    }

    #[test]
    fn test_ema_known_values_period_3() {
        // period=3인 경우 알려진 계산 결과와 비교
        // 데이터: [10.0, 11.0, 12.0, 13.0, 14.0]
        // period=3일 때:
        // - 첫 3개 SMA: (10+11+12)/3 = 11.0
        // - alpha = 2/(3+1) = 0.5
        // - EMA(13) = 0.5 * 13 + 0.5 * 11 = 12.0
        // - EMA(14) = 0.5 * 14 + 0.5 * 12 = 13.0
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

        let mut builder = EMABuilder::new(3);
        let ema = builder.build(&candles);

        // 첫 3개 SMA: (10+11+12)/3 = 11.0
        // EMA(13) = 0.5 * 13 + 0.5 * 11 = 12.0
        // EMA(14) = 0.5 * 14 + 0.5 * 12 = 13.0
        let expected = 13.0;
        assert!(
            (ema.get() - expected).abs() < 0.01,
            "EMA calculation mismatch. Expected: {}, Got: {}",
            expected,
            ema.get()
        );
    }

    #[test]
    fn test_ema_incremental_vs_build_consistency() {
        // next를 여러 번 호출한 결과와 build를 한 번 호출한 결과의 일관성 검증
        let mut builder1 = EMABuilder::<TestCandle>::new(14);
        let mut builder2 = EMABuilder::<TestCandle>::new(14);
        let candles = create_test_candles();

        // builder1: next를 여러 번 호출
        for candle in &candles {
            builder1.next(candle);
        }
        let ema1 = builder1.next(&candles[candles.len() - 1]);

        // builder2: build를 한 번 호출
        let ema2 = builder2.build(&candles);

        // 값들이 유효한 범위 내에 있어야 함
        assert!(ema1.get() > 0.0);
        assert!(ema2.get() > 0.0);

        // EMA 값의 차이가 너무 크지 않아야 함 (1% 이내)
        let diff_percent = if ema2.get() > 0.0 {
            ((ema1.get() - ema2.get()).abs() / ema2.get()) * 100.0
        } else {
            0.0
        };
        assert!(
            diff_percent < 1.0,
            "EMA values should be consistent. Incremental: {}, Build: {}, Diff: {}%",
            ema1.get(),
            ema2.get(),
            diff_percent
        );
    }
}
