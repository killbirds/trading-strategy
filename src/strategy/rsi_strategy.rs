use super::Strategy;
use super::StrategyType;
use super::context::{GetCandle, StrategyContextOps, StrategyDataOps};
use crate::candle_store::CandleStore;
use crate::indicator::ma::{MAType, MAs, MAsBuilder, MAsBuilderFactory};
use crate::indicator::rsi::{RSI, RSIBuilder};
use crate::model::PositionType;
use crate::model::TradePosition;
use log::info;
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::fmt::Display;
use trading_chart::Candle;

/// RSI 전략 설정
///
/// RSI(상대강도지수) 기반 트레이딩 전략에 필요한 모든 설정 파라미터를 포함합니다.
#[derive(Debug, Deserialize)]
pub struct RSIStrategyConfig {
    /// RSI 판단에 필요한 연속 데이터 수
    pub rsi_count: usize,
    /// RSI 하단 기준값 (매수 신호용)
    pub rsi_lower: f64,
    /// RSI 상단 기준값 (매도 신호용)
    pub rsi_upper: f64,
    /// RSI 계산 기간
    pub rsi_period: usize,
    /// 이동평균 유형 (EMA, SMA 등)
    pub ma: MAType,
    /// 이동평균 기간 목록 (여러 이동평균선 사용)
    pub ma_periods: Vec<usize>,
}

impl Default for RSIStrategyConfig {
    fn default() -> Self {
        RSIStrategyConfig {
            rsi_count: 3,
            rsi_lower: 30.0,
            rsi_upper: 70.0,
            rsi_period: 14,
            ma: MAType::EMA,
            ma_periods: vec![5, 20, 60],
        }
    }
}

