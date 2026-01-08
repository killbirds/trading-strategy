use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::analyzer::{MAAnalyzer, MACDAnalyzer, RSIAnalyzer};
use crate::candle_store::CandleStore;
use crate::indicator::ma::MAType;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::marker::PhantomData;
use trading_chart::Candle;

/// 기울기 분석 결과
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SlopeDirection {
    /// 상승 기울기
    Upward,
    /// 하락 기울기
    Downward,
    /// 횡보 (기울기가 거의 없음)
    Sideways,
}

/// 기울기 분석 데이터
#[derive(Debug, Clone)]
pub struct SlopeAnalysis {
    /// 기울기 값 (양수: 상승, 음수: 하락)
    pub slope: f64,
    /// 기울기 방향
    pub direction: SlopeDirection,
    /// 기울기 강도 (절대값)
    pub strength: f64,
    /// 선형 회귀의 결정계수 (R²) - 기울기의 신뢰도 (0.0 ~ 1.0)
    pub r_squared: f64,
    /// 시작 값
    pub start_value: f64,
    /// 종료 값
    pub end_value: f64,
    /// 분석 기간
    pub period: usize,
}

impl SlopeAnalysis {
    /// 새 기울기 분석 결과 생성
    pub fn new(
        slope: f64,
        r_squared: f64,
        start_value: f64,
        end_value: f64,
        period: usize,
        threshold: f64,
    ) -> Self {
        let strength = slope.abs();
        let direction = if strength < threshold {
            SlopeDirection::Sideways
        } else if slope > 0.0 {
            SlopeDirection::Upward
        } else {
            SlopeDirection::Downward
        };

        SlopeAnalysis {
            slope,
            direction,
            strength,
            r_squared,
            start_value,
            end_value,
            period,
        }
    }

    /// 기울기가 상승인지 확인
    pub fn is_upward(&self) -> bool {
        matches!(self.direction, SlopeDirection::Upward)
    }

    /// 기울기가 하락인지 확인
    pub fn is_downward(&self) -> bool {
        matches!(self.direction, SlopeDirection::Downward)
    }

    /// 기울기가 횡보인지 확인
    pub fn is_sideways(&self) -> bool {
        matches!(self.direction, SlopeDirection::Sideways)
    }
}

/// 직렬화 가능한 지표 타입 설정
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IndicatorTypeConfig {
    /// 종가
    ClosePrice,
    /// 고가
    HighPrice,
    /// 저가
    LowPrice,
    /// 이동평균 (MA)
    MovingAverage { ma_type: MAType, period: usize },
    /// RSI
    RSI { period: usize },
    /// MACD
    MACD {
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    },
    /// MACD 라인
    MACDLine {
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    },
    /// MACD 시그널 라인
    MACDSignalLine {
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    },
    /// MACD 히스토그램
    MACDHistogram {
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    },
}

impl IndicatorTypeConfig {
    /// IndicatorType으로 변환
    pub fn to_indicator_type<C: Candle>(&self) -> IndicatorType<C> {
        match self {
            IndicatorTypeConfig::ClosePrice => IndicatorType::ClosePrice(PhantomData),
            IndicatorTypeConfig::HighPrice => IndicatorType::HighPrice(PhantomData),
            IndicatorTypeConfig::LowPrice => IndicatorType::LowPrice(PhantomData),
            IndicatorTypeConfig::MovingAverage { ma_type, period } => {
                IndicatorType::MovingAverage {
                    ma_type: *ma_type,
                    period: *period,
                    _phantom: PhantomData,
                }
            }
            IndicatorTypeConfig::RSI { period } => IndicatorType::RSI {
                period: *period,
                _phantom: PhantomData,
            },
            IndicatorTypeConfig::MACD {
                fast_period,
                slow_period,
                signal_period,
            } => IndicatorType::MACD {
                fast_period: *fast_period,
                slow_period: *slow_period,
                signal_period: *signal_period,
                _phantom: PhantomData,
            },
            IndicatorTypeConfig::MACDLine {
                fast_period,
                slow_period,
                signal_period,
            } => IndicatorType::MACDLine {
                fast_period: *fast_period,
                slow_period: *slow_period,
                signal_period: *signal_period,
                _phantom: PhantomData,
            },
            IndicatorTypeConfig::MACDSignalLine {
                fast_period,
                slow_period,
                signal_period,
            } => IndicatorType::MACDSignalLine {
                fast_period: *fast_period,
                slow_period: *slow_period,
                signal_period: *signal_period,
                _phantom: PhantomData,
            },
            IndicatorTypeConfig::MACDHistogram {
                fast_period,
                slow_period,
                signal_period,
            } => IndicatorType::MACDHistogram {
                fast_period: *fast_period,
                slow_period: *slow_period,
                signal_period: *signal_period,
                _phantom: PhantomData,
            },
        }
    }
}

