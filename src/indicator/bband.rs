use crate::candle_store::CandleStore;
use crate::indicator::TABuilder;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use trading_chart::Candle;

/// 볼린저 밴드 출력값 구조체
#[derive(Clone, Debug)]
struct BollingerBandsOutput {
    average: f64,
    upper: f64,
    lower: f64,
}

/// 볼린저 밴드 계산기
#[derive(Debug)]
struct BollingerBandsIndicator {
    period: usize,
    multiplier: f64,
    values: Vec<f64>,
}

impl BollingerBandsIndicator {
    fn new(period: usize, multiplier: f64) -> Self {
        Self {
            period,
            multiplier,
            values: Vec::with_capacity(period * 2),
        }
    }

    fn next(&mut self, input: &impl Candle) -> BollingerBandsOutput {
        let price = input.close_price();
        self.values.push(price);

        // 충분한 데이터가 쌓일 때까지 기본값 반환
        if self.values.len() < self.period {
            return BollingerBandsOutput {
                average: price,
                upper: price,
                lower: price,
            };
        }

        // 필요한 데이터만 유지
        if self.values.len() > self.period * 2 {
            let excess = self.values.len() - self.period * 2;
            self.values.drain(0..excess);
        }

        // 직접 계산 (Moving Average와 Standard Deviation)
        // 평균 계산
        let sum: f64 = self.values.iter().sum();
        let mean = sum / self.values.len() as f64;

        // 표준편차 계산
        let variance = self
            .values
            .iter()
            .map(|value| {
                let diff = mean - value;
                diff * diff
            })
            .sum::<f64>()
            / self.values.len() as f64;

        let std_dev = variance.sqrt();

        // 볼린저 밴드 계산
        let upper = mean + (std_dev * self.multiplier);
        let lower = mean - (std_dev * self.multiplier);

        BollingerBandsOutput {
            average: mean,
            upper,
            lower,
        }
    }
}

/// 볼린저 밴드 계산 빌더
///
/// 볼린저 밴드는 가격의 변동성을 측정하는 기술적 지표로,
/// 이동평균선과 그 주변의 표준편차 기반 밴드로 구성됩니다.
#[derive(Debug)]
pub struct BBandBuilder<C: Candle> {
    /// 내부 볼린저 밴드 계산 객체
    indicator: BollingerBandsIndicator,
    /// 계산 기간
    period: usize,
    /// 표준편차 승수
    multiplier: f64,
    _phantom: PhantomData<C>,
}

/// 볼린저 밴드 기술적 지표
///
/// 상단, 중간, 하단 밴드로 구성된 볼린저 밴드 값
#[derive(Clone, Debug)]
pub struct BBand {
    /// 내부 볼린저 밴드 계산 결과
    bband: BollingerBandsOutput,
    /// 계산 기간
    period: usize,
    /// 표준편차 승수
    multiplier: f64,
}

impl BBand {
    /// 중간 밴드(이동평균) 값 반환
    ///
    /// # Returns
    /// * `f64` - 중간 밴드 값
    pub fn average(&self) -> f64 {
        self.bband.average
    }

    /// 상단 밴드 값 반환
    ///
    /// # Returns
    /// * `f64` - 상단 밴드 값
    pub fn upper(&self) -> f64 {
        self.bband.upper
    }

    /// 하단 밴드 값 반환
    ///
    /// # Returns
    /// * `f64` - 하단 밴드 값
    pub fn lower(&self) -> f64 {
        self.bband.lower
    }

    /// 현재 밴드폭 계산
    ///
    /// # Returns
    /// * `f64` - 밴드폭 (상단 - 하단) / 중간
    pub fn bandwidth(&self) -> f64 {
        (self.upper() - self.lower()) / self.average()
    }

    /// 가격의 상대적 위치 계산 (%B)
    ///
    /// # Arguments
    /// * `price` - 위치를 계산할 가격
    ///
    /// # Returns
    /// * `f64` - 상대적 위치 (0: 하단 밴드, 0.5: 중간 밴드, 1: 상단 밴드)
    pub fn percent_b(&self, price: f64) -> f64 {
        let range = self.upper() - self.lower();
        if range.abs() < f64::EPSILON {
            return 0.5; // 범위가 없는 경우 중간 위치 반환
        }

        (price - self.lower()) / range
    }
}

impl Display for BBand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BBAND({}, {:.2}, {:.2}, {:.2})",
            self.period,
            self.lower(),
            self.average(),
            self.upper()
        )
    }
}

