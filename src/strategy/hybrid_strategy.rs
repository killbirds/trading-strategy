use super::Strategy;
use super::StrategyType;
use super::context::{GetCandle, StrategyContextOps, StrategyDataOps};
use crate::candle_store::CandleStore;
use crate::indicator::TABuilder;
use crate::indicator::ma::{MA, MABuilderFactory, MAType};
use crate::indicator::macd::{MACD, MACDBuilder};
use crate::indicator::rsi::{RSI, RSIBuilder};
use crate::model::PositionType;
use crate::model::TradePosition;
use log::info;
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::fmt::Display;
use trading_chart::Candle;

/// 하이브리드 전략 설정
#[derive(Debug, Deserialize)]
pub struct HybridStrategyConfig {
    /// RSI 계산 기간
    pub rsi_period: usize,
    /// RSI 상한값
    pub rsi_upper: f64,
    /// RSI 하한값
    pub rsi_lower: f64,
    /// RSI 조건 판단 기간
    pub rsi_count: usize,
    /// 볼린저밴드 계산 기간
    pub bband_period: usize,
    /// 볼린저밴드 표준편차 승수
    pub bband_multiplier: f64,
    /// 마켓 순위 정렬 기준 이동평균 기간
    pub ma_rank_period: usize,
    /// 이동평균 타입
    pub ma_type: MAType,
    /// 이동평균 기간
    pub ma_period: usize,
    /// MACD 빠른 기간
    pub macd_fast_period: usize,
    /// MACD 느린 기간
    pub macd_slow_period: usize,
    /// MACD 시그널 기간
    pub macd_signal_period: usize,
}

impl Default for HybridStrategyConfig {
    /// 기본 설정값 반환
    fn default() -> Self {
        HybridStrategyConfig {
            rsi_period: 14,
            rsi_upper: 70.0,
            rsi_lower: 30.0,
            rsi_count: 3,
            bband_period: 20,
            bband_multiplier: 2.0,
            ma_rank_period: 20,
            ma_type: MAType::SMA,
            ma_period: 20,
            macd_fast_period: 12,
            macd_slow_period: 26,
            macd_signal_period: 9,
        }
    }
}

impl HybridStrategyConfig {
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

        if self.rsi_count == 0 {
            return Err("RSI 판정 횟수는 0보다 커야 합니다".to_string());
        }

        if self.bband_period < 2 {
            return Err("볼린저밴드 기간은 2 이상이어야 합니다".to_string());
        }

        if self.bband_multiplier <= 0.0 {
            return Err("볼린저밴드 승수는 0보다 커야 합니다".to_string());
        }

