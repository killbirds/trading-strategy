use crate::{candle_store::CandleStore, indicator::TAs};
use std::hash::Hash;
use trading_chart::Candle;

/// 캔들 데이터에 접근하기 위한 트레이트
pub trait GetCandle<C: Candle> {
    /// 캔들 데이터 참조 반환
    fn candle(&self) -> &C;
}

/// 전략 데이터 연산을 위한 트레이트
///
/// 이 트레이트는 전략 데이터에 대한 다양한 연산 메서드를 제공합니다.
/// 기본 구현이 포함되어 있어 구현체에서 추가 구현 없이 사용할 수 있습니다.
pub trait AnalyzerDataOps<C: Candle>: GetCandle<C> {
    /// 수익률 계산
    ///
    /// # Arguments
    /// * `get_value` - 기준 가격 가져오는 함수
    ///
    /// # Returns
    /// * `f64` - 수익률 (현재가격 - 기준가격)/기준가격
    fn get_rate_of_return(&self, get_value: impl Fn(&Self) -> f64) -> f64 {
        let value = get_value(self);
        (self.candle().close_price() - value) / value
    }

    /// 두 값을 비교하여 첫 번째 값이 두 번째 값보다 큰지 확인
    ///
    /// # Arguments
    /// * `get_lv` - 왼쪽 값 추출 함수
    /// * `get_rv` - 오른쪽 값 추출 함수
    ///
    /// # Returns
    /// * `bool` - 왼쪽 값이 오른쪽 값보다 크면 true
    fn is_greater_than(
        &self,
        get_lv: impl Fn(&Self) -> f64,
        get_rv: impl Fn(&Self) -> f64,
    ) -> bool {
        let lv = get_lv(self);
        let rv = get_rv(self);
        lv > rv
    }

    /// 두 값을 비교하여 첫 번째 값이 두 번째 값보다 작은지 확인
    ///
    /// # Arguments
    /// * `get_lv` - 왼쪽 값 추출 함수
    /// * `get_rv` - 오른쪽 값 추출 함수
    ///
    /// # Returns
    /// * `bool` - 왼쪽 값이 오른쪽 값보다 작으면 true
    fn is_less_than(&self, get_lv: impl Fn(&Self) -> f64, get_rv: impl Fn(&Self) -> f64) -> bool {
        let lv = get_lv(self);
        let rv = get_rv(self);
        lv < rv
    }

    /// 기술적 지표가 정규 배열(오름차순)인지 확인
    ///
    /// # Arguments
    /// * `get` - 기술적 지표 컬렉션 가져오는 함수
    /// * `get_value` - 개별 지표에서 값 추출 함수
    ///
    /// # Returns
    /// * `bool` - 지표가 오름차순으로 정렬되어 있으면 true
    fn is_regular_arrangement<K, T>(
        &self,
        get: impl Fn(&Self) -> &TAs<K, T>,
        get_value: impl Fn(&T) -> f64,
    ) -> bool
    where
        K: PartialEq + Eq + Hash + std::fmt::Debug,
    {
        let tas = get(self);
        tas.is_regular_arrangement(&get_value)
    }

    /// 기술적 지표가 역배열(내림차순)인지 확인
    ///
    /// # Arguments
    /// * `get` - 기술적 지표 컬렉션 가져오는 함수
    /// * `get_value` - 개별 지표에서 값 추출 함수
    ///
    /// # Returns
    /// * `bool` - 지표가 내림차순으로 정렬되어 있으면 true
    fn is_reverse_arrangement<K, T>(
        &self,
        get: impl Fn(&Self) -> &TAs<K, T>,
        get_value: impl Fn(&T) -> f64,
    ) -> bool
    where
        K: PartialEq + Eq + Hash + std::fmt::Debug,
    {
        let tas = get(self);
        tas.is_reverse_arrangement(&get_value)
    }
}

/// 전략 컨텍스트 연산을 위한 트레이트
///
/// 이 트레이트는 전략 데이터의 시계열 컬렉션에 대한 연산을 제공합니다.
pub trait AnalyzerOps<Data: AnalyzerDataOps<C>, C: Candle> {
    /// 새로운 캔들로부터 전략 데이터 생성
    ///
    /// # Arguments
    /// * `candle` - 새로운 캔들 데이터
    ///
    /// # Returns
    /// * `Data` - 생성된 전략 데이터
    fn next_data(&mut self, candle: C) -> Data;

    /// 컨텍스트에 새 캔들 데이터 추가
    ///
    /// 최대 20개의 데이터를 유지하며, 가장 최신 데이터가 인덱스 0에 위치합니다.
    ///
    /// # Arguments
    /// * `candle` - 새로운 캔들 데이터
    fn next(&mut self, candle: C) {
        let next_data = self.next_data(candle);
        let datum = self.datum_mut();

        if datum.len() < 20 {
            datum.insert(0, next_data);
        } else {
            datum.rotate_right(1);
            if let Some(first) = datum.first_mut() {
                *first = next_data;
            }
        }
    }

