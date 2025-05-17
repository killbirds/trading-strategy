use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use trading_chart::Candle;

use super::atr::ATRBuilder;

// f64를 해시맵 키로 사용하기 위한 래퍼 타입
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct F64Key(pub f64);

impl Eq for F64Key {}

impl Hash for F64Key {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // f64를 비트 패턴으로 변환하여 해시
        let bits = self.0.to_bits();
        bits.hash(state);
    }
}

/// 슈퍼트렌드 정보
#[derive(Debug, Clone, Copy)]
pub struct SuperTrend {
    /// 슈퍼트렌드 값
    pub value: f64,
    /// 추세 방향 (1: 상승, -1: 하락)
    pub direction: i8,
    /// 상단 밴드
    pub upper_band: f64,
    /// 하단 밴드
    pub lower_band: f64,
}

impl SuperTrend {
    /// 새 슈퍼트렌드 값 생성
    pub fn new(value: f64, direction: i8, upper_band: f64, lower_band: f64) -> SuperTrend {
        SuperTrend {
            value,
            direction,
            upper_band,
            lower_band,
        }
    }

    /// 기본 슈퍼트렌드 값 생성
    pub fn default() -> SuperTrend {
        SuperTrend {
            value: 0.0,
            direction: 0,
            upper_band: 0.0,
            lower_band: 0.0,
        }
    }

    /// 상승 추세인지 확인
    pub fn is_uptrend(&self) -> bool {
        self.direction > 0
    }

    /// 하락 추세인지 확인
    pub fn is_downtrend(&self) -> bool {
        self.direction < 0
    }
}

/// 슈퍼트렌드 모음
#[derive(Debug, Clone)]
pub struct SuperTrends {
    /// 기간 및 승수별 슈퍼트렌드 값
    values: HashMap<(usize, F64Key), SuperTrend>,
}

impl Default for SuperTrends {
    fn default() -> Self {
        Self::new()
    }
}

impl SuperTrends {
    /// 새 슈퍼트렌드 모음 생성
    pub fn new() -> SuperTrends {
        SuperTrends {
            values: HashMap::new(),
        }
    }

    /// 슈퍼트렌드 값 추가
    pub fn add(&mut self, period: usize, multiplier: f64, value: SuperTrend) {
        self.values.insert((period, F64Key(multiplier)), value);
    }

    /// 특정 기간 및 승수의 슈퍼트렌드 값 반환
    pub fn get(&self, period: &usize, multiplier: &f64) -> SuperTrend {
        match self.values.get(&(*period, F64Key(*multiplier))) {
            Some(value) => *value,
            None => SuperTrend::default(),
        }
    }

    /// 모든 슈퍼트렌드 값 반환
    pub fn get_all(&self) -> Vec<SuperTrend> {
        let mut result = Vec::new();
        for value in self.values.values() {
            result.push(*value);
        }
        result
    }
}

/// 슈퍼트렌드 계산을 위한 빌더
#[derive(Debug)]
pub struct SuperTrendBuilder<C: Candle> {
    /// 계산 기간
    period: usize,
    /// ATR 승수
    multiplier: f64,
    /// ATR 빌더
    atr_builder: ATRBuilder<C>,
    /// 이전 슈퍼트렌드 값
    previous_supertrend: Option<SuperTrend>,
    /// 이동평균 가격 (중간값)
    avg_prices: Vec<f64>,
    /// 종가 데이터
    close_prices: Vec<f64>,
    /// 캔들 타입 표시자
    _phantom: PhantomData<C>,
}

