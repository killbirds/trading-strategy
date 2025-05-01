use super::StrategyType;
use super::context::{GetCandle, StrategyContextOps, StrategyDataOps};
use super::{Strategy, split_safe};
use crate::candle_store::CandleStore;
use crate::indicator::ma::{MAType, MAs, MAsBuilder, MAsBuilderFactory};
use crate::model::PositionType;
use crate::model::TradePosition;
use log::info;
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::fmt::Display;
use trading_chart::Candle;

/// 이동평균(MA) 전략 설정
#[derive(Debug, Deserialize)]
pub struct MAStrategyConfig {
    /// 이동평균 계산 방식 (SMA, EMA 등)
    pub ma: MAType,
    /// 이동평균 기간 목록 (짧은 것부터 긴 것 순)
    pub ma_periods: Vec<usize>,
    /// 골든 크로스/데드 크로스 판정 조건: 이전 기간
    pub cross_previous_periods: usize,
}

impl Default for MAStrategyConfig {
    /// 기본 설정값 반환
    fn default() -> Self {
        MAStrategyConfig {
            ma: MAType::EMA,
            ma_periods: vec![5, 20, 60],
            cross_previous_periods: 15,
        }
    }
}

impl MAStrategyConfig {
    /// 설정의 유효성을 검사합니다.
    ///
    /// 모든 설정 값이 유효한지 확인하고, 유효하지 않은 경우 오류 메시지를 반환합니다.
    ///
    /// # Returns
    /// * `Result<(), String>` - 유효성 검사 결과 (성공 또는 오류 메시지)
    pub fn validate(&self) -> Result<(), String> {
        if self.ma_periods.is_empty() {
            return Err("이동평균 기간이 지정되지 않았습니다".to_string());
        }

        // 기간이 오름차순으로 정렬되어 있는지 확인
        for i in 1..self.ma_periods.len() {
            if self.ma_periods[i] <= self.ma_periods[i - 1] {
                return Err(format!(
                    "이동평균 기간은 오름차순으로 정렬되어야 합니다: {:?}",
                    self.ma_periods
                ));
            }
        }

        if self.cross_previous_periods == 0 {
            return Err("크로스 판정 기간은 0보다 커야 합니다".to_string());
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
    /// * `Result<MAStrategyConfig, String>` - 로드된 설정 또는 오류
    fn from_json(json: &str) -> Result<MAStrategyConfig, String> {
        match serde_json::from_str::<MAStrategyConfig>(json) {
            Ok(config) => {
                config.validate()?;
                Ok(config)
            }
            Err(e) => Err(format!("JSON 설정 역직렬화 실패: {}", e)),
        }
    }

    /// HashMap에서 설정 로드
    fn from_hash_map(config: &HashMap<String, String>) -> Result<MAStrategyConfig, String> {
        // ma 유형 설정
        let ma = match config.get("ma") {
            Some(ma_type) => match ma_type.to_lowercase().as_str() {
                "sma" => MAType::SMA,
                "ema" => MAType::EMA,
                "wma" => MAType::WMA,
                _ => return Err(format!("알 수 없는 이동평균 유형: {}", ma_type)),
            },
            None => return Err("ma 설정이 필요합니다".to_string()),
        };

        // 이동평균 기간 설정
        let ma_periods = match config.get("ma_periods") {
            Some(periods) => {
                let periods_vec = split_safe::<usize>(periods)
                    .map_err(|e| format!("이동평균 기간 파싱 오류: {}", e))?;

                if periods_vec.is_empty() {
                    return Err("이동평균 기간이 지정되지 않았습니다".to_string());
                }

                // 기간이 오름차순으로 정렬되어 있는지 확인
                for i in 1..periods_vec.len() {
                    if periods_vec[i] <= periods_vec[i - 1] {
                        return Err(format!(
                            "이동평균 기간은 오름차순으로 정렬되어야 합니다: {:?}",
                            periods_vec
                        ));
                    }
                }

                periods_vec
            }
            None => return Err("ma_periods 설정이 필요합니다".to_string()),
        };

        // 크로스 판정 기간 설정
        let cross_previous_periods = match config.get("cross_previous_periods") {
            Some(cross_periods) => {
                let periods = cross_periods
                    .parse::<usize>()
                    .map_err(|_| "크로스 판정 기간 파싱 오류".to_string())?;

                if periods == 0 {
                    return Err("크로스 판정 기간은 0보다 커야 합니다".to_string());
                }

                periods
            }
            None => return Err("cross_previous_periods 설정이 필요합니다".to_string()),
        };

        let result = MAStrategyConfig {
            ma,
            ma_periods,
            cross_previous_periods,
        };

        result.validate()?;
        Ok(result)
    }
}

/// MA 전략 데이터
#[derive(Debug)]
struct StrategyData<C: Candle> {
    /// 현재 캔들 데이터
    candle: C,
    /// 이동평균선 집합
    mas: MAs,
}

impl<C: Candle> StrategyData<C> {
    /// 새 전략 데이터 생성
    fn new(candle: C, mas: MAs) -> StrategyData<C> {
        StrategyData { candle, mas }
    }

    /// 이동평균이 정규 배열(오름차순)인지 확인
    fn is_ma_regular_arrangement(&self) -> bool {
        self.is_regular_arrangement(|data| &data.mas, |ma| ma.get())
    }

    /// 이동평균이 역배열(내림차순)인지 확인
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

/// MA 전략 컨텍스트
#[derive(Debug)]
struct StrategyContext<C: Candle> {
    /// 이동평균 빌더
    masbuilder: MAsBuilder<C>,
    /// 전략 데이터 히스토리 (최신 데이터가 인덱스 0)
    items: Vec<StrategyData<C>>,
}

impl<C: Candle> Display for StrategyContext<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(f, "캔들: {}, MAs: {}", first.candle, first.mas),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> StrategyContext<C> {
    /// 새 전략 컨텍스트 생성
    fn new(config: &MAStrategyConfig, storage: &CandleStore<C>) -> StrategyContext<C> {
        let masbuilder = MAsBuilderFactory::build::<C>(&config.ma, &config.ma_periods);
        let mut ctx = StrategyContext {
            masbuilder,
            items: vec![],
        };
        ctx.init(storage.get_reversed_items());
        ctx
    }

    /// 특정 인덱스의 이동평균 수익률이 목표치보다 작은지 확인
    fn is_ma_less_than_rate_of_return(&self, index: usize, rate_of_return: f64, n: usize) -> bool {
        self.is_all(
            |data| {
                data.is_less_than_target(
                    |data| {
                        data.get_rate_of_return(|data| {
                            let ma = data.mas.get_from_index(index);
                            ma.get()
                        })
                    },
                    rate_of_return,
                )
            },
            n,
        )
    }

    /// 특정 인덱스의 이동평균 수익률이 목표치보다 큰지 확인
    fn is_ma_greater_than_rate_of_return(
        &self,
        index: usize,
        rate_of_return: f64,
        n: usize,
    ) -> bool {
        self.is_all(
            |data| {
                data.is_greater_than_target(
                    |data| {
                        data.get_rate_of_return(|data| {
                            let ma = data.mas.get_from_index(index);
                            ma.get()
                        })
                    },
                    rate_of_return,
                )
            },
            n,
        )
    }

    /// n개의 연속 데이터에서 이동평균이 정규 배열인지 확인
    fn is_ma_regular_arrangement(&self, n: usize) -> bool {
        self.is_all(|data| data.is_ma_regular_arrangement(), n)
    }

    /// 골든 크로스 패턴 확인 (정규 배열이 n개 연속, 이전 m개는 아님)
    fn is_ma_regular_arrangement_golden_cross(&self, n: usize, m: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_ma_regular_arrangement(), n, m)
    }

    /// n개의 연속 데이터에서 이동평균이 역배열인지 확인
    fn is_ma_reverse_arrangement(&self, n: usize) -> bool {
        self.is_all(|data| data.is_ma_reverse_arrangement(), n)
    }

    /// 데드 크로스 패턴 확인 (역배열이 n개 연속, 이전 m개는 아님)
    fn is_ma_reverse_arrangement_golden_cross(&self, n: usize, m: usize) -> bool {
        self.is_break_through_by_satisfying(|data| data.is_ma_reverse_arrangement(), n, m)
    }

    /// 특정 인덱스의 이동평균 값 반환
    fn get_ma(&self, index: usize) -> f64 {
        self.get(0, |data| data.mas.get_from_index(index).get())
    }

    /// 단기와 장기 이동평균의 교차 여부 확인
    fn is_ma_crossed(&self, short_index: usize, long_index: usize) -> bool {
        if self.items.len() < 2 {
            return false;
        }

        let current = &self.items[0];
        let previous = &self.items[1];

        let current_short = current.mas.get_from_index(short_index).get();
        let current_long = current.mas.get_from_index(long_index).get();
        let previous_short = previous.mas.get_from_index(short_index).get();
        let previous_long = previous.mas.get_from_index(long_index).get();

        (current_short > current_long) != (previous_short > previous_long)
    }
}

impl<C: Candle> StrategyContextOps<StrategyData<C>, C> for StrategyContext<C> {
    fn next_data(&mut self, candle: C) -> StrategyData<C> {
        let mas = self.masbuilder.next(&candle);
        StrategyData::new(candle, mas)
    }

    fn datum(&self) -> &Vec<StrategyData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<StrategyData<C>> {
        &mut self.items
    }
}

/// 이동평균 기반 트레이딩 전략
#[derive(Debug)]
pub struct MAStrategy<C: Candle> {
    /// 전략 설정
    config: MAStrategyConfig,
    /// 전략 컨텍스트 (데이터 보관 및 연산)
    ctx: StrategyContext<C>,
}

impl<C: Candle> Display for MAStrategy<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let periods = self
            .config
            .ma_periods
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");

        write!(
            f,
            "[MA전략] 설정: {{타입: {:?}, 기간: [{}], 크로스확인: {}}}, 컨텍스트: {}",
            self.config.ma, periods, self.config.cross_previous_periods, self.ctx
        )
    }
}