impl RSIStrategyConfig {
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
                "RSI 하한({})은 상한({})보다 작아야 합니다",
                self.rsi_lower, self.rsi_upper
            ));
        }

        if self.rsi_count == 0 {
            return Err("RSI 판정 횟수는 0보다 커야 합니다".to_string());
        }

        if self.ma_periods.is_empty() {
            return Err("이동평균 기간이 지정되지 않았습니다".to_string());
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
    /// * `Result<RSIStrategyConfig, String>` - 로드된 설정 또는 오류
    fn from_json(json: &str) -> Result<RSIStrategyConfig, String> {
        match serde_json::from_str::<RSIStrategyConfig>(json) {
            Ok(config) => {
                config.validate()?;
                Ok(config)
            }
            Err(e) => Err(format!("JSON 설정 역직렬화 실패: {}", e)),
        }
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(config: &HashMap<String, String>) -> Result<RSIStrategyConfig, String> {
        // RSI 카운트 설정
        let rsi_count = match config.get("rsi_count") {
            Some(count) => count
                .parse::<usize>()
                .map_err(|_| "RSI 카운트 파싱 오류".to_string())?,
            None => return Err("rsi_count 설정이 필요합니다".to_string()),
        };

        // RSI 하단값 설정
        let rsi_lower = match config.get("rsi_lower") {
            Some(lower) => lower
                .parse::<f64>()
                .map_err(|_| "RSI 하단값 파싱 오류".to_string())?,
            None => return Err("rsi_lower 설정이 필요합니다".to_string()),
        };

        // RSI 상단값 설정
        let rsi_upper = match config.get("rsi_upper") {
            Some(upper) => upper
                .parse::<f64>()
                .map_err(|_| "RSI 상단값 파싱 오류".to_string())?,
            None => return Err("rsi_upper 설정이 필요합니다".to_string()),
        };

        // RSI 기간 설정
        let rsi_period = match config.get("rsi_period") {
            Some(period) => {
                let value = period
                    .parse::<usize>()
                    .map_err(|_| "RSI 기간 파싱 오류".to_string())?;

                if value < 2 {
                    return Err("RSI 기간은 2 이상이어야 합니다".to_string());
                }

                value
            }
            None => return Err("rsi_period 설정이 필요합니다".to_string()),
        };

        // MA 유형 설정
        let ma = match config.get("ma") {
            Some(ma_type) => match ma_type.to_lowercase().as_str() {
                "sma" => MAType::SMA,
                "ema" => MAType::EMA,
                _ => return Err(format!("알 수 없는 이동평균 유형: {}", ma_type)),
            },
            None => return Err("ma 설정이 필요합니다".to_string()),
        };

        // 이동평균 기간 설정
        let ma_periods = match config.get("ma_periods") {
            Some(periods) => {
                let periods_vec = crate::strategy::split_safe::<usize>(periods)
                    .map_err(|e| format!("이동평균 기간 파싱 오류: {}", e))?;

                if periods_vec.is_empty() {
                    return Err("이동평균 기간이 지정되지 않았습니다".to_string());
                }

                periods_vec
            }
            None => return Err("ma_periods 설정이 필요합니다".to_string()),
        };

        // 유효성 검사
        if rsi_lower >= rsi_upper {
            return Err(format!(
                "RSI 하단값({})이 상단값({})보다 크거나 같을 수 없습니다",
                rsi_lower, rsi_upper
            ));
        }

        let result = RSIStrategyConfig {
            rsi_count,
            rsi_lower,
            rsi_upper,
            rsi_period,
            ma,
            ma_periods,
        };

        result.validate()?;
        Ok(result)
    }
}

/// RSI 전략 데이터
///
/// 단일 시점의 RSI 전략 관련 모든 데이터를 포함합니다.
#[derive(Debug)]
struct StrategyData<C: Candle> {
    /// 현재 캔들 데이터
    candle: C,
    /// RSI 지표 값
    rsi: RSI,
    /// 이동평균선 집합
    mas: MAs,
}

impl<C: Candle> StrategyData<C> {
    /// 새 전략 데이터 생성
    ///
    /// # Arguments
    /// * `candle` - 캔들 데이터
    /// * `rsi` - RSI 지표 값
    /// * `mas` - 이동평균선 집합
    ///
    /// # Returns
    /// * `StrategyData` - 새로운 전략 데이터 인스턴스
    fn new(candle: C, mas: MAs, rsi: RSI) -> StrategyData<C> {
        StrategyData { candle, rsi, mas }
    }

    /// 이동평균이 정규 배열(오름차순)인지 확인
    ///
    /// 이동평균선이 짧은 기간부터 긴 기간까지 오름차순으로 배열되어 있는지 확인합니다.
    /// 이는 일반적으로 상승 추세를 나타냅니다.
    ///
    /// # Returns
    /// * `bool` - 정규 배열 여부
    fn is_ma_regular_arrangement(&self) -> bool {
        self.is_regular_arrangement(|data| &data.mas, |ma| ma.get())
    }

    /// 이동평균이 역배열(내림차순)인지 확인
    ///
    /// 이동평균선이 짧은 기간부터 긴 기간까지 내림차순으로 배열되어 있는지 확인합니다.
    /// 이는 일반적으로 하락 추세를 나타냅니다.
    ///
    /// # Returns
    /// * `bool` - 역배열 여부
    fn is_ma_reverse_arrangement(&self) -> bool {
        self.is_reverse_arrangement(|data| &data.mas, |ma| ma.get())
    }
}

impl<C: Candle> GetCandle<C> for StrategyData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> StrategyDataOps<C> for StrategyData<C> {}

/// RSI 전략 컨텍스트
///
/// RSI 전략을 위한 데이터와 상태를 관리합니다.
#[derive(Debug)]
struct StrategyContext<C: Candle> {
    /// RSI 빌더
    rsibuilder: RSIBuilder<C>,
    /// 이동평균 빌더
    masbuilder: MAsBuilder<C>,
    /// 전략 데이터 히스토리 (최신 데이터가 인덱스 0)
    items: Vec<StrategyData<C>>,
}

impl<C: Candle> Display for StrategyContext<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.split_first() {
            Some((head, tail)) => {
                write!(
                    f,
                    "캔들: {}, RSI: [{}, {:?}], MAs: {}",
                    head.candle,
                    head.rsi,
                    tail.iter()
                        .take(4)
                        .map(|item| item.rsi.rsi)
                        .collect::<Vec<_>>(),
                    head.mas
                )
            }
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> StrategyContext<C> {
    /// 새 전략 컨텍스트 생성
    ///
    /// # Arguments
    /// * `config` - 전략 설정
    /// * `storage` - 캔들 데이터 저장소
    ///
    /// # Returns
    /// * `StrategyContext` - 초기화된 전략 컨텍스트
    fn new(config: &RSIStrategyConfig, storage: &CandleStore<C>) -> StrategyContext<C> {
        let rsibuilder = RSIBuilder::new(config.rsi_period);
        let masbuilder = MAsBuilderFactory::build::<C>(&config.ma, &config.ma_periods);
        let mut ctx = StrategyContext {
            rsibuilder,
            masbuilder,
            items: vec![],
        };

        ctx.init(storage.get_reversed_items());
        ctx
    }

    /// n개의 연속 데이터에서 이동평균이 정규 배열인지 확인
    ///
    /// # Arguments
    /// * `n` - 확인할 연속 데이터 개수
    ///
    /// # Returns
    /// * `bool` - 모든 데이터가 정규 배열인지 여부
    fn is_ma_regular_arrangement(&self, n: usize) -> bool {
        self.is_all(|data| data.is_ma_regular_arrangement(), n)
    }

    /// n개의 연속 데이터에서 이동평균이 역배열인지 확인
    ///
    /// # Arguments
    /// * `n` - 확인할 연속 데이터 개수
    ///
    /// # Returns
    /// * `bool` - 모든 데이터가 역배열인지 여부
    fn is_ma_reverse_arrangement(&self, n: usize) -> bool {
        self.is_all(|data| data.is_ma_reverse_arrangement(), n)
    }
}

impl<C: Candle> StrategyContextOps<StrategyData<C>, C> for StrategyContext<C> {
    fn next_data(&mut self, candle: C) -> StrategyData<C> {
        let rsi = self.rsibuilder.next(&candle);
        let mas = self.masbuilder.next(&candle);
        StrategyData::new(candle, mas, rsi)
    }

    fn datum(&self) -> &Vec<StrategyData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<StrategyData<C>> {
        &mut self.items
    }
}

/// RSI 기반 트레이딩 전략
///
/// RSI(상대강도지수)와 이동평균을 사용한 트레이딩 전략을 구현합니다.
#[derive(Debug)]
pub struct RSIStrategy<C: Candle> {
    /// 전략 설정
    config: RSIStrategyConfig,
    /// 전략 컨텍스트 (데이터 보관 및 연산)
    ctx: StrategyContext<C>,
}

impl<C: Candle> Display for RSIStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[RSI전략] 설정: {{RSI기간: {}, 하한: {}, 상한: {}}}, 컨텍스트: {}",
            self.config.rsi_period, self.config.rsi_lower, self.config.rsi_upper, self.ctx
        )
    }
}

