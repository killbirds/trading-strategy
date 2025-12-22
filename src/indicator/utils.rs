use crate::candle_store::CandleStore;
use crate::indicator::adx;
use crate::indicator::adx::ADXsBuilderFactory;
use crate::indicator::bband::{BollingerBands, BollingerBandsBuilder};
use crate::indicator::ichimoku;
use crate::indicator::ichimoku::IchimokusBuilderFactory;
use crate::indicator::ma;
use crate::indicator::ma::MA;
use crate::indicator::ma::ema::EMABuilder;
use crate::indicator::ma::sma::SMABuilder;
use crate::indicator::macd;
use crate::indicator::macd::{MACDParams, MACDsBuilderFactory};
use crate::indicator::max;
use crate::indicator::max::MAXsBuilderFactory;
use crate::indicator::min;
use crate::indicator::min::MINsBuilderFactory;
use crate::indicator::rsi;
use crate::indicator::rsi::{RSIBuilder, RSIsBuilderFactory};
use crate::indicator::volume;
use crate::indicator::volume::VolumesBuilderFactory;
use crate::indicator::{TAs, TAsBuilder};
use trading_chart::Candle;

/// 공통 이동평균 계산 함수들
pub mod moving_average {
    /// 단순이동평균(SMA) 계산 - 공통 유틸리티 함수
    ///
    /// # Arguments
    /// * `values` - 가격 데이터 배열
    /// * `period` - 계산 기간
    ///
    /// # Returns
    /// * `f64` - 계산된 SMA 값 (데이터가 부족하거나 period가 0이면 0.0 반환)
    pub fn calculate_sma(values: &[f64], period: usize) -> f64 {
        if values.is_empty() || period == 0 {
            return 0.0;
        }

        if values.len() >= period {
            let start_idx = values.len() - period;
            let slice = &values[start_idx..];
            slice.iter().sum::<f64>() / period as f64
        } else {
            values.iter().sum::<f64>() / values.len() as f64
        }
    }

    /// 단순이동평균(SMA) 계산 - 충분한 데이터가 없을 때 기본값 사용
    ///
    /// # Arguments
    /// * `values` - 가격 데이터 배열
    /// * `period` - 계산 기간
    /// * `default_value` - 데이터가 충분하지 않을 때 사용할 기본값
    ///
    /// # Returns
    /// * `f64` - 계산된 SMA 값 또는 기본값
    pub fn calculate_sma_or_default(values: &[f64], period: usize, default_value: f64) -> f64 {
        if values.is_empty() || period == 0 {
            return default_value;
        }

        if values.len() >= period {
            let start_idx = values.len() - period;
            let slice = &values[start_idx..];
            slice.iter().sum::<f64>() / period as f64
        } else {
            default_value
        }
    }

    /// 지수이동평균(EMA) 계산을 위한 알파값 계산
    ///
    /// # Arguments
    /// * `period` - EMA 기간
    ///
    /// # Returns
    /// * `f64` - 알파값 (평활화 계수)
    pub fn calculate_ema_alpha(period: usize) -> f64 {
        2.0 / (period + 1) as f64
    }

    /// 지수이동평균(EMA) 한 스텝 계산
    ///
    /// # Arguments
    /// * `current_price` - 현재 가격
    /// * `previous_ema` - 이전 EMA 값
    /// * `alpha` - 평활화 계수
    ///
    /// # Returns
    /// * `f64` - 계산된 EMA 값
    pub fn calculate_ema_step(current_price: f64, previous_ema: f64, alpha: f64) -> f64 {
        alpha * current_price + (1.0 - alpha) * previous_ema
    }
}

/// 일반적으로 사용되는 모든 기술적 지표를 포함하는 분석 결과
#[derive(Debug)]
pub struct TechnicalAnalysisResult {
    /// 단순이동평균 (SMA)
    pub smas: TAs<usize, Box<dyn ma::MA>>,
    /// 지수이동평균 (EMA)
    pub emas: TAs<usize, Box<dyn ma::MA>>,
    /// 상대강도지수 (RSI)
    pub rsis: TAs<usize, rsi::RSI>,
    /// 평균방향지수 (ADX)
    pub adxs: TAs<usize, adx::ADX>,
    /// 볼린저밴드 (BBand)
    pub bbands: TAs<usize, BollingerBands>,
    /// 이동평균수렴발산 (MACD)
    pub macds: TAs<MACDParams, macd::MACD>,
    /// 최대값 (MAX)
    pub maxs: TAs<usize, max::MAX>,
    /// 최소값 (MIN)
    pub mins: TAs<usize, min::MIN>,
    /// 일목균형표 (Ichimoku)
    pub ichimokus: TAs<ichimoku::IchimokuParams, ichimoku::Ichimoku>,
    /// 볼륨 분석
    pub volumes: TAs<usize, volume::Volume>,
}