impl<C> BBandBuilder<C>
where
    C: Candle,
{
    /// 새 볼린저 밴드 빌더 생성
    ///
    /// # Arguments
    /// * `period` - 계산 기간 (일반적으로 20)
    /// * `multiplier` - 표준편차 승수 (일반적으로 2.0)
    ///
    /// # Returns
    /// * `BBandBuilder` - 새 빌더 인스턴스
    ///
    /// # Panics
    /// * 유효하지 않은 매개변수가 제공되면 패닉 발생
    pub fn new(period: usize, multiplier: f64) -> Self {
        if period == 0 {
            panic!("볼린저 밴드 기간은 0보다 커야 합니다");
        }

        if multiplier <= 0.0 {
            panic!("표준편차 승수는 0보다 커야 합니다");
        }

        let indicator = BollingerBandsIndicator::new(period, multiplier);

        BBandBuilder {
            indicator,
            period,
            multiplier,
            _phantom: PhantomData,
        }
    }

    /// 저장소에서 볼린저 밴드 지표 생성
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    ///
    /// # Returns
    /// * `BBand` - 계산된 볼린저 밴드 지표
    pub fn from_storage(&mut self, storage: &CandleStore<C>) -> BBand {
        self.build(&storage.get_reversed_items())
    }

    /// 데이터 벡터에서 볼린저 밴드 지표 생성
    ///
    /// # Arguments
    /// * `data` - 캔들 데이터 벡터
    ///
    /// # Returns
    /// * `BBand` - 계산된 볼린저 밴드 지표
    pub fn build(&mut self, data: &[C]) -> BBand {
        if data.is_empty() {
            // 빈 데이터의 경우 기본값 반환 (모든 밴드가 0인 볼린저 밴드)
            return BBand {
                bband: BollingerBandsOutput {
                    average: 0.0,
                    upper: 0.0,
                    lower: 0.0,
                },
                period: self.period,
                multiplier: self.multiplier,
            };
        }

        let bband = data.iter().fold(
            BollingerBandsOutput {
                average: 0.0,
                upper: 0.0,
                lower: 0.0,
            },
            |_, item| self.indicator.next(item),
        );

        BBand {
            bband,
            period: self.period,
            multiplier: self.multiplier,
        }
    }

    /// 새 캔들 데이터로 볼린저 밴드 지표 업데이트
    ///
    /// # Arguments
    /// * `data` - 새 캔들 데이터
    ///
    /// # Returns
    /// * `BBand` - 업데이트된 볼린저 밴드 지표
    pub fn next(&mut self, data: &C) -> BBand {
        let bband = self.indicator.next(data);
        BBand {
            bband,
            period: self.period,
            multiplier: self.multiplier,
        }
    }
}

impl<C> TABuilder<BBand, C> for BBandBuilder<C>
where
    C: Candle,
{
    fn from_storage(&mut self, storage: &CandleStore<C>) -> BBand {
        self.from_storage(storage)
    }

    fn build(&mut self, data: &[C]) -> BBand {
        self.build(data)
    }

    fn next(&mut self, data: &C) -> BBand {
        self.next(data)
    }
}

/// 가격의 밴드 상대 위치 계산
///
/// # Arguments
/// * `trade_price` - 현재 거래 가격
/// * `band` - 참조 밴드 값
///
/// # Returns
/// * `f64` - 밴드 대비 상대적 위치 비율
fn get_ratio(trade_price: f64, band: f64) -> f64 {
    if band.abs() < f64::EPSILON {
        return 0.0; // 0으로 나누는 것 방지
    }
    (trade_price - band) / band
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percent_b() {
        let bband = BBand {
            bband: BollingerBandsOutput {
                lower: 90.0,
                average: 100.0,
                upper: 110.0,
            },
            period: 20,
            multiplier: 2.0,
        };

        assert_eq!(bband.percent_b(90.0), 0.0); // 하단 밴드
        assert_eq!(bband.percent_b(100.0), 0.5); // 중간 밴드
        assert_eq!(bband.percent_b(110.0), 1.0); // 상단 밴드
        assert_eq!(bband.percent_b(105.0), 0.75); // 중간과 상단 사이
    }

    #[test]
    fn test_bandwidth() {
        let bband = BBand {
            bband: BollingerBandsOutput {
                lower: 90.0,
                average: 100.0,
                upper: 110.0,
            },
            period: 20,
            multiplier: 2.0,
        };

        assert_eq!(bband.bandwidth(), 0.2); // (110 - 90) / 100 = 0.2
    }
}
