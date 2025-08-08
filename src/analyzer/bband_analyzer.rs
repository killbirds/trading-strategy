use crate::analyzer::base::{AnalyzerDataOps, AnalyzerOps, GetCandle};
use crate::candle_store::CandleStore;
use crate::indicator::bband::{BollingerBands, BollingerBandsBuilder};
use std::fmt::Display;
use trading_chart::Candle;

/// 볼린저 밴드 분석기 데이터
#[derive(Debug)]
pub struct BBandAnalyzerData<C: Candle> {
    /// 현재 캔들 데이터
    pub candle: C,
    /// 볼린저 밴드
    pub bband: BollingerBands,
}

impl<C: Candle> BBandAnalyzerData<C> {
    /// 새 분석기 데이터 생성
    pub fn new(candle: C, bband: BollingerBands) -> BBandAnalyzerData<C> {
        BBandAnalyzerData { candle, bband }
    }
}

impl<C: Candle> GetCandle<C> for BBandAnalyzerData<C> {
    fn candle(&self) -> &C {
        &self.candle
    }
}

impl<C: Candle> AnalyzerDataOps<C> for BBandAnalyzerData<C> {}

/// 볼린저 밴드 분석기 컨텍스트
#[derive(Debug)]
pub struct BBandAnalyzer<C: Candle> {
    /// 볼린저 밴드 빌더
    pub bbandbuilder: BollingerBandsBuilder<C>,
    /// 분석기 데이터 히스토리 (최신 데이터가 인덱스 0)
    pub items: Vec<BBandAnalyzerData<C>>,
}

impl<C: Candle> Display for BBandAnalyzer<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.items.first() {
            Some(first) => write!(
                f,
                "캔들: {}, 밴드: {{상: {:.2}, 중: {:.2}, 하: {:.2}}}",
                first.candle,
                first.bband.upper(),
                first.bband.middle(),
                first.bband.lower()
            ),
            None => write!(f, "데이터 없음"),
        }
    }
}