        if self.ma_rank_period < 2 {
            return Err("마켓 랭크 이동평균 기간은 2 이상이어야 합니다".to_string());
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
    /// * `Result<HybridStrategyConfig, String>` - 로드된 설정 또는 오류
    fn from_json(json: &str) -> Result<HybridStrategyConfig, String> {
        match serde_json::from_str::<HybridStrategyConfig>(json) {
            Ok(config) => {
                config.validate()?;
                Ok(config)
            }
            Err(e) => Err(format!("JSON 설정 역직렬화 실패: {}", e)),
        }
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(config: &HashMap<String, String>) -> Result<HybridStrategyConfig, String> {
        // 카운트 설정
        let count = match config.get("count") {
            Some(value_str) => {
                let value = value_str
                    .parse::<usize>()
                    .map_err(|_| "카운트 파싱 오류".to_string())?;

                if value == 0 {
                    return Err("카운트는 0보다 커야 합니다".to_string());
                }

                value
            }
            None => return Err("count 설정이 필요합니다".to_string()),
        };

        // 이동평균 관련 설정
        let ma_type = match config.get("ma_type").map(|s| s.as_str()) {
            Some("sma") => MAType::SMA,
            Some("ema") => MAType::EMA,
            Some(unknown) => return Err(format!("알 수 없는 이동평균 유형: {}", unknown)),
            None => return Err("ma_type 설정이 필요합니다".to_string()),
        };

        let ma_period = match config.get("ma_period") {
            Some(value_str) => {
                let value = value_str
                    .parse::<usize>()
                    .map_err(|_| "이동평균 기간 파싱 오류".to_string())?;

                if value == 0 {
                    return Err("이동평균 기간은 0보다 커야 합니다".to_string());
                }

                value
            }
            None => return Err("ma_period 설정이 필요합니다".to_string()),
        };

        // MACD 관련 설정
        let macd_fast_period = match config.get("macd_fast_period") {
            Some(value_str) => {
                let value = value_str
                    .parse::<usize>()
                    .map_err(|_| "MACD 빠른 기간 파싱 오류".to_string())?;

                if value == 0 {
                    return Err("MACD 빠른 기간은 0보다 커야 합니다".to_string());
                }

                value
            }
            None => return Err("macd_fast_period 설정이 필요합니다".to_string()),
        };

        let macd_slow_period = match config.get("macd_slow_period") {
            Some(value_str) => {
                let value = value_str
                    .parse::<usize>()
                    .map_err(|_| "MACD 느린 기간 파싱 오류".to_string())?;

                if value == 0 {
                    return Err("MACD 느린 기간은 0보다 커야 합니다".to_string());
                }

                value
            }
            None => return Err("macd_slow_period 설정이 필요합니다".to_string()),
        };

        if macd_fast_period >= macd_slow_period {
            return Err(format!(
                "MACD 빠른 기간({})은 느린 기간({})보다 작아야 합니다",
                macd_fast_period, macd_slow_period
            ));
        }

        let macd_signal_period = match config.get("macd_signal_period") {
            Some(value_str) => {
                let value = value_str
                    .parse::<usize>()
                    .map_err(|_| "MACD 시그널 기간 파싱 오류".to_string())?;

                if value == 0 {
                    return Err("MACD 시그널 기간은 0보다 커야 합니다".to_string());
                }

                value
            }
            None => return Err("macd_signal_period 설정이 필요합니다".to_string()),
        };

        // RSI 관련 설정
        let rsi_period = match config.get("rsi_period") {
            Some(value_str) => {
                let value = value_str
                    .parse::<usize>()
                    .map_err(|_| "RSI 기간 파싱 오류".to_string())?;

                if value == 0 {
                    return Err("RSI 기간은 0보다 커야 합니다".to_string());
                }

                value
            }
            None => return Err("rsi_period 설정이 필요합니다".to_string()),
        };

        let rsi_lower = match config.get("rsi_lower") {
            Some(value_str) => {
                let value = value_str
                    .parse::<f64>()
                    .map_err(|_| "RSI 하한값 파싱 오류".to_string())?;

                if !(0.0..=100.0).contains(&value) {
                    return Err(format!("RSI 하한값({})은 0과 100 사이여야 합니다", value));
                }

                value
            }
            None => return Err("rsi_lower 설정이 필요합니다".to_string()),
        };

        let rsi_upper = match config.get("rsi_upper") {
            Some(value_str) => {
                let value = value_str
                    .parse::<f64>()
                    .map_err(|_| "RSI 상한값 파싱 오류".to_string())?;

                if !(0.0..=100.0).contains(&value) {
                    return Err(format!("RSI 상한값({})은 0과 100 사이여야 합니다", value));
                }

                value
            }
            None => return Err("rsi_upper 설정이 필요합니다".to_string()),
        };

        if rsi_lower >= rsi_upper {
            return Err(format!(
                "RSI 하한값({})은 상한값({})보다 작아야 합니다",
                rsi_lower, rsi_upper
            ));
        }

        Ok(HybridStrategyConfig {
            rsi_count: count,
            ma_type,
            ma_period,
            macd_fast_period,
            macd_slow_period,
            macd_signal_period,
            rsi_period,
            rsi_lower,
            rsi_upper,
            bband_period: config
                .get("bband_period")
                .map_or(20, |s| s.parse::<usize>().unwrap()),
            bband_multiplier: config
                .get("bband_multiplier")
                .map_or(2.0, |s| s.parse::<f64>().unwrap()),
            ma_rank_period: config
                .get("ma_rank_period")
                .map_or(20, |s| s.parse::<usize>().unwrap()),
        })
    }
}

struct StrategyData<C: Candle> {
    candle: C,
    ma: Box<dyn MA>,
    macd: MACD,
    rsi: RSI,
}

impl<C: Candle + Clone> StrategyData<C> {
    fn new(candle: C, ma: Box<dyn MA>, macd: MACD, rsi: RSI) -> StrategyData<C> {
        StrategyData {
            candle,
            ma,
            macd,
            rsi,
        }
    }

    fn clone_with_stored_values(&self) -> StrategyData<C> {
        // Box<dyn MA>는 클론할 수 없으므로, MA 구현체의 값을 저장하고 새 객체 생성
        let ma_period = self.ma.period();
        let ma_value = self.ma.get();

        // 값을 가지고 있는 간단한 MA 구현체
        struct SimpleMA {
            period: usize,
            value: f64,
        }

        impl MA for SimpleMA {
            fn period(&self) -> usize {
                self.period
            }

            fn get(&self) -> f64 {
                self.value
            }
        }

        impl Display for SimpleMA {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "MA({}: {:.2})", self.period, self.value)
            }
        }

        impl std::fmt::Debug for SimpleMA {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "SimpleMA({}: {:.2})", self.period, self.value)
            }
        }

        let simple_ma = SimpleMA {
            period: ma_period,
            value: ma_value,
        };

        StrategyData {
            candle: self.candle.clone(),
            ma: Box::new(simple_ma),
            macd: self.macd.clone(),
            rsi: self.rsi.clone(),
        }
    }
}