/// 단순한 빌더 팩토리들 - 이 파일에서만 사용됨
mod factories {
    use super::*;

    /// 단순이동평균 빌더 생성
    pub fn build_sma_builders<C: Candle + 'static>(
        periods: &[usize],
    ) -> TAsBuilder<usize, Box<dyn ma::MA>, C> {
        TAsBuilder::new("smas".to_owned(), periods, |period| {
            Box::new(SMABuilder::<C>::new(*period))
        })
    }

    /// 지수이동평균 빌더 생성
    pub fn build_ema_builders<C: Candle + 'static>(
        periods: &[usize],
    ) -> TAsBuilder<usize, Box<dyn ma::MA>, C> {
        TAsBuilder::new("emas".to_owned(), periods, |period| {
            Box::new(EMABuilder::<C>::new(*period))
        })
    }

    /// 볼린저밴드 빌더 생성
    pub fn build_bband_builders<C: Candle + 'static>(
        periods: &[usize],
    ) -> TAsBuilder<usize, BollingerBands, C> {
        TAsBuilder::new("bbands".to_owned(), periods, |period| {
            Box::new(BollingerBandsBuilder::<C>::new(*period, 2.0))
        })
    }
}

/// 기술적 분석 유틸리티 빌더
///
/// 여러 기술적 지표를 동시에 계산하고 관리하는 빌더 클래스
#[derive(Debug)]
pub struct TechnicalAnalysisBuilder<C: Candle> {
    /// SMA 빌더
    sma_builder: TAsBuilder<usize, Box<dyn ma::MA>, C>,
    /// EMA 빌더
    ema_builder: TAsBuilder<usize, Box<dyn ma::MA>, C>,
    /// RSI 빌더
    rsi_builder: TAsBuilder<usize, rsi::RSI, C>,
    /// ADX 빌더
    adx_builder: TAsBuilder<usize, adx::ADX, C>,
    /// 볼린저밴드 빌더
    bband_builder: TAsBuilder<usize, BollingerBands, C>,
    /// MACD 빌더
    macd_builder: TAsBuilder<MACDParams, macd::MACD, C>,
    /// MAX 빌더
    max_builder: TAsBuilder<usize, max::MAX, C>,
    /// MIN 빌더
    min_builder: TAsBuilder<usize, min::MIN, C>,
    /// 일목균형표 빌더
    ichimoku_builder: TAsBuilder<ichimoku::IchimokuParams, ichimoku::Ichimoku, C>,
    /// 볼륨 빌더
    volume_builder: TAsBuilder<usize, volume::Volume, C>,
}

impl<C: Candle + 'static> TechnicalAnalysisBuilder<C> {
    /// 새 기술적 분석 빌더 생성
    ///
    /// 기본적인 매개변수 세트를 사용하여 빌더 초기화
    ///
    /// # Returns
    /// * `TechnicalAnalysisBuilder` - 초기화된 빌더
    pub fn new() -> Self {
        // 기본 이동평균 기간
        let ma_periods = vec![5, 10, 20, 50, 100, 200];

        // 기본 RSI 기간
        let rsi_periods = vec![9, 14, 25];

        // 기본 ADX 기간
        let adx_periods = vec![14];

        // 기본 볼린저밴드 기간과 배수
        let bband_periods = vec![20];

        // 기본 MACD 매개변수
        let macd_params = vec![MACDParams {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
        }];

        // 기본 최대/최소값 기간
        let max_min_periods = vec![10, 20, 50];

        // 기본 일목균형표 매개변수
        let ichimoku_params = vec![ichimoku::IchimokuParams {
            tenkan_period: 9,
            kijun_period: 26,
            senkou_period: 52,
        }];

        // 기본 볼륨 기간
        let volume_periods = vec![10, 20, 50];

        Self {
            sma_builder: factories::build_sma_builders(&ma_periods),
            ema_builder: factories::build_ema_builders(&ma_periods),
            rsi_builder: RSIsBuilderFactory::build(&rsi_periods),
            adx_builder: ADXsBuilderFactory::build(&adx_periods),
            bband_builder: factories::build_bband_builders(&bband_periods),
            macd_builder: MACDsBuilderFactory::build(&macd_params),
            max_builder: MAXsBuilderFactory::build(&max_min_periods),
            min_builder: MINsBuilderFactory::build(&max_min_periods),
            ichimoku_builder: IchimokusBuilderFactory::build(&ichimoku_params),
            volume_builder: VolumesBuilderFactory::build(&volume_periods),
        }
    }

    /// 저장소에서 모든 기술적 지표 계산
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    ///
    /// # Returns
    /// * `TechnicalAnalysisResult` - 계산된 모든 지표
    pub fn build_from_storage(&mut self, storage: &CandleStore<C>) -> TechnicalAnalysisResult {
        TechnicalAnalysisResult {
            smas: self.sma_builder.build_from_storage(storage),
            emas: self.ema_builder.build_from_storage(storage),
            rsis: self.rsi_builder.build_from_storage(storage),
            adxs: self.adx_builder.build_from_storage(storage),
            bbands: self.bband_builder.build_from_storage(storage),
            macds: self.macd_builder.build_from_storage(storage),
            maxs: self.max_builder.build_from_storage(storage),
            mins: self.min_builder.build_from_storage(storage),
            ichimokus: self.ichimoku_builder.build_from_storage(storage),
            volumes: self.volume_builder.build_from_storage(storage),
        }
    }

    /// 데이터에서 모든 기술적 지표 계산
    ///
    /// # Arguments
    /// * `data` - 캔들 데이터 벡터
    ///
    /// # Returns
    /// * `TechnicalAnalysisResult` - 계산된 모든 지표
    pub fn build(&mut self, data: &[C]) -> TechnicalAnalysisResult {
        TechnicalAnalysisResult {
            smas: self.sma_builder.build(data),
            emas: self.ema_builder.build(data),
            rsis: self.rsi_builder.build(data),
            adxs: self.adx_builder.build(data),
            bbands: self.bband_builder.build(data),
            macds: self.macd_builder.build(data),
            maxs: self.max_builder.build(data),
            mins: self.min_builder.build(data),
            ichimokus: self.ichimoku_builder.build(data),
            volumes: self.volume_builder.build(data),
        }
    }

    /// 새 캔들로 모든 기술적 지표 업데이트
    ///
    /// # Arguments
    /// * `data` - 새 캔들 데이터
    ///
    /// # Returns
    /// * `TechnicalAnalysisResult` - 업데이트된 모든 지표
    pub fn next(&mut self, data: &C) -> TechnicalAnalysisResult {
        TechnicalAnalysisResult {
            smas: self.sma_builder.next(data),
            emas: self.ema_builder.next(data),
            rsis: self.rsi_builder.next(data),
            adxs: self.adx_builder.next(data),
            bbands: self.bband_builder.next(data),
            macds: self.macd_builder.next(data),
            maxs: self.max_builder.next(data),
            mins: self.min_builder.next(data),
            ichimokus: self.ichimoku_builder.next(data),
            volumes: self.volume_builder.next(data),
        }
    }
}