/// 분석할 지표 타입
#[derive(Debug)]
pub enum IndicatorType<C: Candle> {
    /// 종가
    ClosePrice(PhantomData<C>),
    /// 고가
    HighPrice(PhantomData<C>),
    /// 저가
    LowPrice(PhantomData<C>),
    /// 이동평균 (MA)
    MovingAverage {
        ma_type: MAType,
        period: usize,
        _phantom: PhantomData<C>,
    },
    /// RSI
    RSI {
        period: usize,
        _phantom: PhantomData<C>,
    },
    /// MACD
    MACD {
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
        _phantom: PhantomData<C>,
    },
    /// MACD 라인
    MACDLine {
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
        _phantom: PhantomData<C>,
    },
    /// MACD 시그널 라인
    MACDSignalLine {
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
        _phantom: PhantomData<C>,
    },
    /// MACD 히스토그램
    MACDHistogram {
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
        _phantom: PhantomData<C>,
    },
}

/// 기울기 분석기 데이터
#[derive(Debug)]
pub struct SlopeAnalyzerData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// 분석할 지표 값
    pub value: f64,
}

impl<C: Candle> SlopeAnalyzerData<C> {
    /// 새 분석 데이터 생성
    pub fn new(candle: C, value: f64) -> Self {
        SlopeAnalyzerData { candle, value }
    }
}

impl<C: Candle> GetCandle<C> for SlopeAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for SlopeAnalyzerData<C> {}

/// 저장된 analyzer 타입
#[derive(Debug)]
enum StoredAnalyzer<C: Candle> {
    None,
    MAAnalyzer(MAAnalyzer<C>),
    RSIAnalyzer(RSIAnalyzer<C>),
    MACDAnalyzer(MACDAnalyzer<C>),
}

/// 기울기 분석기
#[derive(Debug)]
pub struct SlopeAnalyzer<C: Candle> {
    /// 지표 타입
    indicator_type: IndicatorType<C>,
    /// 저장된 analyzer (재사용을 위해)
    analyzer: StoredAnalyzer<C>,
    /// 분석 데이터 히스토리 (최신 데이터가 인덱스 0)
    pub items: Vec<SlopeAnalyzerData<C>>,
}

impl<C: Candle> Display for SlopeAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(f, "캔들: {}, 값: {}", first.candle, first.value),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> SlopeAnalyzer<C> {
    /// IndicatorTypeConfig로부터 기울기 분석기 생성
    pub fn from_config(storage: &CandleStore<C>, config: &IndicatorTypeConfig) -> Self {
        Self::new(storage, config.to_indicator_type())
    }

    /// 종가 기반 기울기 분석기 생성
    pub fn for_close_price(storage: &CandleStore<C>) -> Self {
        Self::new(storage, IndicatorType::ClosePrice(PhantomData))
    }

    /// 고가 기반 기울기 분석기 생성
    pub fn for_high_price(storage: &CandleStore<C>) -> Self {
        Self::new(storage, IndicatorType::HighPrice(PhantomData))
    }

    /// 저가 기반 기울기 분석기 생성
    pub fn for_low_price(storage: &CandleStore<C>) -> Self {
        Self::new(storage, IndicatorType::LowPrice(PhantomData))
    }

    /// 이동평균 기반 기울기 분석기 생성
    pub fn for_ma(storage: &CandleStore<C>, ma_type: MAType, period: usize) -> Self {
        Self::new(
            storage,
            IndicatorType::MovingAverage {
                ma_type,
                period,
                _phantom: PhantomData,
            },
        )
    }