impl<C: Candle + 'static> RSIStrategy<C> {
    /// 새 RSI 전략 인스턴스 생성 (JSON 설정 파일 사용)
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `json_config` - JSON 형식의 설정 문자열
    ///
    /// # Returns
    /// * `Result<RSIStrategy<C>, String>` - 초기화된 RSI 전략 인스턴스 또는 오류
    pub fn new(storage: &CandleStore<C>, json_config: &str) -> Result<RSIStrategy<C>, String> {
        let config = RSIStrategyConfig::from_json(json_config)?;
        info!("RSI 전략 설정: {:?}", config);
        let ctx = StrategyContext::new(&config, storage);

        Ok(RSIStrategy { config, ctx })
    }

    /// 새 RSI 전략 인스턴스 생성 (설정 직접 제공)
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `config` - 전략 설정 (HashMap 형태)
    ///
    /// # Returns
    /// * `Result<RSIStrategy<C>, String>` - 초기화된 RSI 전략 인스턴스 또는 오류
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<RSIStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => RSIStrategyConfig::from_hash_map(&cfg)?,
            None => RSIStrategyConfig::default(),
        };

        info!("RSI 전략 설정: {:?}", strategy_config);
        let ctx = StrategyContext::new(&strategy_config, storage);

        Ok(RSIStrategy {
            config: strategy_config,
            ctx,
        })
    }

    /// RSI가 과매수 영역인지 확인
    fn is_rsi_overbought(&self) -> bool {
        if self.ctx.items.len() < self.config.rsi_count {
            return false;
        }

        // 과매수 판단: RSI가 상단 임계값을 넘어서면 과매수로 판단
        self.ctx
            .items
            .iter()
            .take(self.config.rsi_count)
            .all(|item| item.rsi.rsi > self.config.rsi_upper)
    }

    /// RSI가 과매도 영역인지 확인
    fn is_rsi_oversold(&self) -> bool {
        if self.ctx.items.len() < self.config.rsi_count {
            return false;
        }

        // 과매도 판단: RSI가 하단 임계값 아래로 내려가면 과매도로 판단
        self.ctx
            .items
            .iter()
            .take(self.config.rsi_count)
            .all(|item| item.rsi.rsi < self.config.rsi_lower)
    }
}

impl<C: Candle + 'static> Strategy<C> for RSIStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle)
    }

    fn should_enter(&self, _candle: &C) -> bool {
        // RSI가 과매도 구간에서 진입
        self.is_rsi_oversold()
    }

    fn should_exit(&self, _holdings: &TradePosition, _candle: &C) -> bool {
        // RSI가 과매수 구간에서 청산
        self.is_rsi_overbought()
    }

    fn get_position(&self) -> PositionType {
        PositionType::Long
    }

    fn get_name(&self) -> StrategyType {
        StrategyType::RSI
    }
}