impl<C: Candle> SuperTrendBuilder<C> {
    /// 새 슈퍼트렌드 빌더 생성
    pub fn new(period: usize, multiplier: f64) -> SuperTrendBuilder<C> {
        SuperTrendBuilder {
            period,
            multiplier,
            atr_builder: ATRBuilder::new(period),
            previous_supertrend: None,
            avg_prices: Vec::new(),
            close_prices: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// 다음 캔들 데이터로 슈퍼트렌드 계산
    pub fn next(&mut self, candle: &C) -> SuperTrend {
        // ATR 계산
        let atr = self.atr_builder.next(candle);

        // 중간 가격 계산 및 저장
        let avg_price = (candle.high_price() + candle.low_price()) / 2.0;
        self.avg_prices.push(avg_price);
        self.close_prices.push(candle.close_price());

        // 필요한 데이터만 유지
        if self.avg_prices.len() > self.period * 2 {
            self.avg_prices.remove(0);
            self.close_prices.remove(0);
        }

        // 충분한 데이터가 없는 경우
        if self.avg_prices.len() < 2 {
            return SuperTrend::default();
        }

        // 밴드 계산
        let basic_upper_band = avg_price + (self.multiplier * atr);
        let basic_lower_band = avg_price - (self.multiplier * atr);

        let close_price = candle.close_price();

        // 이전 슈퍼트렌드 값 가져오기
        let (final_upper_band, final_lower_band, super_trend, direction) =
            match self.previous_supertrend {
                Some(prev) => {
                    // 상단 밴드 계산
                    let upper_band = if basic_upper_band < prev.upper_band
                        || self.close_prices[self.close_prices.len() - 2] > prev.upper_band
                    {
                        basic_upper_band
                    } else {
                        prev.upper_band
                    };

                    // 하단 밴드 계산
                    let lower_band = if basic_lower_band > prev.lower_band
                        || self.close_prices[self.close_prices.len() - 2] < prev.lower_band
                    {
                        basic_lower_band
                    } else {
                        prev.lower_band
                    };

                    // 슈퍼트렌드 값 및 방향 결정
                    let (super_trend, direction) =
                        if prev.value == prev.upper_band && close_price <= upper_band {
                            (upper_band, -1)
                        } else if prev.value == prev.lower_band && close_price >= lower_band {
                            (lower_band, 1)
                        } else if close_price <= upper_band && prev.direction > 0 {
                            (upper_band, -1)
                        } else if close_price >= lower_band && prev.direction < 0 {
                            (lower_band, 1)
                        } else if prev.direction > 0 {
                            (lower_band, 1)
                        } else {
                            (upper_band, -1)
                        };

                    (upper_band, lower_band, super_trend, direction)
                }
                None => {
                    // 초기 방향 결정
                    let direction = if close_price > basic_upper_band {
                        1
                    } else {
                        -1
                    };

                    // 초기 슈퍼트렌드 값
                    let super_trend = if direction > 0 {
                        basic_lower_band
                    } else {
                        basic_upper_band
                    };

                    (basic_upper_band, basic_lower_band, super_trend, direction)
                }
            };

        // 계산된 슈퍼트렌드 저장
        let result = SuperTrend::new(super_trend, direction, final_upper_band, final_lower_band);
        self.previous_supertrend = Some(result);
        result
    }
}

/// 슈퍼트렌드 빌더 집합
#[derive(Debug)]
pub struct SuperTrendsBuilder<C: Candle> {
    /// 기간 및 승수별 슈퍼트렌드 빌더
    builders: HashMap<(usize, F64Key), SuperTrendBuilder<C>>,
}

impl<C: Candle> SuperTrendsBuilder<C> {
    /// 새 슈퍼트렌드 빌더 집합 생성
    pub fn new(periods: &[(usize, f64)]) -> SuperTrendsBuilder<C> {
        let mut builders = HashMap::new();
        for &(period, multiplier) in periods {
            builders.insert(
                (period, F64Key(multiplier)),
                SuperTrendBuilder::new(period, multiplier),
            );
        }
        SuperTrendsBuilder { builders }
    }

    /// 다음 캔들 데이터로 모든 슈퍼트렌드 계산
    pub fn next(&mut self, candle: &C) -> SuperTrends {
        let mut supertrends = SuperTrends::new();
        for (&(period, F64Key(multiplier)), builder) in &mut self.builders {
            let st = builder.next(candle);
            supertrends.add(period, multiplier, st);
        }
        supertrends
    }
}

/// 슈퍼트렌드 빌더 팩토리
pub struct SuperTrendsBuilderFactory;

impl SuperTrendsBuilderFactory {
    /// 새 슈퍼트렌드 빌더 생성
    pub fn new<C: Candle>(periods: &[(usize, f64)]) -> SuperTrendsBuilder<C> {
        SuperTrendsBuilder::new(periods)
    }
}