impl<C: Candle + 'static> Default for TechnicalAnalysisBuilder<C> {
    fn default() -> Self {
        Self::new()
    }
}

/// 특정 기간에 대한 빠른 기술적 분석 수행
///
/// # Arguments
/// * `data` - 분석할 캔들 데이터
/// * `period` - SMA, EMA, RSI 계산 기간
///
/// # Returns
/// * `(f64, f64, f64)` - (SMA, EMA, RSI) 값 튜플
pub fn quick_analysis<C: Candle>(data: &[C], period: usize) -> (f64, f64, f64) {
    if data.is_empty() {
        return (0.0, 0.0, 50.0);
    }

    let mut sma_builder = SMABuilder::<C>::new(period);
    let mut ema_builder = EMABuilder::<C>::new(period);
    let mut rsi_builder = RSIBuilder::<C>::new(period);

    let sma = sma_builder.build(data);
    let ema = ema_builder.build(data);
    let rsi = rsi_builder.build(data);

    (sma.get(), ema.get(), rsi.value)
}

/// 캔들 데이터에서 급격한 가격 변동 감지
///
/// # Arguments
/// * `data` - 검사할 캔들 데이터
/// * `threshold_percent` - 급격한 변동으로 간주할 백분율 변화 (기본값 3.0%)
///
/// # Returns
/// * `bool` - 급격한 변동 감지 여부
pub fn detect_price_spike<C: Candle>(data: &[C], threshold_percent: Option<f64>) -> bool {
    if data.len() < 2 {
        return false;
    }

    let threshold = threshold_percent.unwrap_or(3.0);
    let last = match data.last() {
        Some(candle) => candle,
        None => return false,
    };
    let prev = &data[data.len() - 2];

    let prev_close = prev.close_price();
    if prev_close == 0.0 {
        return false;
    }

    let percent_change = ((last.close_price() - prev_close) / prev_close) * 100.0;
    percent_change.abs() >= threshold
}

/// RSI 기반 과매수/과매도 분석
///
/// # Arguments
/// * `data` - 분석할 캔들 데이터
///
/// # Returns
/// * `i8` - 분석 결과 (-1: 과매도, 0: 중립, 1: 과매수)
pub fn overbought_oversold_analysis<C: Candle>(data: &[C]) -> i8 {
    if data.is_empty() {
        return 0;
    }

    let mut rsi_builder = RSIBuilder::<C>::new(14);
    let rsi = rsi_builder.build(data);

    if rsi.is_overbought(None) {
        1
    } else if rsi.is_oversold(None) {
        -1
    } else {
        0
    }
}