    /// 전략 데이터 컬렉션 참조 반환
    fn datum(&self) -> &Vec<Data>;

    /// 전략 데이터 컬렉션 가변 참조 반환
    fn datum_mut(&mut self) -> &mut Vec<Data>;

    fn get(&self, index: usize) -> Option<&Data> {
        self.datum().get(index)
    }

    /// 초기 캔들 데이터로 컨텍스트 초기화
    ///
    /// # Arguments
    /// * `items` - 초기화할 캔들 데이터 컬렉션
    fn init(&mut self, items: Vec<C>) {
        items.into_iter().for_each(|item| self.next(item));
    }

    fn init_from_storage(&mut self, storage: &CandleStore<C>) {
        self.init(storage.get_time_ordered_items())
    }

    /// 특정 인덱스의 데이터에서 값 추출
    ///
    /// # Arguments
    /// * `index` - 데이터 인덱스
    /// * `get_value` - 값 추출 함수
    ///
    /// # Returns
    /// * `f64` - 추출된 값
    ///
    /// # Panics
    /// * 인덱스가 범위를 벗어나면 패닉 발생
    fn get_value(&self, index: usize, get_value: impl Fn(&Data) -> f64) -> f64 {
        self.get(index).map(get_value).unwrap()
    }

    /// 특정 인덱스 데이터의 수익률 계산
    ///
    /// # Arguments
    /// * `index` - 데이터 인덱스
    /// * `get_value` - 기준 가격 추출 함수
    ///
    /// # Returns
    /// * `f64` - 계산된 수익률
    ///
    /// # Panics
    /// * 인덱스가 범위를 벗어나면 패닉 발생
    fn get_rate_of_return(&self, index: usize, get_value: impl Fn(&Data) -> f64) -> f64 {
        self.datum()
            .get(index)
            .map(|data| data.get_rate_of_return(&get_value))
            .unwrap()
    }

    /// n개의 연속된 데이터가 특정 조건을 모두 만족하는지 확인
    ///
    /// # Arguments
    /// * `is_fn` - 확인할 조건 함수
    /// * `n` - 확인할 데이터 개수
    /// * `p` - 최신 데이터에서 drop할 개수
    ///
    /// # Returns
    /// * `bool` - 모든 데이터가 조건을 만족하면 true
    fn is_all(&self, is_fn: impl Fn(&Data) -> bool, n: usize, p: usize) -> bool {
        let data = self.datum();
        if data.len() < n + p {
            false
        } else {
            data.iter().skip(p).take(n).all(is_fn)
        }
    }

    /// 먼저 n개 데이터는 조건을 만족하고, 이전 m개 데이터는 만족하지 않는 돌파 패턴 확인
    ///
    /// # Arguments
    /// * `is_fn` - 확인할 조건 함수
    /// * `n` - 조건을 만족해야 하는 최근 데이터 개수
    /// * `m` - 조건을 만족하지 않아야 하는 이전 데이터 개수
    /// * `p` - 최신 데이터에서 drop할 개수
    ///
    /// # Returns
    /// * `bool` - 돌파 패턴이 확인되면 true
    fn is_break_through_by_satisfying(
        &self,
        is_fn: impl Fn(&Data) -> bool + Copy,
        n: usize,
        m: usize,
        p: usize,
    ) -> bool {
        if self.datum().len() < n + m + p {
            false
        } else {
            let data = self.datum();
            let (heads, tails) = data.split_at(n + p);
            let heads = &heads[p..]; // p만큼 drop
            let result = heads.iter().all(is_fn);
            result && tails.iter().take(m).all(|data| !is_fn(data))
        }
    }

    /// n개의 연속 데이터에서 기술적 지표가 정규 배열인지 확인
    ///
    /// # Arguments
    /// * `get` - 기술적 지표 컬렉션 가져오는 함수
    /// * `get_value` - 개별 지표에서 값 추출 함수
    /// * `n` - 확인할 데이터 개수
    /// * `p` - 최신 데이터에서 drop할 개수
    ///
    /// # Returns
    /// * `bool` - 모든 데이터가 조건을 만족하면 true
    fn is_regular_arrangement<K, T>(
        &self,
        get: impl Fn(&Data) -> &TAs<K, T>,
        get_value: impl Fn(&T) -> f64,
        n: usize,
        p: usize,
    ) -> bool
    where
        K: PartialEq + Eq + Hash + std::fmt::Debug,
    {
        self.is_all(
            |data| {
                let tas = get(data);
                tas.is_regular_arrangement(&get_value)
            },
            n,
            p,
        )
    }

