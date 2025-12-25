use trading_chart::Candle;

/// 제한된 크기의 데이터 저장소
///
/// 지정된 최대 크기를 유지하며 데이터를 저장하는 구조체입니다.
/// 최대 크기를 초과하면 가장 오래된 데이터가 자동으로 제거됩니다.
/// 데이터는 datetime 기준으로 내림차순 정렬되어 저장됩니다 (최신 데이터가 먼저 옴).
pub struct CandleStore<T: Candle> {
    items: Vec<T>,
    pub max_size: usize,
    pub use_duplicated_filter: bool,
}

impl<T> CandleStore<T>
where
    T: Candle,
{
    /// 새로운 CandleStore 인스턴스를 생성합니다.
    ///
    /// # Arguments
    /// * `items` - 초기 아이템 목록
    /// * `max_size` - 저장소의 최대 크기
    /// * `use_duplicated_filter` - 중복 아이템 필터링 사용 여부
    ///
    /// # Returns
    /// * `CandleStore<T>` - 생성된 저장소 인스턴스
    pub fn new(mut items: Vec<T>, max_size: usize, use_duplicated_filter: bool) -> CandleStore<T> {
        // datetime 기준으로 내림차순 정렬 (최신 데이터가 먼저 오도록)
        items.sort_by_key(|b| std::cmp::Reverse(b.datetime()));

        // 최대 크기를 초과하는 아이템들 제거
        if items.len() > max_size {
            items.truncate(max_size);
        }
        CandleStore {
            items,
            max_size,
            use_duplicated_filter,
        }
    }

    /// 데이터를 datetime 기준으로 내림차순 정렬하여 삽입합니다.
    ///
    /// 이미 저장소가 최대 크기에 도달했다면, 가장 오래된 데이터가 제거됩니다.
    /// 중복 필터링이 활성화된 경우, 첫 번째 아이템과 동일한 타임스탬프를 가진 데이터는 삽입하지 않습니다.
    ///
    /// # Arguments
    /// * `data` - 삽입할 데이터
    pub fn add(&mut self, data: T) {
        // 중복 필터링이 활성화되고 첫 번째 아이템과 동일한 타임스탬프면 무시
        // 내림차순 정렬이므로 최신 데이터가 첫 번째에 위치하여 첫 번째만 확인하면 됨
        if self.use_duplicated_filter
            && !self.items.is_empty()
            && self.items.first().map(|item| item.datetime()) == Some(data.datetime())
        {
            return;
        }

        // datetime 기준으로 내림차순 정렬된 위치 찾기
        // binary_search_by는 오름차순 배열을 가정하지만, Err(idx)로 반환되는 인덱스가
        // 내림차순 배열에서도 올바른 삽입 위치를 제공함 (테스트로 검증됨)
        let insert_idx = self
            .items
            .binary_search_by(|item| data.datetime().cmp(&item.datetime()))
            .unwrap_or_else(|idx| idx);

        // 데이터 삽입
        self.items.insert(insert_idx, data);

        // 최대 크기 초과 시 초과분 제거 (truncate 사용으로 최적화)
        if self.items.len() > self.max_size {
            self.items.truncate(self.max_size);
        }
    }

    /// 저장소에 있는 아이템 수를 반환합니다.
    ///
    /// # Returns
    /// * `usize` - 아이템 수
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 저장소가 비어 있는지 확인합니다.
    ///
    /// # Returns
    /// * `bool` - 저장소가 비어 있으면 true
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// 첫 번째 아이템을 반환합니다 (가장 최신 데이터).
    ///
    /// # Returns
    /// * `Option<&T>` - 첫 번째 아이템 또는 None
    pub fn first(&self) -> Option<&T> {
        self.items.first()
    }

    /// 마지막 아이템을 반환합니다 (가장 오래된 데이터).
    ///
    /// # Returns
    /// * `Option<&T>` - 마지막 아이템 또는 None
    pub fn last(&self) -> Option<&T> {
        self.items.last()
    }

    /// 지정된 인덱스의 아이템을 반환합니다.
    ///
    /// # Arguments
    /// * `index` - 가져올 아이템의 인덱스
    ///
    /// # Returns
    /// * `Option<&T>` - 해당 인덱스의 아이템 또는 None
    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }

    /// 저장소의 모든 아이템에 대한 참조 슬라이스를 반환합니다.
    ///
    /// # Returns
    /// * `&[T]` - 아이템 슬라이스
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// 저장된 캔들의 가격이 연속적으로 상승하는지 확인합니다.
    ///
    /// 내림차순 배열에서 최신 데이터(인덱스 0)가 가장 오래된 데이터보다 높은 가격을 가져야 상승으로 판단합니다.
    ///
    /// # Arguments
    /// * `n` - 확인할 캔들의 수
    ///
    /// # Returns
    /// * `bool` - n개의 캔들이 연속적으로 상승하면 true
    pub fn is_rise(&self, n: usize) -> bool {
        let count = self.items.len().min(n);
        if count < 2 {
            return false;
        }

        // windows 반복자를 사용하여 연속된 쌍을 효율적으로 확인
        // 내림차순 배열이므로 최신(인덱스 0) > 이전(인덱스 1) > ... 순서로 가격이 높아야 상승
        let result = self.items[..count]
            .windows(2)
            .all(|w| w[0].close_price() > w[1].close_price());

        if result {
            log::trace!("RISE: true");
        }

        result
    }

    /// 저장된 캔들의 가격이 연속적으로 하락하는지 확인합니다.
    ///
    /// 내림차순 배열에서 최신 데이터(인덱스 0)가 가장 오래된 데이터보다 낮은 가격을 가져야 하락으로 판단합니다.
    ///
    /// # Arguments
    /// * `n` - 확인할 캔들의 수
    ///
    /// # Returns
    /// * `bool` - n개의 캔들이 연속적으로 하락하면 true
    pub fn is_fall(&self, n: usize) -> bool {
        let count = self.items.len().min(n);
        if count < 2 {
            return false;
        }

        // windows 반복자를 사용하여 연속된 쌍을 효율적으로 확인
        // 내림차순 배열이므로 최신(인덱스 0) < 이전(인덱스 1) < ... 순서로 가격이 낮아야 하락
        let result = self.items[..count]
            .windows(2)
            .all(|w| w[0].close_price() < w[1].close_price());

        if result {
            log::trace!("FALL: true");
        }

        result
    }

    /// 저장된 캔들을 시간 오름차순으로 정렬하여 반환합니다.
    ///
    /// 오래된 캔들부터 최신 캔들 순서로 정렬된 벡터를 반환합니다.
    ///
    /// # Returns
    /// * `Vec<T>` - 시간 오름차순으로 정렬된 캔들 목록 (오래된 것부터 최신 순서)
    pub fn get_ascending_items(&self) -> Vec<T> {
        // 내림차순으로 저장된 데이터를 오름차순으로 변환
        self.items.iter().rev().cloned().collect()
    }
}