impl<C: Candle> GetCandle<C> for StrategyData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> StrategyDataOps<C> for StrategyData<C> {}

struct StrategyContext<C: Candle + Clone> {
    mabuilder: Box<dyn TABuilder<Box<dyn MA>, C>>,
    macdbuilder: MACDBuilder<C>,
    rsibuilder: RSIBuilder<C>,
    items: Vec<StrategyData<C>>,
}

impl<C: Candle + Clone> Display for StrategyContext<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(first) = self.items.first() {
            write!(
                f,
                "candle: {}, ma: {:.2}, macd: {}, rsi: {:.2}",
                first.candle,
                first.ma.get(),
                first.macd,
                first.rsi.value()
            )
        } else {
            write!(f, "데이터 없음")
        }
    }
}

impl<C: Candle + 'static> StrategyContext<C> {
    fn new(config: &HybridStrategyConfig, storage: &CandleStore<C>) -> StrategyContext<C> {
        let mabuilder = MABuilderFactory::build(&config.ma_type, config.ma_period);
        let macdbuilder = MACDBuilder::new(
            config.macd_fast_period,
            config.macd_slow_period,
            config.macd_signal_period,
        );
        let rsibuilder = RSIBuilder::new(config.rsi_period);

        let mut ctx = StrategyContext {
            mabuilder,
            macdbuilder,
            rsibuilder,
            items: vec![],
        };

        ctx.init(storage.get_reversed_items());
        ctx
    }

    fn init(&mut self, candles: Vec<C>) {
        for candle in candles {
            let _ = self.next_data(candle);
        }
    }
}

impl<C: Candle + Clone> StrategyContextOps<StrategyData<C>, C> for StrategyContext<C> {
    fn next_data(&mut self, candle: C) -> StrategyData<C> {
        let ma = self.mabuilder.next(&candle);
        let macd = self.macdbuilder.next(&candle);
        let rsi = self.rsibuilder.next(&candle);

        let data = StrategyData::new(candle, ma, macd, rsi);
        self.items.push(data.clone_with_stored_values());

        if self.items.len() > 100 {
            self.items.remove(0);
        }

        data
    }

