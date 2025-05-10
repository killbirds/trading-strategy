// 기존 ta 모듈에서 이동된 기술적 지표 모듈
// 각종 기술적 분석 지표를 제공합니다.

pub mod adx;
pub mod bband;
pub mod ichimoku;
pub mod ma;
pub mod macd;
pub mod max;
pub mod min;
pub mod rsi;
pub mod utils;
pub mod volume;
pub mod vwap;

// 이 모듈은 이전의 ta 모듈을 대체합니다.
// 더 명확한 이름으로 기술적 지표를 표현합니다.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use crate::candle_store::CandleStore;
use std::fmt::Display;
use trading_chart::Candle;

/// 기술적 지표(TA)의 컬렉션을 관리하는 구조체
///
/// 여러 기술적 지표를 키-값 쌍으로 저장하고 관리합니다.
#[derive(Debug)]
pub struct TAs<K, T>
where
    K: PartialEq + Eq + Hash + std::fmt::Debug,
{
    /// 이 컬렉션의 이름
    name: String,
    /// 순서가 유지되는 키 목록
    keys: Vec<K>,
    /// 키-값 쌍으로 저장된 기술적 지표
    data: HashMap<K, T>,
}

impl<K, T> Display for TAs<K, T>
where
    K: PartialEq + Eq + Hash + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TAs({})", self.name)
    }
}

/// 값들의 배열 정렬 여부를 확인하는 내부 함수
///
/// # Arguments
/// * `values` - 확인할 값들의 벡터
/// * `init` - 초기 비교값
/// * `cmp` - 비교 함수
///
/// # Returns
/// * `bool` - 정렬 조건 만족 여부
fn is_arrangement(values: Vec<f64>, init: f64, cmp: impl Fn(f64, f64) -> bool) -> bool {
    if values.is_empty() {
        return true;
    }

    let mut prev = init;
    for value in values {
        if !cmp(value, prev) {
            return false;
        }
        prev = value;
    }

    true
}

/// 값들이 내림차순으로 정렬되어 있는지 확인
///
/// # Arguments
/// * `values` - 확인할 값들의 벡터
///
/// # Returns
/// * `bool` - 내림차순 정렬 여부
fn is_regular_arrangement(values: Vec<f64>) -> bool {
    is_arrangement(values, f64::MAX, |current, prev| current < prev)
}

/// 값들이 오름차순으로 정렬되어 있는지 확인
///
/// # Arguments
/// * `values` - 확인할 값들의 벡터
///
/// # Returns
/// * `bool` - 오름차순 정렬 여부
fn is_reverse_arrangement(values: Vec<f64>) -> bool {
    is_arrangement(values, f64::MIN, |current, prev| current > prev)
}