impl<C: Candle + 'static> BBandAnalyzer<C> {
    /// 새 분석기 컨텍스트 생성
    pub fn new(period: usize, multiplier: f64, storage: &CandleStore<C>) -> BBandAnalyzer<C> {
        let bbandbuilder = BollingerBandsBuilder::new(period, multiplier);
        let mut ctx = BBandAnalyzer {
            bbandbuilder,
            items: vec![],
        };

        ctx.init_from_storage(storage);
        ctx
    }

    pub fn get_bband(&self) -> (f64, f64, f64) {
        match self.items.first() {
            Some(data) => (data.bband.lower(), data.bband.middle(), data.bband.upper()),
            None => (0.0, 0.0, 0.0),
        }
    }

    /// 가격이 볼린저 밴드 하한선 아래로 내려갔는지 확인
    pub fn is_below_lower_band(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.candle.close_price() < data.bband.lower(), n, p)
    }

    /// n개의 연속된 캔들이 하단 밴드 위에 있는지 확인
    pub fn is_above_lower_band(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.candle.close_price() > data.bband.lower(), n, p)
    }

    /// 가격이 볼린저 밴드 상한선 위로 올라갔는지 확인
    pub fn is_below_upper_band(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.candle.close_price() < data.bband.upper(), n, p)
    }

    pub fn is_above_upper_band(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.candle.close_price() > data.bband.upper(), n, p)
    }

    /// 가격이 볼린저 밴드 중앙선 위로 올라갔는지 확인
    pub fn is_above_middle_band(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.candle.close_price() > data.bband.middle(), n, p)
    }

    /// 가격이 볼린저 밴드 중앙선 아래로 내려갔는지 확인
    pub fn is_below_middle_band(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.candle.close_price() < data.bband.middle(), n, p)
    }

    /// 하단 밴드 아래에서 위로 돌파한 경우 (상승 반전 신호) 확인
    pub fn is_break_through_lower_band_from_below(&self, n: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.candle.low_price() > data.bband.lower(),
            1,
            n,
            p,
        )
    }

    /// 상단 밴드 아래에서 위로 돌파한 경우 확인
    pub fn is_break_through_upper_band_from_below(&self, n: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.candle.high_price() > data.bband.upper(),
            1,
            n,
            p,
        )
    }

    /// 밴드 폭이 충분히 넓은지 확인
    pub fn is_band_width_sufficient(&self, p: usize) -> bool {
        self.is_all(
            |data| {
                let band_width = (data.bband.upper() - data.bband.lower()) / data.bband.middle();
                band_width > 0.02
            },
            1,
            p,
        )
    }

    /// 볼린저 밴드 폭이 좁아지는지 확인
    ///
    /// # Arguments
    /// * `n` - 확인할 캔들 수
    ///
    /// # Returns
    /// * `bool` - 밴드 폭이 연속적으로 감소하면 true
    pub fn is_band_width_narrowing(&self, n: usize) -> bool {
        if self.items.len() < n + 1 {
            return false;
        }

        // 최근 n개 캔들의 밴드 폭이 이전 대비 감소하는지 확인
        for i in 0..n {
            let current_width = self.items[i].bband.upper() - self.items[i].bband.lower();
            let previous_width = self.items[i + 1].bband.upper() - self.items[i + 1].bband.lower();

            if current_width >= previous_width {
                return false;
            }
        }
        true
    }

    /// 고가가 볼린저 밴드 상단을 돌파하는지 확인
    ///
    /// # Arguments
    /// * `n` - 확인할 캔들 수
    /// * `p` - 최신 데이터에서 drop할 개수
    ///
    /// # Returns
    /// * `bool` - 고가가 상단을 돌파하면 true
    pub fn is_high_break_through_upper_band(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.candle.high_price() > data.bband.upper(), n, p)
    }

    /// 종가가 볼린저 밴드 상단 위에 있는지 확인
    ///
    /// # Arguments
    /// * `n` - 확인할 캔들 수
    /// * `p` - 최신 데이터에서 drop할 개수
    ///
    /// # Returns
    /// * `bool` - 종가가 상단 위에 있으면 true
    pub fn is_close_above_upper_band(&self, n: usize, p: usize) -> bool {
        self.is_all(|data| data.candle.close_price() > data.bband.upper(), n, p)
    }

    /// 볼린저 밴드 스퀴즈 돌파 패턴 확인
    ///
    /// 조건:
    /// 1. 이전 n개 캔들에서 밴드 폭이 좁아지다가
    /// 2. 현재 캔들의 고가가 상단을 돌파하고
    /// 3. 현재 캔들의 종가가 상단 위에 위치
    ///
    /// # Arguments
    /// * `narrowing_period` - 밴드 폭 감소를 확인할 기간
    ///
    /// # Returns
    /// * `bool` - 모든 조건이 만족되면 true
    pub fn is_squeeze_breakout_with_close_above_upper(&self, narrowing_period: usize) -> bool {
        if self.items.is_empty() {
            return false;
        }

        // 현재 캔들의 고가가 상단을 돌파하고 종가가 상단 위에 있는지 확인
        let current_data = &self.items[0];
        let high_breaks_upper = current_data.candle.high_price() > current_data.bband.upper();
        let close_above_upper = current_data.candle.close_price() > current_data.bband.upper();

        // 이전 캔들들에서 밴드 폭이 좁아지는지 확인
        let band_narrowing = if self.items.len() > narrowing_period {
            self.is_band_width_narrowing_from_index(1, narrowing_period)
        } else {
            false
        };

        high_breaks_upper && close_above_upper && band_narrowing
    }

    /// 특정 인덱스부터 밴드 폭이 좁아지는지 확인하는 헬퍼 메서드
    ///
    /// # Arguments
    /// * `start_index` - 확인 시작 인덱스
    /// * `n` - 확인할 캔들 수
    ///
    /// # Returns
    /// * `bool` - 밴드 폭이 연속적으로 감소하면 true
    fn is_band_width_narrowing_from_index(&self, start_index: usize, n: usize) -> bool {
        if self.items.len() < start_index + n + 1 {
            return false;
        }

        for i in 0..n {
            let current_idx = start_index + i;
            let previous_idx = start_index + i + 1;

            let current_width =
                self.items[current_idx].bband.upper() - self.items[current_idx].bband.lower();
            let previous_width =
                self.items[previous_idx].bband.upper() - self.items[previous_idx].bband.lower();

            if current_width >= previous_width {
                return false;
            }
        }
        true
    }

    /// 현재 밴드 폭 반환
    ///
    /// # Returns
    /// * `f64` - 현재 밴드 폭 (상단 - 하단)
    pub fn get_current_band_width(&self) -> f64 {
        if let Some(data) = self.items.first() {
            data.bband.upper() - data.bband.lower()
        } else {
            0.0
        }
    }

    /// 밴드 폭의 변화율 반환
    ///
    /// # Returns
    /// * `f64` - 밴드 폭 변화율 (현재 - 이전) / 이전 * 100
    pub fn get_band_width_change_rate(&self) -> f64 {
        if self.items.len() < 2 {
            return 0.0;
        }

        let current_width = self.items[0].bband.upper() - self.items[0].bband.lower();
        let previous_width = self.items[1].bband.upper() - self.items[1].bband.lower();

        if previous_width == 0.0 {
            return 0.0;
        }

        ((current_width - previous_width) / previous_width) * 100.0
    }

    /// 볼린저 밴드 폭이 좁은 상태인지 확인 (스퀴즈 상태)
    ///
    /// # Arguments
    /// * `n` - 확인할 캔들 수
    /// * `threshold` - 좁은 상태 판정 임계값 (밴드 폭 / 중간값 비율)
    /// * `p` - 최신 데이터에서 drop할 개수
    ///
    /// # Returns
    /// * `bool` - 밴드 폭이 임계값 이하로 좁으면 true
    pub fn is_band_width_squeeze(&self, n: usize, threshold: f64, p: usize) -> bool {
        self.is_all(
            |data| {
                let band_width = data.bband.upper() - data.bband.lower();
                let middle = data.bband.middle();
                if middle == 0.0 {
                    return false;
                }
                let width_ratio = band_width / middle;
                width_ratio <= threshold
            },
            n,
            p,
        )
    }

    /// 현재 밴드 폭 비율 반환 (밴드 폭 / 중간값)
    ///
    /// # Returns
    /// * `f64` - 밴드 폭 비율
    pub fn get_band_width_ratio(&self) -> f64 {
        if let Some(data) = self.items.first() {
            let band_width = data.bband.upper() - data.bband.lower();
            let middle = data.bband.middle();
            if middle == 0.0 {
                return 0.0;
            }
            band_width / middle
        } else {
            0.0
        }
    }

    /// 밴드 폭이 좁아지다가 좁은 상태를 유지하는 패턴 확인
    ///
    /// # Arguments
    /// * `narrowing_period` - 밴드 폭 감소 확인 기간
    /// * `squeeze_period` - 좁은 상태 유지 기간
    /// * `threshold` - 좁은 상태 판정 임계값
    ///
    /// # Returns
    /// * `bool` - 패턴이 확인되면 true
    pub fn is_narrowing_then_squeeze_pattern(
        &self,
        narrowing_period: usize,
        squeeze_period: usize,
        threshold: f64,
    ) -> bool {
        if self.items.len() < narrowing_period + squeeze_period + 1 {
            return false;
        }

        // 최근 squeeze_period 동안 좁은 상태 유지
        let recent_squeeze = self.is_band_width_squeeze(squeeze_period, threshold, 0);

        // 그 이전에 narrowing_period 동안 밴드 폭 감소
        let previous_narrowing =
            self.is_band_width_narrowing_from_index(squeeze_period, narrowing_period);

        recent_squeeze && previous_narrowing
    }

    /// 향상된 스퀴즈 돌파 패턴 확인 (좁아지다가 좁은 상태 유지 후 돌파)
    ///
    /// 조건:
    /// 1. 이전에 밴드 폭이 좁아지다가
    /// 2. 좁은 상태를 유지하다가
    /// 3. 현재 캔들의 고가가 상단을 돌파하고
    /// 4. 현재 캔들의 종가가 상단 위에 위치
    ///
    /// # Arguments
    /// * `narrowing_period` - 밴드 폭 감소 확인 기간
    /// * `squeeze_period` - 좁은 상태 유지 기간
    /// * `threshold` - 좁은 상태 판정 임계값
    ///
    /// # Returns
    /// * `bool` - 모든 조건이 만족되면 true
    pub fn is_enhanced_squeeze_breakout_with_close_above_upper(
        &self,
        narrowing_period: usize,
        squeeze_period: usize,
        threshold: f64,
    ) -> bool {
        if self.items.is_empty() {
            return false;
        }

        // 현재 캔들의 고가가 상단을 돌파하고 종가가 상단 위에 있는지 확인
        let current_data = &self.items[0];
        let high_breaks_upper = current_data.candle.high_price() > current_data.bband.upper();
        let close_above_upper = current_data.candle.close_price() > current_data.bband.upper();

        // 좁아지다가 좁은 상태를 유지하는 패턴 확인
        let narrowing_squeeze_pattern =
            self.is_narrowing_then_squeeze_pattern(narrowing_period, squeeze_period, threshold);

        high_breaks_upper && close_above_upper && narrowing_squeeze_pattern
    }

    /// 스퀴즈 상태에서 밴드 폭 확대 시작 확인
    ///
    /// # Arguments
    /// * `threshold` - 좁은 상태 판정 임계값
    ///
    /// # Returns
    /// * `bool` - 스퀴즈 상태에서 확대가 시작되면 true
    pub fn is_squeeze_expansion_start(&self, threshold: f64) -> bool {
        if self.items.len() < 2 {
            return false;
        }

        // 현재는 스퀴즈 상태가 아니고
        let current_not_squeeze = !self.is_band_width_squeeze(1, threshold, 0);

        // 이전에는 스퀴즈 상태였는지 확인
        let previous_was_squeeze = if self.items.len() >= 2 {
            let previous_data = &self.items[1];
            let band_width = previous_data.bband.upper() - previous_data.bband.lower();
            let middle = previous_data.bband.middle();
            if middle == 0.0 {
                return false;
            }
            let width_ratio = band_width / middle;
            width_ratio <= threshold
        } else {
            false
        };

        current_not_squeeze && previous_was_squeeze
    }

    /// 하단 밴드 하향 돌파 신호 확인 (n개 연속 하단 밴드 아래, 이전 m개는 아님)
    pub fn is_below_lower_band_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.candle.close_price() < data.bband.lower(),
            n,
            m,
            p,
        )
    }

    /// 상단 밴드 상향 돌파 신호 확인 (n개 연속 상단 밴드 위, 이전 m개는 아님)
    pub fn is_above_upper_band_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.candle.close_price() > data.bband.upper(),
            n,
            m,
            p,
        )
    }

    /// 중간선 상향 돌파 신호 확인 (n개 연속 중간선 위, 이전 m개는 아님)
    pub fn is_above_middle_band_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.candle.close_price() > data.bband.middle(),
            n,
            m,
            p,
        )
    }

    /// 중간선 하향 돌파 신호 확인 (n개 연속 중간선 아래, 이전 m개는 아님)
    pub fn is_below_middle_band_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.candle.close_price() < data.bband.middle(),
            n,
            m,
            p,
        )
    }

    /// 스퀴즈 상태 돌파 신호 확인 (n개 연속 스퀴즈 상태, 이전 m개는 아님)
    pub fn is_squeeze_state_signal(&self, n: usize, m: usize, threshold: f64, p: usize) -> bool {
        self.is_break_through_by_satisfying(
            |data| {
                let band_width = data.bband.upper() - data.bband.lower();
                let middle = data.bband.middle();
                if middle == 0.0 {
                    return false;
                }
                let width_ratio = band_width / middle;
                width_ratio <= threshold
            },
            n,
            m,
            p,
        )
    }

    /// 밴드폭 확장 신호 확인 (n개 연속 밴드폭 확장, 이전 m개는 아님)
    pub fn is_band_expansion_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(
            |_data| {
                if self.items.len() < 2 {
                    return false;
                }
                // 현재 밴드폭이 이전보다 큰지 확인
                let current_width = self.items[0].bband.upper() - self.items[0].bband.lower();
                let previous_width = self.items[1].bband.upper() - self.items[1].bband.lower();
                current_width > previous_width
            },
            n,
            m,
            p,
        )
    }

    /// 상단 밴드 고가 돌파 신호 확인 (n개 연속 고가가 상단 밴드 돌파, 이전 m개는 아님)
    pub fn is_high_breaks_upper_band_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.candle.high_price() > data.bband.upper(),
            n,
            m,
            p,
        )
    }

    /// 종가가 상단 밴드 위 신호 확인 (n개 연속 종가가 상단 밴드 위, 이전 m개는 아님)
    pub fn is_close_above_upper_band_signal(&self, n: usize, m: usize, p: usize) -> bool {
        self.is_break_through_by_satisfying(
            |data| data.candle.close_price() > data.bband.upper(),
            n,
            m,
            p,
        )
    }

    /// 스퀴즈 브레이크아웃 상단 돌파 신호 확인 (n개 연속 상단 돌파, 이전 m개는 아님)
    pub fn is_squeeze_breakout_upper_signal(
        &self,
        n: usize,
        m: usize,
        narrowing_period: usize,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |_| {
                // 스퀴즈 브레이크아웃 로직 (기존 is_squeeze_breakout_with_close_above_upper와 유사)
                if self.items.is_empty() {
                    return false;
                }
                let current_data = &self.items[0];
                let high_breaks_upper =
                    current_data.candle.high_price() > current_data.bband.upper();
                let close_above_upper =
                    current_data.candle.close_price() > current_data.bband.upper();
                let narrowing = self.is_band_width_narrowing(narrowing_period);
                high_breaks_upper && close_above_upper && narrowing
            },
            n,
            m,
            p,
        )
    }

    /// 밴드폭 임계값 돌파 신호 확인 (n개 연속 밴드폭 임계값 초과, 이전 m개는 아님)
    pub fn is_band_width_threshold_breakthrough(
        &self,
        n: usize,
        m: usize,
        threshold: f64,
        p: usize,
    ) -> bool {
        self.is_break_through_by_satisfying(
            |data| {
                let band_width = data.bband.upper() - data.bband.lower();
                let middle = data.bband.middle();
                if middle == 0.0 {
                    return false;
                }
                let width_ratio = band_width / middle;
                width_ratio > threshold
            },
            n,
            m,
            p,
        )
    }

    /// n개의 연속 데이터에서 중간 밴드가 횡보 상태인지 확인
    pub fn is_middle_band_sideways(&self, n: usize, p: usize, threshold: f64) -> bool {
        self.is_sideways(
            |data: &BBandAnalyzerData<C>| data.bband.middle(),
            n,
            p,
            threshold,
        )
    }

    /// n개의 연속 데이터에서 상단 밴드가 횡보 상태인지 확인
    pub fn is_upper_band_sideways(&self, n: usize, p: usize, threshold: f64) -> bool {
        self.is_sideways(
            |data: &BBandAnalyzerData<C>| data.bband.upper(),
            n,
            p,
            threshold,
        )
    }

    /// n개의 연속 데이터에서 하단 밴드가 횡보 상태인지 확인
    pub fn is_lower_band_sideways(&self, n: usize, p: usize, threshold: f64) -> bool {
        self.is_sideways(
            |data: &BBandAnalyzerData<C>| data.bband.lower(),
            n,
            p,
            threshold,
        )
    }

    /// n개의 연속 데이터에서 밴드폭이 횡보 상태인지 확인
    pub fn is_band_width_sideways(&self, n: usize, p: usize, threshold: f64) -> bool {
        self.is_sideways(
            |data: &BBandAnalyzerData<C>| data.bband.upper() - data.bband.lower(),
            n,
            p,
            threshold,
        )
    }
}

impl<C: Candle> AnalyzerOps<BBandAnalyzerData<C>, C> for BBandAnalyzer<C> {
    fn next_data(&mut self, candle: C) -> BBandAnalyzerData<C> {
        let bband = self.bbandbuilder.next(&candle);
        BBandAnalyzerData::new(candle, bband)
    }

    fn datum(&self) -> &Vec<BBandAnalyzerData<C>> {
        &self.items
    }

    fn datum_mut(&mut self) -> &mut Vec<BBandAnalyzerData<C>> {
        &mut self.items
    }
}