    fn datum(&self) -> &Vec<StrategyData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<StrategyData<C>> {
        &mut self.items
    }
}

/// 하이브리드 전략 구현
///
/// MACD, RSI, 이동평균선을 결합하여 시장 상황에 따라 적응적으로 대응하는 전략
pub struct HybridStrategy<C: Candle + Clone> {
    config: HybridStrategyConfig,
    ctx: StrategyContext<C>,
}

impl<C: Candle + Clone> Display for HybridStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[하이브리드전략] 설정: {{RSI: {}(상:{}/하:{}), MACD: {}/{}/{}}}, 컨텍스트: {}",
            self.config.rsi_period,
            self.config.rsi_upper,
            self.config.rsi_lower,
            self.config.macd_fast_period,
            self.config.macd_slow_period,
            self.config.macd_signal_period,
            self.ctx
        )
    }
}

impl<C: Candle + 'static> HybridStrategy<C> {
    /// 새 하이브리드 전략 인스턴스 생성 (JSON 설정 파일 사용)
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `json_config` - JSON 형식의 설정 문자열
    ///
    /// # Returns
    /// * `Result<HybridStrategy<C>, String>` - 초기화된 하이브리드 전략 인스턴스 또는 오류
    pub fn new(storage: &CandleStore<C>, json_config: &str) -> Result<HybridStrategy<C>, String> {
        let config = HybridStrategyConfig::from_json(json_config)?;
        info!("하이브리드 전략 설정: {:?}", config);
        let ctx = StrategyContext::new(&config, storage);

        Ok(HybridStrategy { config, ctx })
    }

    /// 새 하이브리드 전략 인스턴스 생성 (설정 직접 제공)
    ///
    /// # 인자
    ///
    /// * `storage` - 캔들 데이터 저장소
    /// * `config` - 전략 설정 맵
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<HybridStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => HybridStrategyConfig::from_hash_map(&cfg)?,
            None => HybridStrategyConfig::default(),
        };

        let ctx = StrategyContext::new(&strategy_config, storage);

        Ok(HybridStrategy {
            config: strategy_config,
            ctx,
        })
    }

    /// 여러 지표를 종합하여 매수 신호 강도 계산
    ///
    /// # Returns
    ///
    /// * `f64` - 매수 신호 강도 (0.0 ~ 1.0)
    fn calculate_buy_signal_strength(&self) -> f64 {
        if self.ctx.items.len() < 2 {
            return 0.0;
        }

        let current = self.ctx.items.last().unwrap();
        let previous = &self.ctx.items[self.ctx.items.len() - 2];

        let mut strength = 0.0;
        let mut count = 0.0;

        // 1. 이동평균선 기반 신호
        if current.candle.close_price() > current.ma.get() {
            strength += 1.0;
            count += 1.0;
        }

        // 2. MACD 기반 신호
        if current.macd.histogram > 0.0 && previous.macd.histogram < 0.0 {
            // MACD 히스토그램이 0선을 상향 돌파 (강한 매수 신호)
            strength += 2.0;
            count += 1.0;
        } else if current.macd.histogram > 0.0 {
            // MACD 히스토그램이 0선 위에 있음 (약한 매수 신호)
            strength += 0.5;
            count += 1.0;
        }

        // 3. RSI 기반 신호
        let rsi = current.rsi.value();
        if rsi < self.config.rsi_lower && rsi > previous.rsi.value() {
            // RSI가 과매도 상태에서 반등 (강한 매수 신호)
            strength += 2.0;
            count += 1.0;
        } else if rsi > self.config.rsi_lower && rsi < 50.0 {
            // RSI가 과매도 상태를 벗어나 상승 중 (약한 매수 신호)
            strength += 0.5;
            count += 1.0;
        }

        // 최종 강도 계산 (정규화)
        if count > 0.0 {
            strength / (count * 2.0) // 최대 강도를 기준으로 정규화
        } else {
            0.0
        }
    }

    /// 여러 지표를 종합하여 매도 신호 강도 계산
    ///
    /// # Returns
    ///
    /// * `f64` - 매도 신호 강도 (0.0 ~ 1.0)
    fn calculate_sell_signal_strength(&self, profit_percentage: f64) -> f64 {
        if self.ctx.items.len() < 2 {
            return 0.0;
        }

        let current = self.ctx.items.last().unwrap();
        let previous = &self.ctx.items[self.ctx.items.len() - 2];

        let mut strength = 0.0;
        let mut count = 0.0;

        // 1. 이동평균선 기반 신호
        if current.candle.close_price() < current.ma.get() {
            strength += 1.0;
            count += 1.0;
        }

        // 2. MACD 기반 신호
        if current.macd.histogram < 0.0 && previous.macd.histogram > 0.0 {
            // MACD 히스토그램이 0선을 하향 돌파 (강한 매도 신호)
            strength += 2.0;
            count += 1.0;
        } else if current.macd.histogram < 0.0 {
            // MACD 히스토그램이 0선 아래에 있음 (약한 매도 신호)
            strength += 0.5;
            count += 1.0;
        }

        // 3. RSI 기반 신호
        let rsi = current.rsi.value();
        if rsi > self.config.rsi_upper && rsi < previous.rsi.value() {
            // RSI가 과매수 상태에서 하락 (강한 매도 신호)
            strength += 2.0;
            count += 1.0;
        } else if rsi < self.config.rsi_upper && rsi > 50.0 {
            // RSI가 과매수 상태로 접근 중 (약한 매도 신호)
            strength += 0.5;
            count += 1.0;
        }

        // 4. 수익률 기반 신호
        if profit_percentage > 5.0 {
            // 5% 이상 수익 (적절한 매도 신호)
            strength += 1.0;
            count += 1.0;
        } else if profit_percentage < -3.0 {
            // 3% 이상 손실 (손절 매도 신호)
            strength += 1.5;
            count += 1.0;
        }

        // 최종 강도 계산 (정규화)
        if count > 0.0 {
            strength / (count * 2.0) // 최대 강도를 기준으로 정규화
        } else {
            0.0
        }
    }
}