    /// n개의 연속 데이터에서 기술적 지표가 역배열인지 확인
    ///
    /// # Arguments
    /// * `get` - 기술적 지표 컬렉션 가져오는 함수
    /// * `get_value` - 개별 지표에서 값 추출 함수
    /// * `n` - 확인할 데이터 개수
    /// * `p` - 최신 데이터에서 drop할 개수
    ///
    /// # Returns
    /// * `bool` - 모든 데이터가 조건을 만족하면 true
    fn is_reverse_arrangement<K, T>(
        &self,
        get: impl Fn(&Data) -> &TAs<K, T>,
        get_value: impl Fn(&T) -> f64,
        n: usize,
        p: usize,
    ) -> bool
    where
        K: PartialEq + Eq + Hash + std::fmt::Debug,
    {
        self.is_all(
            |data| data.is_reverse_arrangement::<K, T>(&get, &get_value),
            n,
            p,
        )
    }

    /// 최근 n개 캔들 중에서 매수 시그널이 있는지 확인
    ///
    /// # Arguments
    /// * `signal_fn` - 각 데이터에서 매수 시그널이 있는지 확인하는 함수
    /// * `n` - 검사할 캔들 수
    /// * `p` - 최신 데이터에서 drop할 개수
    /// * `threshold` - 신호 감지를 위한 임계값 (0.0 ~ 1.0)
    ///
    /// # Returns
    /// * `Option<usize>` - 매수 시그널이 있는 캔들의 인덱스 (없으면 None)
    fn detect_buy_signal(
        &self,
        signal_fn: impl Fn(&Data) -> f64,
        n: usize,
        p: usize,
        threshold: f64,
    ) -> Option<usize> {
        let data = self.datum();
        if data.len() < n + p {
            return None;
        }

        (p..p + n.min(data.len() - p)).find(|&i| signal_fn(&data[i]) >= threshold)
    }

    /// 최근 n개 캔들 중에서 매도 시그널이 있는지 확인
    ///
    /// # Arguments
    /// * `signal_fn` - 각 데이터에서 매도 시그널이 있는지 확인하는 함수
    /// * `n` - 검사할 캔들 수
    /// * `p` - 최신 데이터에서 drop할 개수 (기본값: 0)
    /// * `threshold` - 신호 감지를 위한 임계값 (0.0 ~ 1.0)
    ///
    /// # Returns
    /// * `Option<usize>` - 매도 시그널이 있는 캔들의 인덱스 (없으면 None)
    fn detect_sell_signal(
        &self,
        signal_fn: impl Fn(&Data) -> f64,
        n: usize,
        p: usize,
        threshold: f64,
    ) -> Option<usize> {
        let data = self.datum();
        if data.len() < n + p {
            return None;
        }

        (p..p + n.min(data.len() - p)).find(|&i| signal_fn(&data[i]) >= threshold)
    }

    /// 특정 패턴의 시그널을 감지 (범용 패턴 감지)
    ///
    /// # Arguments
    /// * `conditions` - 조건 함수들의 벡터 (각 함수는 특정 조건이 만족하는지 확인)
    /// * `n` - 검사할 캔들 수
    /// * `p` - 최신 데이터에서 drop할 개수 (기본값: 0)
    ///
    /// # Returns
    /// * `bool` - 모든 조건이 충족되면 true
    fn detect_pattern(&self, conditions: Vec<impl Fn(&Data) -> bool>, n: usize, p: usize) -> bool {
        let data = self.datum();
        if data.len() < n + p {
            return false;
        }

        conditions
            .iter()
            .all(|cond| (p..p + n.min(data.len() - p)).any(|i| cond(&data[i])))
    }

    /// 지정된 기간 동안 거래량이 급증했는지 확인
    ///
    /// # Arguments
    /// * `n` - 검사할 캔들 수
    /// * `p` - 최신 데이터에서 drop할 개수 (기본값: 0)
    /// * `threshold` - 평균 대비 거래량 증가 비율 (예: 2.0은 평균의 2배)
    ///
    /// # Returns
    /// * `bool` - 거래량 급증이 감지되면 true
    fn is_volume_spike(&self, n: usize, p: usize, threshold: f64) -> bool {
        let data = self.datum();
        if data.len() <= n + p {
            return false;
        }

        // 최근 n개를 제외한 캔들들의 평균 거래량 계산 (p만큼 drop 후)
        let avg_volume: f64 = data
            .iter()
            .skip(n + p)
            .map(|d| d.candle().volume())
            .sum::<f64>()
            / (data.len() - n - p) as f64;

        // 최근 n개 캔들 중 하나라도 평균의 threshold배 이상인지 확인 (p만큼 drop 후)
        data.iter()
            .skip(p)
            .take(n)
            .any(|d| d.candle().volume() > avg_volume * threshold)
    }
}
