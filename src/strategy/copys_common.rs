use super::Strategy;
use crate::analyzer::base::AnalyzerOps;
use crate::analyzer::bband_analyzer::BBandAnalyzer;
use crate::candle_store::CandleStore;
use crate::indicator::ma::MAType;
use serde::Deserialize;
use serde_json;
use trading_chart::Candle;

// analyzer에서 RSIAnalyzer 관련 구조체 가져오기
pub use crate::analyzer::rsi_analyzer::{RSIAnalyzer, RSIAnalyzerData};
pub type CopysStrategyContext<C> = crate::analyzer::rsi_analyzer::RSIAnalyzer<C>;

/// Copys 전략 공통 설정 기본 구조체
#[derive(Debug, Deserialize, Clone)]
pub struct CopysStrategyConfigBase {
    /// RSI 계산 기간
    pub rsi_period: usize,
    /// RSI 상한값 (과매수 기준)
    pub rsi_upper: f64,
    /// RSI 하한값 (과매도 기준)
    pub rsi_lower: f64,
    /// 볼린저밴드 계산 기간
    pub bband_period: usize,
    /// 볼린저밴드 표준편차 승수
    pub bband_multiplier: f64,
}

impl CopysStrategyConfigBase {
    /// 설정의 유효성을 검사합니다.
    ///
    /// 모든 설정 값이 유효한지 확인하고, 유효하지 않은 경우 오류 메시지를 반환합니다.
    ///
    /// # Returns
    /// * `Result<(), String>` - 유효성 검사 결과 (성공 또는 오류 메시지)
    pub fn validate(&self) -> Result<(), String> {
        if self.rsi_period < 2 {
            return Err("RSI 기간은 2 이상이어야 합니다".to_string());
        }

        if self.rsi_lower >= self.rsi_upper {
            return Err(format!(
                "RSI 하한값({})이 상한값({})보다 크거나 같을 수 없습니다",
                self.rsi_lower, self.rsi_upper
            ));
        }

        if self.bband_period < 2 {
            return Err("볼린저밴드 기간은 2 이상이어야 합니다".to_string());
        }

        if self.bband_multiplier <= 0.0 {
            return Err("볼린저밴드 승수는 0보다 커야 합니다".to_string());
        }

        Ok(())
    }

    /// JSON 문자열에서 설정 로드
    ///
    /// JSON 문자열로부터 설정을 로드하고, 로드에 실패할 경우 오류를 반환합니다.
    ///
    /// # Arguments
    /// * `json` - JSON 형식의 문자열
    ///
    /// # Returns
    /// * `Result<T, String>` - 로드된 설정 또는 오류
    pub fn from_json<T>(json: &str) -> Result<T, String>
    where
        T: for<'de> Deserialize<'de>,
    {
        match serde_json::from_str::<T>(json) {
            Ok(config) => Ok(config),
            Err(e) => Err(format!("JSON 설정 역직렬화 실패: {e}")),
        }
    }
}