impl<K, T> TAs<K, T>
where
    K: PartialEq + Eq + Hash + std::fmt::Debug,
{
    /// 새로운 TAs 인스턴스 생성
    ///
    /// # Arguments
    /// * `name` - 컬렉션 이름
    /// * `keys` - 키 목록
    /// * `data` - 키-값 데이터
    ///
    /// # Returns
    /// * `TAs<K, T>` - 새 인스턴스
    pub fn new(name: String, keys: Vec<K>, data: HashMap<K, T>) -> TAs<K, T> {
        TAs { name, keys, data }
    }

    /// 키 목록 참조 반환
    ///
    /// # Returns
    /// * `&Vec<K>` - 키 목록 참조
    pub fn get_keys(&self) -> &Vec<K> {
        &self.keys
    }

    /// 지정된 키에 해당하는 값 참조 반환
    ///
    /// # Arguments
    /// * `key` - 검색할 키
    ///
    /// # Returns
    /// * `&T` - 찾은 값 참조
    ///
    /// # Panics
    /// 키가 없는 경우 패닉 발생
    pub fn get(&self, key: &K) -> &T {
        self.data
            .get(key)
            .unwrap_or_else(|| panic!("키가 존재하지 않습니다: {:?}", key))
    }

    /// 인덱스로 키를 찾고, 해당 키로 데이터를 가져옵니다.
    ///
    /// # Arguments
    /// * `index` - 키를 찾을 인덱스
    ///
    /// # Returns
    /// * `&T` - 해당 인덱스의 키로 찾은 데이터 참조
    ///
    /// # Panics
    /// * 인덱스가 범위를 벗어난 경우
    pub fn get_by_key_index(&self, index: usize) -> &T {
        if let Some(key) = &self.keys.get(index) {
            self.get(key)
        } else {
            panic!("인덱스가 범위를 벗어났습니다: {}", index)
        }
    }

    /// 모든 값의 참조 벡터 반환
    ///
    /// # Returns
    /// * `Vec<&T>` - 모든 값 참조 벡터
    pub fn get_all(&self) -> Vec<&T> {
        self.keys
            .iter()
            .map(|key| self.get(key))
            .collect::<Vec<_>>()
    }

    /// 값들이 내림차순으로 정렬되어 있는지 확인
    ///
    /// # Arguments
    /// * `get_value` - 값 추출 함수
    ///
    /// # Returns
    /// * `bool` - 내림차순 정렬 여부
    pub fn is_regular_arrangement(&self, get_value: impl Fn(&T) -> f64) -> bool {
        let all_ta = self.get_all();
        let values = all_ta.iter().map(|ta| get_value(ta)).collect::<Vec<_>>();
        is_regular_arrangement(values)
    }

    /// 값들이 오름차순으로 정렬되어 있는지 확인
    ///
    /// # Arguments
    /// * `get_value` - 값 추출 함수
    ///
    /// # Returns
    /// * `bool` - 오름차순 정렬 여부
    pub fn is_reverse_arrangement(&self, get_value: impl Fn(&T) -> f64) -> bool {
        let all_ta = self.get_all();
        let values = all_ta.iter().map(|ta| get_value(ta)).collect::<Vec<_>>();
        is_reverse_arrangement(values)
    }

    pub fn is_all(&self, is_fn: impl Fn(&T) -> bool) -> bool {
        let all_ta = self.get_all();
        all_ta.into_iter().all(is_fn)
    }
}

/// 기술적 지표 생성 인터페이스
///
/// 기술적 지표를 생성하고 업데이트하기 위한 빌더 패턴 구현
pub trait TABuilder<T, C: Candle>: Send + std::fmt::Debug {
    /// 저장소에서 기술적 지표 생성
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    ///
    /// # Returns
    /// * `T` - 생성된 기술적 지표
    fn from_storage(&mut self, storage: &CandleStore<C>) -> T;

    /// 데이터에서 기술적 지표 생성
    ///
    /// # Arguments
    /// * `data` - 캔들 데이터 벡터
    ///
    /// # Returns
    /// * `T` - 생성된 기술적 지표
    fn build(&mut self, data: &[C]) -> T;

    /// 새 데이터로 기술적 지표 업데이트
    ///
    /// # Arguments
    /// * `data` - 새 캔들 데이터
    ///
    /// # Returns
    /// * `T` - 업데이트된 기술적 지표
    fn next(&mut self, data: &C) -> T;
}

/// 여러 기술적 지표 빌더를 관리하는 구조체
///
/// 여러 기술적 지표를 동시에 생성하고 관리하기 위한 빌더
pub struct TAsBuilder<K, T, C: Candle> {
    /// 이 빌더 컬렉션의 이름
    name: String,
    /// 순서가 유지되는 키 목록
    keys: Vec<K>,
    /// 각 키에 대응하는 개별 빌더
    builders: HashMap<K, Box<dyn TABuilder<T, C>>>,
}