    /// RSI 기반 기울기 분석기 생성
    pub fn for_rsi(storage: &CandleStore<C>, period: usize) -> Self {
        Self::new(
            storage,
            IndicatorType::RSI {
                period,
                _phantom: PhantomData,
            },
        )
    }

    /// MACD 기반 기울기 분석기 생성
    pub fn for_macd(
        storage: &CandleStore<C>,
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    ) -> Self {
        Self::new(
            storage,
            IndicatorType::MACD {
                fast_period,
                slow_period,
                signal_period,
                _phantom: PhantomData,
            },
        )
    }

    /// MACD 라인 기반 기울기 분석기 생성
    pub fn for_macd_line(
        storage: &CandleStore<C>,
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    ) -> Self {
        Self::new(
            storage,
            IndicatorType::MACDLine {
                fast_period,
                slow_period,
                signal_period,
                _phantom: PhantomData,
            },
        )
    }

    /// MACD 시그널 라인 기반 기울기 분석기 생성
    pub fn for_macd_signal_line(
        storage: &CandleStore<C>,
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    ) -> Self {
        Self::new(
            storage,
            IndicatorType::MACDSignalLine {
                fast_period,
                slow_period,
                signal_period,
                _phantom: PhantomData,
            },
        )
    }

    /// MACD 히스토그램 기반 기울기 분석기 생성
    pub fn for_macd_histogram(
        storage: &CandleStore<C>,
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    ) -> Self {
        Self::new(
            storage,
            IndicatorType::MACDHistogram {
                fast_period,
                slow_period,
                signal_period,
                _phantom: PhantomData,
            },
        )
    }

    /// 새 기울기 분석기 생성 (내부 메서드)
    fn new(storage: &CandleStore<C>, indicator_type: IndicatorType<C>) -> Self {
        let mut analyzer = SlopeAnalyzer {
            indicator_type,
            analyzer: StoredAnalyzer::None,
            items: vec![],
        };
        analyzer.init_from_storage(storage);
        analyzer
    }

    /// 저장소에서 초기화
    fn init_from_storage(&mut self, storage: &CandleStore<C>) {
        match &self.indicator_type {
            IndicatorType::ClosePrice(_) => {
                for candle in storage.items() {
                    let value = candle.close_price();
                    self.items
                        .push(SlopeAnalyzerData::new(candle.clone(), value));
                }
            }
            IndicatorType::HighPrice(_) => {
                for candle in storage.items() {
                    let value = candle.high_price();
                    self.items
                        .push(SlopeAnalyzerData::new(candle.clone(), value));
                }
            }
            IndicatorType::LowPrice(_) => {
                for candle in storage.items() {
                    let value = candle.low_price();
                    self.items
                        .push(SlopeAnalyzerData::new(candle.clone(), value));
                }
            }
            IndicatorType::MovingAverage {
                ma_type, period, ..
            } => {
                let ma_analyzer = MAAnalyzer::new(ma_type, &[*period], storage);
                for data in ma_analyzer.items.iter() {
                    let value = data.mas.get_by_key_index(0).get();
                    self.items
                        .push(SlopeAnalyzerData::new(data.candle.clone(), value));
                }
                self.analyzer = StoredAnalyzer::MAAnalyzer(ma_analyzer);
            }
            IndicatorType::RSI { period, .. } => {
                let rsi_analyzer = RSIAnalyzer::new(*period, &MAType::SMA, &[], storage);
                for data in rsi_analyzer.items.iter() {
                    let value = data.rsi.value();
                    self.items
                        .push(SlopeAnalyzerData::new(data.candle.clone(), value));
                }
                self.analyzer = StoredAnalyzer::RSIAnalyzer(rsi_analyzer);
            }
            IndicatorType::MACD {
                fast_period,
                slow_period,
                signal_period,
                ..
            } => {
                let macd_analyzer =
                    MACDAnalyzer::new(*fast_period, *slow_period, *signal_period, storage);
                for data in macd_analyzer.items.iter() {
                    let value = data.macd.macd_line;
                    self.items
                        .push(SlopeAnalyzerData::new(data.candle.clone(), value));
                }
                self.analyzer = StoredAnalyzer::MACDAnalyzer(macd_analyzer);
            }
            IndicatorType::MACDLine {
                fast_period,
                slow_period,
                signal_period,
                ..
            } => {
                let macd_analyzer =
                    MACDAnalyzer::new(*fast_period, *slow_period, *signal_period, storage);
                for data in macd_analyzer.items.iter() {
                    let value = data.macd.macd_line;
                    self.items
                        .push(SlopeAnalyzerData::new(data.candle.clone(), value));
                }
                self.analyzer = StoredAnalyzer::MACDAnalyzer(macd_analyzer);
            }
            IndicatorType::MACDSignalLine {
                fast_period,
                slow_period,
                signal_period,
                ..
            } => {
                let macd_analyzer =
                    MACDAnalyzer::new(*fast_period, *slow_period, *signal_period, storage);
                for data in macd_analyzer.items.iter() {
                    let value = data.macd.signal_line;
                    self.items
                        .push(SlopeAnalyzerData::new(data.candle.clone(), value));
                }
                self.analyzer = StoredAnalyzer::MACDAnalyzer(macd_analyzer);
            }
            IndicatorType::MACDHistogram {
                fast_period,
                slow_period,
                signal_period,
                ..
            } => {
                let macd_analyzer =
                    MACDAnalyzer::new(*fast_period, *slow_period, *signal_period, storage);
                for data in macd_analyzer.items.iter() {
                    let value = data.macd.histogram;
                    self.items
                        .push(SlopeAnalyzerData::new(data.candle.clone(), value));
                }
                self.analyzer = StoredAnalyzer::MACDAnalyzer(macd_analyzer);
            }
        }
    }