impl<C: Candle + 'static> MAStrategy<C> {
    /// 새 MA 전략 인스턴스 생성 (JSON 설정 파일 사용)
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    /// * `json_config` - JSON 형식의 설정 문자열
    ///
    /// # Returns
    /// * `Result<MAStrategy<C>, String>` - 초기화된 MA 전략 인스턴스 또는 오류
    pub fn new(storage: &CandleStore<C>, json_config: &str) -> Result<MAStrategy<C>, String> {
        let config = MAStrategyConfig::from_json(json_config)?;
        info!("MA 전략 설정: {:?}", config);
        let ctx = StrategyContext::new(&config, storage);

        Ok(MAStrategy { config, ctx })
    }

    /// 새 MA 전략 생성 (설정을 직접 제공)
    pub fn new_with_config(
        storage: &CandleStore<C>,
        config: Option<HashMap<String, String>>,
    ) -> Result<MAStrategy<C>, String> {
        let strategy_config = match config {
            Some(cfg) => MAStrategyConfig::from_hash_map(&cfg)?,
            None => MAStrategyConfig::default(),
        };

        info!("MA 전략 설정: {:?}", strategy_config);
        let ctx = StrategyContext::new(&strategy_config, storage);

        Ok(MAStrategy {
            config: strategy_config,
            ctx,
        })
    }
}

impl<C: Candle + 'static> Strategy<C> for MAStrategy<C> {
    fn next(&mut self, candle: C) {
        self.ctx.next(candle)
    }

    fn should_enter(&self, _candle: &C) -> bool {
        // 골든 크로스 발생시 롱 진입 신호
        self.ctx
            .is_ma_regular_arrangement_golden_cross(1, self.config.cross_previous_periods)
    }

    fn should_exit(&self, _holdings: &TradePosition, _candle: &C) -> bool {
        // 단기 이동평균이 장기 이동평균보다 낮아질 때(데드 크로스) 롱 청산
        self.ctx.is_break_through_by_satisfying(
            |data| {
                let short_ma = data.mas.get_from_index(0).get();
                let long_ma = data.mas.get_from_index(2).get();
                short_ma < long_ma
            },
            1,
            self.config.cross_previous_periods,
        )
    }

    fn get_position(&self) -> PositionType {
        PositionType::Long
    }

    fn get_name(&self) -> StrategyType {
        StrategyType::MA
    }
}
