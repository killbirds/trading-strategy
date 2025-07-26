# 기술적 필터 설정 가이드 (TechnicalFilterConfig)

이 문서는 트레이딩 전략에서 사용할 수 있는 모든 기술적 필터의 설정 방법과 파라미터에 대한 완전한 가이드입니다.

## 목차

1. [RSI 필터](#rsi-필터)
2. [MACD 필터](#macd-필터)
3. [볼린저 밴드 필터](#볼린저-밴드-필터)
4. [ADX 필터](#adx-필터)
5. [이동평균선 필터](#이동평균선-필터)
6. [이치모쿠 필터](#이치모쿠-필터)
7. [VWAP 필터](#vwap-필터)
8. [CopyS 필터](#copys-필터)
9. [ATR 필터](#atr-필터)
10. [SuperTrend 필터](#supertrend-필터)
11. [Volume 필터](#volume-필터)
12. [ThreeRSI 필터](#threersi-필터)
13. [CandlePattern 필터](#candlepattern-필터)
14. [SupportResistance 필터](#supportresistance-필터)
15. [Momentum 필터](#momentum-필터)

---

## RSI 필터

**목적**: Relative Strength Index를 사용한 과매수/과매도 상태 필터링

### 파라미터

| 파라미터 | 타입 | 기본값 | 설명 |
|---------|------|-------|------|
| `period` | usize | 14 | RSI 계산 기간 |
| `oversold` | f64 | 30.0 | 과매도 기준점 |
| `overbought` | f64 | 70.0 | 과매수 기준점 |
| `filter_type` | i32 | 0 | 필터 유형 (아래 참조) |
| `consecutive_n` | usize | 1 | 연속 캔들 수 |

### 필터 유형 (filter_type)

| 값 | 설명 |
|----|------|
| 0 | 과매수 (RSI > overbought) |
| 1 | 과매도 (RSI < oversold) |
| 2 | 정상 범위 (oversold < RSI < overbought) |
| 3 | 상향 돌파 (RSI가 임계값을 상향 돌파) |
| 4 | 하향 돌파 (RSI가 임계값을 하향 돌파) |
| 5 | 50 상향 돌파 |
| 6 | 50 하향 돌파 |
| 7 | RSI 상승 추세 |
| 8 | RSI 하락 추세 |

### 설정 예시

```toml
[[filters]]
type = "RSI"
period = 14
oversold = 30.0
overbought = 70.0
filter_type = 1  # 과매도 상태
consecutive_n = 2
```

---

## MACD 필터

**목적**: Moving Average Convergence Divergence를 사용한 추세 필터링

### 파라미터

| 파라미터 | 타입 | 기본값 | 설명 |
|---------|------|-------|------|
| `fast_period` | usize | 12 | 빠른 이동평균 기간 |
| `slow_period` | usize | 26 | 느린 이동평균 기간 |
| `signal_period` | usize | 9 | 시그널 라인 기간 |
| `filter_type` | i32 | 0 | 필터 유형 |
| `consecutive_n` | usize | 1 | 연속 캔들 수 |
| `threshold` | f64 | 0.0 | 히스토그램 임계값 |

### 필터 유형 (filter_type)

| 값 | 설명 |
|----|------|
| 0 | MACD > 시그널 |
| 1 | MACD < 시그널 |
| 2 | 시그널 상향돌파 |
| 3 | 시그널 하향돌파 |
| 4 | 히스토그램 > 임계값 |
| 5 | 히스토그램 < 임계값 |
| 6 | 제로라인 상향돌파 |
| 7 | 제로라인 하향돌파 |
| 8 | 히스토그램 음전환 |
| 9 | 히스토그램 양전환 |
| 10 | 강한 상승 추세 |

### 설정 예시

```toml
[[filters]]
type = "MACD"
fast_period = 12
slow_period = 26
signal_period = 9
filter_type = 2  # 시그널 상향돌파
consecutive_n = 1
threshold = 0.001
```

---

## 볼린저 밴드 필터

**목적**: Bollinger Bands를 사용한 변동성 및 가격 위치 필터링

### 파라미터

| 파라미터 | 타입 | 기본값 | 설명 |
|---------|------|-------|------|
| `period` | usize | 20 | 볼린저 밴드 기간 |
| `dev_mult` | f64 | 2.0 | 표준편차 배수 |
| `filter_type` | i32 | 0 | 필터 유형 |
| `consecutive_n` | usize | 1 | 연속 캔들 수 |

### 필터 유형 (filter_type)

| 값 | 설명 |
|----|------|
| 0 | 상단밴드 위 |
| 1 | 하단밴드 아래 |
| 2 | 밴드 내부 |
| 3 | 밴드 외부 |
| 4 | 중간밴드 위 |
| 5 | 중간밴드 아래 |
| 6 | 밴드 폭 충분 |
| 7 | 하단밴드 상향돌파 |
| 8 | 스퀴즈 돌파 |
| 9 | 향상된 스퀴즈 돌파 |
| 10 | 스퀴즈 상태 |
| 11 | 밴드 폭 좁아짐 |
| 12 | 스퀴즈 확장 시작 |

### 설정 예시

```toml
[[filters]]
type = "BOLLINGER_BAND"
period = 20
dev_mult = 2.0
filter_type = 7  # 하단밴드 상향돌파
consecutive_n = 1
```

---

## ADX 필터

**목적**: Average Directional Index를 사용한 추세 강도 필터링

### 파라미터

| 파라미터 | 타입 | 기본값 | 설명 |
|---------|------|-------|------|
| `period` | usize | 14 | ADX 계산 기간 |
| `threshold` | f64 | 25.0 | ADX 임계값 |
| `filter_type` | i32 | 0 | 필터 유형 |
| `consecutive_n` | usize | 1 | 연속 캔들 수 |

### 필터 유형 (filter_type)

| 값 | 설명 |
|----|------|
| 0 | ADX < 임계값 (약한 추세) |
| 1 | ADX > 임계값 (강한 추세) |
| 2 | +DI > -DI (상승 추세) |
| 3 | -DI > +DI (하락 추세) |
| 4 | ADX > 임계값 & +DI > -DI |
| 5 | ADX > 임계값 & -DI > +DI |
| 6 | ADX 상승 |
| 7 | ADX 하락 |
| 8 | DI 간격 확대 |
| 9 | DI 간격 축소 |

### 설정 예시

```toml
[[filters]]
type = "ADX"
period = 14
threshold = 25.0
filter_type = 4  # 강한 상승 추세
consecutive_n = 2
```

---

## 이동평균선 필터

**목적**: Moving Average를 사용한 추세 및 가격 위치 필터링

### 파라미터

| 파라미터 | 타입 | 기본값 | 설명 |
|---------|------|-------|------|
| `periods` | Vec<usize> | [5, 20] | 이동평균 기간 목록 |
| `filter_type` | i32 | 0 | 필터 유형 |
| `consecutive_n` | usize | 1 | 연속 캔들 수 |

### 필터 유형 (filter_type)

| 값 | 설명 |
|----|------|
| 0 | 가격 > 첫번째 MA |
| 1 | 가격 > 마지막 MA |
| 2 | 정규 배열 (짧은 기간 > 긴 기간) |
| 3 | 첫번째 MA > 마지막 MA |
| 4 | 첫번째 MA < 마지막 MA |
| 5 | 골든 크로스 |
| 6 | 가격이 첫번째와 마지막 MA 사이 |
| 7 | MA 수렴 |
| 8 | MA 발산 |
| 9 | 모든 MA 위 |
| 10 | 모든 MA 아래 |

### 설정 예시

```toml
[[filters]]
type = "MOVING_AVERAGE"
periods = [5, 20, 60]
filter_type = 2  # 정규 배열
consecutive_n = 3
```

---

## 이치모쿠 필터

**목적**: Ichimoku Cloud를 사용한 종합적 추세 분석

### 파라미터

| 파라미터 | 타입 | 기본값 | 설명 |
|---------|------|-------|------|
| `tenkan_period` | usize | 9 | 전환선 기간 |
| `kijun_period` | usize | 26 | 기준선 기간 |
| `senkou_span_b_period` | usize | 52 | 선행스팬B 기간 |
| `filter_type` | i32 | 0 | 필터 유형 |
| `consecutive_n` | usize | 1 | 연속 캔들 수 |

### 필터 유형 (filter_type)

| 값 | 설명 |
|----|------|
| 0 | 가격 > 구름 |
| 1 | 가격 < 구름 |
| 2 | 전환선 > 기준선 |
| 3 | 골든 크로스 |
| 4 | 데드 크로스 |
| 5 | 구름 상향돌파 |
| 6 | 구름 하향돌파 |
| 7 | 매수 신호 |
| 8 | 매도 신호 |
| 9 | 구름 두께 증가 |
| 10 | 완벽 정렬 |
| 11 | 완벽 역배열 |
| 12 | 강한 매수 신호 |

### 설정 예시

```toml
[[filters]]
type = "ICHIMOKU"
tenkan_period = 9
kijun_period = 26
senkou_span_b_period = 52
filter_type = 7  # 매수 신호
consecutive_n = 1
```

---

## VWAP 필터

**목적**: Volume Weighted Average Price를 사용한 가격/거래량 분석

### 파라미터

| 파라미터 | 타입 | 기본값 | 설명 |
|---------|------|-------|------|
| `period` | usize | 20 | VWAP 계산 기간 |
| `filter_type` | i32 | 0 | 필터 유형 |
| `consecutive_n` | usize | 1 | 연속 캔들 수 |
| `threshold` | f64 | 0.05 | 임계값 (5%) |

### 필터 유형 (filter_type)

| 값 | 설명 |
|----|------|
| 0 | 가격 > VWAP |
| 1 | 가격 < VWAP |
| 2 | 가격 ≈ VWAP |
| 3 | VWAP 상향돌파 |
| 4 | VWAP 하향돌파 |
| 5 | VWAP 리바운드 |
| 6 | VWAP 간격 확대 |
| 7 | VWAP 간격 축소 |
| 8 | 강한 상승 |
| 9 | 강한 하락 |
| 10 | 추세 강화 |
| 11 | 추세 약화 |

### 설정 예시

```toml
[[filters]]
type = "VWAP"
period = 20
filter_type = 3  # VWAP 상향돌파
consecutive_n = 1
threshold = 0.03
```

---

## CopyS 필터

**목적**: 복합 기술적 지표를 조합한 통합 신호 분석

### 파라미터

| 파라미터 | 타입 | 기본값 | 설명 |
|---------|------|-------|------|
| `rsi_period` | usize | 14 | RSI 계산 기간 |
| `rsi_upper` | f64 | 70.0 | RSI 상한 기준점 |
| `rsi_lower` | f64 | 30.0 | RSI 하한 기준점 |
| `filter_type` | i32 | 0 | 필터 유형 |
| `consecutive_n` | usize | 1 | 연속 캔들 수 |

### 필터 유형 (filter_type)

| 값 | 설명 |
|----|------|
| 0 | 기본 매수 신호 |
| 1 | 기본 매도 신호 |
| 2 | RSI 과매도 |
| 3 | RSI 과매수 |
| 4 | 볼린저밴드 하단 터치 |
| 5 | 볼린저밴드 상단 터치 |
| 6 | 이평선 지지 |
| 7 | 이평선 저항 |
| 8 | 강한 매수 신호 |
| 9 | 강한 매도 신호 |
| 10 | 약한 매수 신호 |
| 11 | 약한 매도 신호 |
| 12 | RSI 중립대 |
| 13 | 볼린저밴드 내부 |
| 14 | 이평선 정배열 |
| 15 | 이평선 역배열 |

### 설정 예시

```toml
[[filters]]
type = "COPYS"
rsi_period = 14
rsi_upper = 70.0
rsi_lower = 30.0
filter_type = 8  # 강한 매수 신호
consecutive_n = 2
```

---

## ATR 필터

**목적**: Average True Range를 사용한 변동성 분석

### 파라미터

| 파라미터 | 타입 | 기본값 | 설명 |
|---------|------|-------|------|
| `period` | usize | 14 | ATR 계산 기간 |
| `threshold` | f64 | 0.01 | ATR 임계값 |
| `filter_type` | i32 | 0 | 필터 유형 |
| `consecutive_n` | usize | 1 | 연속 캔들 수 |

### 필터 유형 (filter_type)

| 값 | 설명 |
|----|------|
| 0 | ATR이 임계값 이상 |
| 1 | 변동성 확장 |
| 2 | 변동성 수축 |
| 3 | 높은 변동성 |
| 4 | 낮은 변동성 |
| 5 | 변동성 증가 |
| 6 | 변동성 감소 |

### 설정 예시

```toml
[[filters]]
type = "ATR"
period = 14
threshold = 0.015
filter_type = 1  # 변동성 확장
consecutive_n = 1
```

---

## SuperTrend 필터

**목적**: SuperTrend 지표를 사용한 추세 방향 분석

### 파라미터

| 파라미터 | 타입 | 기본값 | 설명 |
|---------|------|-------|------|
| `period` | usize | 10 | SuperTrend 계산 기간 |
| `multiplier` | f64 | 3.0 | SuperTrend 승수 |
| `filter_type` | i32 | 0 | 필터 유형 |
| `consecutive_n` | usize | 1 | 연속 캔들 수 |

### 필터 유형 (filter_type)

| 값 | 설명 |
|----|------|
| 0 | 모든 설정에서 상승 추세 |
| 1 | 모든 설정에서 하락 추세 |
| 2 | 가격이 슈퍼트렌드 위에 있음 |
| 3 | 가격이 슈퍼트렌드 아래에 있음 |
| 4 | 가격이 슈퍼트렌드를 상향 돌파 |
| 5 | 가격이 슈퍼트렌드를 하향 돌파 |
| 6 | 추세 변경 |
| 7 | 특정 설정에서 상승 추세 |
| 8 | 특정 설정에서 하락 추세 |

### 설정 예시

```toml
[[filters]]
type = "SUPERTREND"
period = 10
multiplier = 3.0
filter_type = 4  # 상향 돌파
consecutive_n = 1
```

---

## Volume 필터

**목적**: 거래량 분석을 통한 시장 참여도 확인

### 파라미터

| 파라미터 | 타입 | 기본값 | 설명 |
|---------|------|-------|------|
| `period` | usize | 20 | Volume 계산 기간 |
| `threshold` | f64 | 1.5 | Volume 임계값 |
| `filter_type` | i32 | 0 | 필터 유형 |
| `consecutive_n` | usize | 1 | 연속 캔들 수 |

### 필터 유형 (filter_type)

| 값 | 설명 |
|----|------|
| 0 | 볼륨이 평균 이상 |
| 1 | 볼륨이 평균 이하 |
| 2 | 볼륨 급등 |
| 3 | 볼륨 감소 |
| 4 | 볼륨이 현저히 높음 |
| 5 | 상승과 함께 볼륨 증가 |
| 6 | 하락과 함께 볼륨 증가 |
| 7 | 상승 추세에서 볼륨 증가 |
| 8 | 하락 추세에서 볼륨 감소 |

### 설정 예시

```toml
[[filters]]
type = "VOLUME"
period = 20
threshold = 2.0
filter_type = 5  # 상승과 함께 볼륨 증가
consecutive_n = 1
```

---

## ThreeRSI 필터

**목적**: 3개의 다른 기간 RSI를 조합한 강화된 모멘텀 분석

### 파라미터

| 파라미터 | 타입 | 기본값 | 설명 |
|---------|------|-------|------|
| `rsi_periods` | Vec<usize> | [7, 14, 21] | RSI 계산 기간 목록 |
| `ma_type` | String | "SMA" | 이동평균 타입 |
| `ma_period` | usize | 20 | 이동평균 기간 |
| `adx_period` | usize | 14 | ADX 기간 |
| `filter_type` | i32 | 0 | 필터 유형 |
| `consecutive_n` | usize | 1 | 연속 캔들 수 |

### 필터 유형 (filter_type)

| 값 | 설명 |
|----|------|
| 0 | 모든 RSI가 50 미만 |
| 1 | 모든 RSI가 50 이상 |
| 2 | RSI 역순 배열 |
| 3 | RSI 정상 배열 |
| 4 | 캔들 저가가 이동평균 아래 |
| 5 | 캔들 고가가 이동평균 위 |
| 6 | ADX가 20 이상 |

### 설정 예시

```toml
[[filters]]
type = "THREERSI"
rsi_periods = [7, 14, 21]
ma_type = "EMA"
ma_period = 20
adx_period = 14
filter_type = 3  # RSI 정상 배열
consecutive_n = 2
```

---

## CandlePattern 필터

**목적**: 캔들스틱 패턴 분석을 통한 시장 심리 파악

### 파라미터

| 파라미터 | 타입 | 기본값 | 설명 |
|---------|------|-------|------|
| `min_body_ratio` | f64 | 0.3 | 최소 몸통 크기 비율 |
| `min_shadow_ratio` | f64 | 0.3 | 최소 꼬리 크기 비율 |
| `pattern_history_length` | usize | 5 | 패턴 히스토리 길이 |
| `threshold` | f64 | 0.5 | 임계값 |
| `filter_type` | i32 | 0 | 필터 유형 |
| `consecutive_n` | usize | 1 | 연속 캔들 수 |

### 필터 유형 (filter_type)

| 값 | 설명 |
|----|------|
| 0 | 강한 상승 패턴 |
| 1 | 강한 하락 패턴 |
| 2 | 반전 패턴 |
| 3 | 지속 패턴 |
| 4 | 볼륨으로 확인된 패턴 |
| 5 | 높은 신뢰도 패턴 |
| 6 | 컨텍스트에 맞는 패턴 |
| 7 | 강한 반전 신호 |
| 8 | 높은 신뢰도 신호 |
| 9 | 볼륨 확인 신호 |
| 10 | 패턴 클러스터링 신호 |

### 설정 예시

```toml
[[filters]]
type = "CANDLEPATTERN"
min_body_ratio = 0.4
min_shadow_ratio = 0.2
pattern_history_length = 7
threshold = 0.6
filter_type = 5  # 높은 신뢰도 패턴
consecutive_n = 1
```

---

## SupportResistance 필터

**목적**: 지지선과 저항선 분석을 통한 가격 레벨 확인

### 파라미터

| 파라미터 | 타입 | 기본값 | 설명 |
|---------|------|-------|------|
| `lookback_period` | usize | 20 | 되돌아 볼 기간 |
| `touch_threshold` | f64 | 0.01 | 터치 임계값 |
| `min_touch_count` | usize | 2 | 최소 터치 횟수 |
| `threshold` | f64 | 0.05 | 거리 임계값 |
| `filter_type` | i32 | 0 | 필터 유형 |
| `consecutive_n` | usize | 1 | 연속 캔들 수 |

### 필터 유형 (filter_type)

| 값 | 설명 |
|----|------|
| 0 | 지지선 하향 돌파 |
| 1 | 저항선 상향 돌파 |
| 2 | 지지선 반등 |
| 3 | 저항선 거부 |
| 4 | 강한 지지선 근처 |
| 5 | 강한 저항선 근처 |
| 6 | 지지선 위에 있음 |
| 7 | 저항선 아래에 있음 |
| 8 | 지지선 근처 |
| 9 | 저항선 근처 |

### 설정 예시

```toml
[[filters]]
type = "SUPPORTRESISTANCE"
lookback_period = 30
touch_threshold = 0.008
min_touch_count = 3
threshold = 0.03
filter_type = 1  # 저항선 상향 돌파
consecutive_n = 1
```

---

## Momentum 필터

**목적**: 다양한 모멘텀 지표를 조합한 종합적 모멘텀 분석

### 파라미터

| 파라미터 | 타입 | 기본값 | 설명 |
|---------|------|-------|------|
| `rsi_period` | usize | 14 | RSI 기간 |
| `stoch_period` | usize | 14 | 스토캐스틱 기간 |
| `williams_period` | usize | 14 | 윌리엄스 %R 기간 |
| `roc_period` | usize | 10 | ROC 기간 |
| `cci_period` | usize | 20 | CCI 기간 |
| `momentum_period` | usize | 10 | 모멘텀 기간 |
| `history_length` | usize | 50 | 히스토리 길이 |
| `threshold` | f64 | 0.5 | 임계값 |
| `filter_type` | i32 | 0 | 필터 유형 |
| `consecutive_n` | usize | 1 | 연속 캔들 수 |

### 필터 유형 (filter_type)

| 값 | 설명 |
|----|------|
| 0 | 강한 양의 모멘텀 |
| 1 | 강한 음의 모멘텀 |
| 2 | 가속하는 모멘텀 |
| 3 | 감속하는 모멘텀 |
| 4 | 과매수 상태 |
| 5 | 과매도 상태 |
| 6 | 모멘텀 다이버전스 |
| 7 | 불리시 다이버전스 |
| 8 | 베어리시 다이버전스 |
| 9 | 지속적인 모멘텀 |
| 10 | 안정적인 모멘텀 |
| 11 | 모멘텀 반전 신호 |

### 설정 예시

```toml
[[filters]]
type = "MOMENTUM"
rsi_period = 14
stoch_period = 14
williams_period = 14
roc_period = 10
cci_period = 20
momentum_period = 10
history_length = 50
threshold = 0.6
filter_type = 2  # 가속하는 모멘텀
consecutive_n = 2
```

---

## 복합 필터 설정 예시

여러 필터를 조합하여 사용하는 예시:

```toml
# 강세 시장을 위한 복합 필터 설정
[[filters]]
type = "ADX"
period = 14
threshold = 25.0
filter_type = 4  # 강한 상승 추세
consecutive_n = 2

[[filters]]
type = "RSI"
period = 14
oversold = 30.0
overbought = 70.0
filter_type = 2  # 정상 범위
consecutive_n = 1

[[filters]]
type = "MOVING_AVERAGE"
periods = [5, 20, 60]
filter_type = 2  # 정규 배열
consecutive_n = 3

[[filters]]
type = "VOLUME"
period = 20
threshold = 1.5
filter_type = 5  # 상승과 함께 볼륨 증가
consecutive_n = 1
```

---

## 주의사항

1. **연속 캔들 수 (consecutive_n)**: 높은 값은 더 확실한 신호를 제공하지만 진입 기회를 줄일 수 있습니다.

2. **임계값 설정**: 각 필터의 임계값은 시장 상황과 자산의 특성에 맞게 조정해야 합니다.

3. **필터 조합**: 너무 많은 필터를 사용하면 과적합될 수 있으므로 적절한 균형을 찾는 것이 중요합니다.

4. **백테스팅**: 실제 운용 전에 충분한 백테스팅을 통해 필터 설정을 검증해야 합니다.

5. **시장 상황**: 불안정한 시장에서는 더 보수적인 설정을 사용하는 것이 좋습니다. 