    /// 선형 회귀를 사용한 기울기 계산
    ///
    /// # Arguments
    /// * `period` - 분석 기간 (캔들 수)
    /// * `offset` - 시작 오프셋 (0 = 최신 캔들)
    ///
    /// # Returns
    /// * `Option<SlopeAnalysis>` - 기울기 분석 결과
    pub fn calculate_slope(&self, period: usize, offset: usize) -> Option<SlopeAnalysis> {
        if self.items.len() < period + offset {
            return None;
        }

        // 시간 순서대로 정렬 (오래된 것부터 최신 순서)
        let values: Vec<f64> = self
            .items
            .iter()
            .skip(offset)
            .take(period)
            .map(|data| data.value)
            .rev() // 시간 순서대로 (오래된 것부터)
            .collect();

        if values.len() < period {
            return None;
        }

        let start_value = *values.first().unwrap(); // 가장 오래된 값
        let end_value = *values.last().unwrap(); // 가장 최신 값

        let n = values.len() as f64;
        let sum_x: f64 = (0..values.len()).map(|i| i as f64).sum();
        let sum_y: f64 = values.iter().sum();
        let sum_xy: f64 = values.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
        let sum_x2: f64 = (0..values.len()).map(|i| (i as f64).powi(2)).sum();

        let mean_x = sum_x / n;
        let mean_y = sum_y / n;

        let slope = if sum_x2 - n * mean_x * mean_x != 0.0 {
            (sum_xy - n * mean_x * mean_y) / (sum_x2 - n * mean_x * mean_x)
        } else {
            0.0
        };

        let ss_res: f64 = values
            .iter()
            .enumerate()
            .map(|(i, &y)| {
                let predicted = mean_y + slope * (i as f64 - mean_x);
                (y - predicted).powi(2)
            })
            .sum();

        let ss_tot: f64 = values.iter().map(|&y| (y - mean_y).powi(2)).sum();

        let r_squared = if ss_tot != 0.0 {
            1.0 - (ss_res / ss_tot)
        } else {
            0.0
        };

        let threshold = (end_value.abs() * 0.01).max(0.0001);
        Some(SlopeAnalysis::new(
            slope,
            r_squared,
            start_value,
            end_value,
            period,
            threshold,
        ))
    }

