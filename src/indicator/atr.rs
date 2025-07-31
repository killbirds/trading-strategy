use std::collections::HashMap;
use std::marker::PhantomData;
use trading_chart::Candle;

/// ATR 단일 값
#[derive(Debug, Clone, Copy)]
pub struct ATR {
    /// ATR 값
    pub value: f64,
}

impl ATR {
    /// 새 ATR 값 생성
    pub fn new(value: f64) -> ATR {
        ATR { value }
    }
}

/// ATR 값 모음
#[derive(Debug, Clone)]
pub struct ATRs {
    /// 기간별 ATR 값
    values: HashMap<usize, ATR>,
}

impl Default for ATRs {
    fn default() -> Self {
        Self::new()
    }
}

impl ATRs {
    /// 새 ATR 값 모음 생성
    pub fn new() -> ATRs {
        ATRs {
            values: HashMap::new(),
        }
    }

    /// ATR 값 추가
    pub fn add(&mut self, period: usize, value: f64) {
        self.values.insert(period, ATR::new(value));
    }

    /// 특정 기간의 ATR 값 반환
    pub fn get(&self, period: &usize) -> ATR {
        match self.values.get(period) {
            Some(value) => *value,
            None => ATR::new(0.0),
        }
    }

    /// 모든 ATR 값 반환
    pub fn get_all(&self) -> Vec<ATR> {
        let mut result = Vec::new();
        for value in self.values.values() {
            result.push(*value);
        }
        result
    }
}

/// ATR 계산을 위한 빌더
#[derive(Debug)]
pub struct ATRBuilder<C: Candle> {
    /// ATR 계산 기간
    period: usize,
    /// 고가 데이터
    high_values: Vec<f64>,
    /// 저가 데이터
    low_values: Vec<f64>,
    /// 종가 데이터
    close_values: Vec<f64>,
    /// 이전 ATR 값
    previous_atr: Option<f64>,
    /// 캔들 타입 표시자 (제네릭 타입 표시용)
    _phantom: PhantomData<C>,
}

impl<C: Candle> ATRBuilder<C> {
    /// 새 ATR 빌더 생성
    pub fn new(period: usize) -> ATRBuilder<C> {
        ATRBuilder {
            period,
            high_values: Vec::new(),
            low_values: Vec::new(),
            close_values: Vec::new(),
            previous_atr: None,
            _phantom: PhantomData,
        }
    }

    /// 다음 캔들 데이터로 ATR 계산
    pub fn next(&mut self, candle: &C) -> f64 {
        // 가격 데이터 저장
        self.high_values.push(candle.high_price());
        self.low_values.push(candle.low_price());
        self.close_values.push(candle.close_price());

        // 필요한 데이터만 유지
        if self.high_values.len() > self.period * 2 {
            self.high_values.remove(0);
            self.low_values.remove(0);
            self.close_values.remove(0);
        }

        // 충분한 데이터가 없는 경우
        if self.high_values.len() < 2 {
            return 0.0;
        }

        // True Range 계산
        let mut tr_values = Vec::with_capacity(self.high_values.len() - 1);
        for i in 1..self.high_values.len() {
            let high = self.high_values[i];
            let low = self.low_values[i];
            let prev_close = self.close_values[i - 1];

            // True Range = max(고가-저가, |고가-이전종가|, |저가-이전종가|)
            let tr = (high - low)
                .max((high - prev_close).abs())
                .max((low - prev_close).abs());
            tr_values.push(tr);
        }

        // ATR 계산
        let atr = if tr_values.len() >= self.period {
            if let Some(prev_atr) = self.previous_atr {
                // Wilder의 평활화 방식으로 업데이트
                (prev_atr * (self.period as f64 - 1.0) + tr_values.last().unwrap())
                    / self.period as f64
            } else {
                // 처음 계산할 때는 단순 평균 사용
                tr_values.iter().take(self.period).sum::<f64>() / self.period as f64
            }
        } else if !tr_values.is_empty() {
            // 충분한 데이터가 없는 경우 가용 데이터로 평균 계산
            tr_values.iter().sum::<f64>() / tr_values.len() as f64
        } else {
            0.0
        };

        // 계산된 ATR 저장
        self.previous_atr = Some(atr);
        atr
    }
}

/// ATR 빌더 집합
#[derive(Debug)]
pub struct ATRsBuilder<C: Candle> {
    /// 기간별 ATR 빌더
    builders: HashMap<usize, ATRBuilder<C>>,
}

impl<C: Candle> ATRsBuilder<C> {
    /// 새 ATR 빌더 집합 생성
    pub fn new(periods: &[usize]) -> ATRsBuilder<C> {
        let mut builders = HashMap::new();
        for &period in periods {
            builders.insert(period, ATRBuilder::new(period));
        }
        ATRsBuilder { builders }
    }

    /// 다음 캔들 데이터로 모든 ATR 계산
    pub fn next(&mut self, candle: &C) -> ATRs {
        let mut atrs = ATRs::new();
        for (&period, builder) in &mut self.builders {
            let atr = builder.next(candle);
            atrs.add(period, atr);
        }
        atrs
    }
}

/// ATR 빌더 팩토리
pub struct ATRsBuilderFactory;

impl ATRsBuilderFactory {
    /// 새 ATR 빌더 생성
    pub fn build<C: Candle>(periods: &[usize]) -> ATRsBuilder<C> {
        ATRsBuilder::new(periods)
    }
}