impl<K, T, C> TAsBuilder<K, T, C>
where
    K: PartialEq + Eq + Hash + Clone + std::fmt::Debug,
    C: Candle,
{
    /// 새 TAsBuilder 인스턴스 생성
    ///
    /// # Arguments
    /// * `name` - 빌더 이름
    /// * `keys` - 키 목록
    /// * `gen_builder` - 각 키에 대한 빌더 생성 함수
    ///
    /// # Returns
    /// * `TAsBuilder<K, T>` - 새 인스턴스
    pub fn new(
        name: String,
        keys: &[K],
        gen_builder: impl Fn(&K) -> Box<dyn TABuilder<T, C>>,
    ) -> TAsBuilder<K, T, C> {
        let mut builders: HashMap<K, Box<dyn TABuilder<T, C>>> = HashMap::new();
        for key in keys {
            builders.insert(key.clone(), gen_builder(key));
        }

        TAsBuilder {
            name,
            keys: keys.to_vec(),
            builders,
        }
    }

    /// 저장소에서 기술적 지표 컬렉션 생성
    ///
    /// # Arguments
    /// * `storage` - 캔들 데이터 저장소
    ///
    /// # Returns
    /// * `TAs<K, T>` - 생성된 기술적 지표 컬렉션
    pub fn from_storage(&mut self, storage: &CandleStore<C>) -> TAs<K, T> {
        self.build(&storage.get_time_ordered_items())
    }

    /// 데이터에서 기술적 지표 컬렉션 생성
    ///
    /// # Arguments
    /// * `data` - 캔들 데이터 벡터
    ///
    /// # Returns
    /// * `TAs<K, T>` - 생성된 기술적 지표 컬렉션
    pub fn build(&mut self, data: &[C]) -> TAs<K, T> {
        let mut tas: HashMap<K, T> = HashMap::new();
        for (key, builder) in self.builders.iter_mut() {
            let ta = builder.build(data);
            tas.insert(key.clone(), ta);
        }

        TAs::new(self.name.to_owned(), self.keys.clone(), tas)
    }

    /// 새 데이터로 기술적 지표 컬렉션 업데이트
    ///
    /// # Arguments
    /// * `data` - 새 캔들 데이터
    ///
    /// # Returns
    /// * `TAs<K, T>` - 업데이트된 기술적 지표 컬렉션
    pub fn next(&mut self, data: &C) -> TAs<K, T> {
        let mut tas: HashMap<K, T> = HashMap::new();
        for (key, builder) in self.builders.iter_mut() {
            let ta = builder.next(data);
            tas.insert(key.clone(), ta);
        }
        TAs::new(self.name.to_owned(), self.keys.clone(), tas)
    }
}

impl<K: std::fmt::Debug, T, C> std::fmt::Debug for TAsBuilder<K, T, C>
where
    C: Candle,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TAsBuilder")
            .field("name", &self.name)
            .field("keys", &self.keys)
            .field("builders", &format!("<{} builders>", self.builders.len()))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_regular_arrangement() {
        // 내림차순 정렬 테스트
        assert!(is_regular_arrangement(vec![3.0, 2.0, 1.0]));
        assert!(!is_regular_arrangement(vec![1.0, 2.0, 3.0]));
        assert!(!is_regular_arrangement(vec![3.0, 1.0, 2.0]));
        assert!(!is_regular_arrangement(vec![3.0, 2.0, 2.0]));

        // 빈 벡터는 항상 정렬되어 있음
        assert!(is_regular_arrangement(vec![]));
    }

    #[test]
    fn test_is_reverse_arrangement() {
        // 오름차순 정렬 테스트
        assert!(is_reverse_arrangement(vec![1.0, 2.0, 3.0]));
        assert!(!is_reverse_arrangement(vec![3.0, 2.0, 1.0]));
        assert!(!is_reverse_arrangement(vec![1.0, 3.0, 2.0]));
        assert!(!is_reverse_arrangement(vec![1.0, 2.0, 2.0]));

        // 빈 벡터는 항상 정렬되어 있음
        assert!(is_reverse_arrangement(vec![]));
    }

    #[test]
    fn test_tas_is_regular_arrangement() {
        let tas = TAs::new(
            "test".to_owned(),
            vec![1, 2, 3],
            HashMap::from([(1, 3.0), (2, 2.0), (3, 1.0)]),
        );
        assert!(tas.is_regular_arrangement(|value| *value));

        let tas = TAs::new(
            "test".to_owned(),
            vec![1, 2, 3],
            HashMap::from([(1, 1.0), (2, 2.0), (3, 3.0)]),
        );
        assert!(!tas.is_regular_arrangement(|value| *value));

        let tas = TAs::new(
            "test".to_owned(),
            vec![1, 2, 3],
            HashMap::from([(1, 3.0), (2, 1.0), (3, 2.0)]),
        );
        assert!(!tas.is_regular_arrangement(|value| *value));
    }
}