    /// 단순 차이 기반 기울기 계산
    ///
    /// # Arguments
    /// * `period` - 분석 기간 (캔들 수)
    /// * `offset` - 시작 오프셋 (0 = 최신 캔들)
    ///
    /// # Returns
    /// * `Option<SlopeAnalysis>` - 기울기 분석 결과
    pub fn calculate_simple_slope(&self, period: usize, offset: usize) -> Option<SlopeAnalysis> {
        if self.items.len() < period + offset {
            return None;
        }

        // items는 최신이 앞에 있으므로, 오래된 값이 뒤에 있음
        let start_value = self.items.get(offset + period - 1)?.value; // 가장 오래된 값
        let end_value = self.items.get(offset)?.value; // 가장 최신 값

        let slope = (end_value - start_value) / period as f64;
        let strength = slope.abs();
        let threshold = (end_value.abs() * 0.01).max(0.0001);

        let direction = if strength < threshold {
            SlopeDirection::Sideways
        } else if slope > 0.0 {
            SlopeDirection::Upward
        } else {
            SlopeDirection::Downward
        };

        Some(SlopeAnalysis {
            slope,
            direction,
            strength,
            r_squared: 0.0,
            start_value,
            end_value,
            period,
        })
    }

    /// 기울기가 상승인지 확인
    ///
    /// # Arguments
    /// * `period` - 분석 기간
    /// * `offset` - 시작 오프셋
    /// * `use_linear_regression` - 선형 회귀 사용 여부
    ///
    /// # Returns
    /// * `bool` - 기울기가 상승이면 true
    pub fn is_slope_upward(
        &self,
        period: usize,
        offset: usize,
        use_linear_regression: bool,
    ) -> bool {
        let analysis = if use_linear_regression {
            self.calculate_slope(period, offset)
        } else {
            self.calculate_simple_slope(period, offset)
        };

        analysis.map(|a| a.is_upward()).unwrap_or(false)
    }

    /// 기울기가 하락인지 확인
    ///
    /// # Arguments
    /// * `period` - 분석 기간
    /// * `offset` - 시작 오프셋
    /// * `use_linear_regression` - 선형 회귀 사용 여부
    ///
    /// # Returns
    /// * `bool` - 기울기가 하락이면 true
    pub fn is_slope_downward(
        &self,
        period: usize,
        offset: usize,
        use_linear_regression: bool,
    ) -> bool {
        let analysis = if use_linear_regression {
            self.calculate_slope(period, offset)
        } else {
            self.calculate_simple_slope(period, offset)
        };

        analysis.map(|a| a.is_downward()).unwrap_or(false)
    }

    /// 기울기가 횡보인지 확인
    ///
    /// # Arguments
    /// * `period` - 분석 기간
    /// * `offset` - 시작 오프셋
    /// * `use_linear_regression` - 선형 회귀 사용 여부
    ///
    /// # Returns
    /// * `bool` - 기울기가 횡보이면 true
    pub fn is_slope_sideways(
        &self,
        period: usize,
        offset: usize,
        use_linear_regression: bool,
    ) -> bool {
        let analysis = if use_linear_regression {
            self.calculate_slope(period, offset)
        } else {
            self.calculate_simple_slope(period, offset)
        };

        analysis.map(|a| a.is_sideways()).unwrap_or(false)
    }

    /// 기울기 강도가 임계값을 초과하는지 확인
    ///
    /// # Arguments
    /// * `period` - 분석 기간
    /// * `offset` - 시작 오프셋
    /// * `threshold` - 기울기 강도 임계값
    /// * `use_linear_regression` - 선형 회귀 사용 여부
    ///
    /// # Returns
    /// * `bool` - 기울기 강도가 임계값을 초과하면 true
    pub fn is_slope_strength_above(
        &self,
        period: usize,
        offset: usize,
        threshold: f64,
        use_linear_regression: bool,
    ) -> bool {
        let analysis = if use_linear_regression {
            self.calculate_slope(period, offset)
        } else {
            self.calculate_simple_slope(period, offset)
        };

        analysis.map(|a| a.strength >= threshold).unwrap_or(false)
    }