impl<C: Candle + 'static> Strategy<C> for HybridStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next_data(candle);
    }

    fn should_enter(&self, _candle: &C) -> bool {
        // 여러 지표를 종합한 매수 신호를 기반으로 결정
        let signal_strength = self.calculate_buy_signal_strength();

        // 신호 강도가 0.7 이상인 경우에만 매수 (임계값은 조정 가능)
        signal_strength >= 0.7
    }

    fn should_exit(&self, holdings: &TradePosition, _candle: &C) -> bool {
        if self.ctx.items.is_empty() {
            return false;
        }

        // 현재 가격
        let current_price = self.ctx.items.last().unwrap().candle.close_price();

        // 현재 수익률 계산
        let profit_percentage = (current_price / holdings.price - 1.0) * 100.0;

        // 여러 지표를 종합한 매도 신호를 기반으로 결정
        let signal_strength = self.calculate_sell_signal_strength(profit_percentage);

        // 신호 강도가 0.6 이상인 경우에만 매도 (임계값은 조정 가능)
        // 또는 10% 이상 수익 시 현재 신호와 관계없이 매도 (이익 확정)
        // 또는 7% 이상 손실 시 현재 신호와 관계없이 매도 (손절)
        signal_strength >= 0.6 || !(-7.0..=10.0).contains(&profit_percentage)
    }

    fn get_position(&self) -> PositionType {
        PositionType::Long
    }

    fn get_name(&self) -> StrategyType {
        StrategyType::Hybrid
    }
}