/// Copys 전략 공통 트레이트
pub trait CopysStrategyCommon<C: Candle + 'static>: Strategy<C> {
    /// RSI 분석기 참조 반환
    fn context(&self) -> &RSIAnalyzer<C>;

    /// 볼린저밴드 분석기 참조 반환
    fn bband_analyzer(&self) -> &BBandAnalyzer<C>;

    /// 설정의 rsi_lower 반환
    fn config_rsi_lower(&self) -> f64;

    /// 설정의 rsi_upper 반환
    fn config_rsi_upper(&self) -> f64;

    /// RSI 판정 횟수 반환
    fn config_rsi_count(&self) -> usize;

    /// 볼린저밴드 기간 반환
    fn config_bband_period(&self) -> usize;

    /// 볼린저밴드 배수 반환
    fn config_bband_multiplier(&self) -> f64;

    /// 매수 신호 체크 - RSI 과매도 + 볼린저밴드 하단 터치 + 이평선 지지
    fn check_buy_signal(&self, consecutive_n: usize) -> bool {
        // 1. RSI 과매도 신호 (RSI < 30)
        let rsi_oversold = self.context().is_all(
            |data| data.rsi.value() < self.config_rsi_lower(),
            consecutive_n,
            0,
        );

        // 2. 볼린저밴드 하단 터치 반등 신호
        let bband_support = self.bband_analyzer().is_below_lower_band(1, 0)
            || self
                .bband_analyzer()
                .is_break_through_lower_band_from_below(1, 0);

        // 3. 이동평균선 지지선 역할 확인 (가격이 주요 이평선 근처에서 반등)
        let ma_support = self.check_ma_support();

        // 세 조건 중 두 개 이상 만족하면 매수 신호
        let signal_count = [rsi_oversold, bband_support, ma_support]
            .iter()
            .filter(|&&x| x)
            .count();
        signal_count >= 2
    }

    /// 매도 신호 체크 - RSI 과매수 + 볼린저밴드 상단 터치 + 이평선 저항
    fn check_sell_signal(&self, consecutive_n: usize) -> bool {
        // 1. RSI 과매수 신호 (RSI > 70)
        let rsi_overbought = self.context().is_all(
            |data| data.rsi.value() > self.config_rsi_upper(),
            consecutive_n,
            0,
        );

        // 2. 볼린저밴드 상단 터치 반락 신호
        let bband_resistance = self.bband_analyzer().is_above_upper_band(1, 0);

        // 3. 이동평균선 저항선 역할 확인 (가격이 주요 이평선 근처에서 반락)
        let ma_resistance = self.check_ma_resistance();

        // 세 조건 중 두 개 이상 만족하면 매도 신호
        let signal_count = [rsi_overbought, bband_resistance, ma_resistance]
            .iter()
            .filter(|&&x| x)
            .count();
        signal_count >= 2
    }

    /// 이동평균선 지지선 확인 (5, 20, 60, 120, 200, 240일선 중 하나라도 지지)
    fn check_ma_support(&self) -> bool {
        if self.context().items.is_empty() {
            return false;
        }

        let current_price = self.context().items[0].candle.close_price();

        // 주요 이평선들과의 거리 확인 (가격이 이평선 근처에 있으면 지지 가능)
        for i in 0..self.context().items[0].mas.len() {
            let ma_value = self.context().items[0].mas.get_by_key_index(i).get();
            let distance_percent = ((current_price - ma_value) / ma_value).abs();

            // 이평선과 2% 이내 거리에 있고, 이평선 위에 있으면 지지로 판단
            if distance_percent <= 0.02 && current_price >= ma_value {
                return true;
            }
        }

        false
    }

    /// 이동평균선 저항선 확인 (5, 20, 60, 120, 200, 240일선 중 하나라도 저항)
    fn check_ma_resistance(&self) -> bool {
        if self.context().items.is_empty() {
            return false;
        }

        let current_price = self.context().items[0].candle.close_price();

        // 주요 이평선들과의 거리 확인 (가격이 이평선 근처에 있으면 저항 가능)
        for i in 0..self.context().items[0].mas.len() {
            let ma_value = self.context().items[0].mas.get_by_key_index(i).get();
            let distance_percent = ((current_price - ma_value) / ma_value).abs();

            // 이평선과 2% 이내 거리에 있고, 이평선 아래에 있으면 저항으로 판단
            if distance_percent <= 0.02 && current_price <= ma_value {
                return true;
            }
        }

        false
    }
}

/// Copys 필터에서 임시로 사용할 컨텍스트 생성 (RSIAnalyzer 활용)
pub fn create_strategy_context_for_filter<C: Candle + 'static>(
    _symbol: &str,
    rsi_period: usize,
    ma_type: &MAType,
    ma_periods: &[usize],
    candles: &[C],
) -> anyhow::Result<RSIAnalyzer<C>> {
    // CandleStore를 직접 사용하지 않아 임시 저장소 사용
    let storage = CandleStore::<C>::new(Vec::new(), 1000, false);
    let mut analyzer = RSIAnalyzer::new(rsi_period, ma_type, ma_periods, &storage);

    // 캔들 데이터 추가
    for candle in candles.iter() {
        analyzer.next(candle.clone());
    }

    Ok(analyzer)
}
