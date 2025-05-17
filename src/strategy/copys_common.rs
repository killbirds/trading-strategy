use super::Strategy;
use crate::analyzer::base::AnalyzerOps;
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
    /// RSI 상한값
    pub rsi_upper: f64,
    /// RSI 하한값
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
            Err(e) => Err(format!("JSON 설정 역직렬화 실패: {}", e)),
        }
    }
}

/// Copys 전략 공통 트레이트
pub trait CopysStrategyCommon<C: Candle + 'static>: Strategy<C> {
    /// 분석기 참조 반환
    fn context(&self) -> &RSIAnalyzer<C>;

    /// 설정의 rsi_lower 반환
    fn config_rsi_lower(&self) -> f64;

    /// 설정의 rsi_upper 반환
    fn config_rsi_upper(&self) -> f64;

    /// RSI 판정 횟수 반환
    fn config_rsi_count(&self) -> usize;

    /// 매수 신호 체크 (for 필터)
    fn check_buy_signal(&self, consecutive_n: usize) -> bool {
        // 이동평균선이 역배열이면 진입하지 않음
        if self.context().is_ma_reverse_arrangement(1) {
            return false;
        }

        // RSI가 하한선보다 낮으면 매수 신호
        self.context().is_break_through_by_satisfying(
            |data| data.rsi.value < self.config_rsi_lower(),
            1,
            consecutive_n,
        )
    }

    /// 매도 신호 체크 (for 필터)
    fn check_sell_signal(&self, consecutive_n: usize) -> bool {
        // 이동평균선이 정배열이면 청산하지 않음
        if self.context().is_ma_regular_arrangement(1) {
            return false;
        }

        // RSI가 상한선보다 높으면 매도 신호
        self.context().is_break_through_by_satisfying(
            |data| data.rsi.value > self.config_rsi_upper(),
            1,
            consecutive_n,
        )
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