    /// 두 기간의 기울기를 비교
    ///
    /// # Arguments
    /// * `short_period` - 단기 기간
    /// * `long_period` - 장기 기간
    /// * `offset` - 시작 오프셋
    /// * `use_linear_regression` - 선형 회귀 사용 여부
    ///
    /// # Returns
    /// * `Option<(SlopeAnalysis, SlopeAnalysis)>` - (단기, 장기) 기울기 분석 결과
    pub fn compare_slopes(
        &self,
        short_period: usize,
        long_period: usize,
        offset: usize,
        use_linear_regression: bool,
    ) -> Option<(SlopeAnalysis, SlopeAnalysis)> {
        let short = if use_linear_regression {
            self.calculate_slope(short_period, offset)?
        } else {
            self.calculate_simple_slope(short_period, offset)?
        };

        let long = if use_linear_regression {
            self.calculate_slope(long_period, offset)?
        } else {
            self.calculate_simple_slope(long_period, offset)?
        };

        Some((short, long))
    }

    /// 기울기 가속도 확인 (단기 기울기가 장기 기울기보다 강한지)
    ///
    /// # Arguments
    /// * `short_period` - 단기 기간
    /// * `long_period` - 장기 기간
    /// * `offset` - 시작 오프셋
    /// * `use_linear_regression` - 선형 회귀 사용 여부
    ///
    /// # Returns
    /// * `bool` - 단기 기울기가 장기 기울기보다 강하면 true
    pub fn is_slope_accelerating(
        &self,
        short_period: usize,
        long_period: usize,
        offset: usize,
        use_linear_regression: bool,
    ) -> bool {
        if let Some((short, long)) =
            self.compare_slopes(short_period, long_period, offset, use_linear_regression)
        {
            short.strength > long.strength && short.slope.signum() == long.slope.signum()
        } else {
            false
        }
    }

    /// 기울기 감속 확인 (단기 기울기가 장기 기울기보다 약한지)
    ///
    /// # Arguments
    /// * `short_period` - 단기 기간
    /// * `long_period` - 장기 기간
    /// * `offset` - 시작 오프셋
    /// * `use_linear_regression` - 선형 회귀 사용 여부
    ///
    /// # Returns
    /// * `bool` - 단기 기울기가 장기 기울기보다 약하면 true
    pub fn is_slope_decelerating(
        &self,
        short_period: usize,
        long_period: usize,
        offset: usize,
        use_linear_regression: bool,
    ) -> bool {
        if let Some((short, long)) =
            self.compare_slopes(short_period, long_period, offset, use_linear_regression)
        {
            short.strength < long.strength && short.slope.signum() == long.slope.signum()
        } else {
            false
        }
    }

    /// 현재 지표 값 반환
    pub fn get_value(&self) -> f64 {
        self.items.first().map(|data| data.value).unwrap_or(0.0)
    }
}

impl<C: Candle + 'static + Clone> AnalyzerOps<SlopeAnalyzerData<C>, C> for SlopeAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> SlopeAnalyzerData<C> {
        let value = match &mut self.analyzer {
            StoredAnalyzer::None => match &self.indicator_type {
                IndicatorType::ClosePrice(_) => candle.close_price(),
                IndicatorType::HighPrice(_) => candle.high_price(),
                IndicatorType::LowPrice(_) => candle.low_price(),
                _ => 0.0,
            },
            StoredAnalyzer::MAAnalyzer(ma_analyzer) => {
                ma_analyzer.next(candle.clone());
                ma_analyzer.get_ma(0)
            }
            StoredAnalyzer::RSIAnalyzer(rsi_analyzer) => {
                rsi_analyzer.next(candle.clone());
                rsi_analyzer.get_rsi()
            }
            StoredAnalyzer::MACDAnalyzer(macd_analyzer) => {
                macd_analyzer.next(candle.clone());
                match &self.indicator_type {
                    IndicatorType::MACD { .. } | IndicatorType::MACDLine { .. } => macd_analyzer
                        .items
                        .first()
                        .map(|d| d.macd.macd_line)
                        .unwrap_or(0.0),
                    IndicatorType::MACDSignalLine { .. } => macd_analyzer
                        .items
                        .first()
                        .map(|d| d.macd.signal_line)
                        .unwrap_or(0.0),
                    IndicatorType::MACDHistogram { .. } => macd_analyzer
                        .items
                        .first()
                        .map(|d| d.macd.histogram)
                        .unwrap_or(0.0),
                    _ => 0.0,
                }
            }
        };
        SlopeAnalyzerData::new(candle, value)
    }

    fn datum(&self) -> &Vec<SlopeAnalyzerData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<SlopeAnalyzerData<C>> {
        &mut self.items
    }
}
