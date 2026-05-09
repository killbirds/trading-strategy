# 기술적 필터 설정 가이드

이 문서는 `src/filter/` 실제 구현을 기준으로 다시 정리한 **구현 기준 레퍼런스**입니다.

- 기준 소스: `src/filter/mod.rs`, `src/filter/*.rs`, `src/analyzer/slope_analyzer.rs`, `src/indicator/ma/mod.rs`, `src/strategy/copys_common.rs`
- 지원 필터 종류: **28개**
- 전체 `filter_type` variant 수: **381개**
- 실제 조합 예시는 `ta_filter_sample/` 문서를 참고하세요.

이전 문서에 있던 장세 해석/전략 추천 성격의 설명은 코드와 1:1로 대응되지 않는 부분이 많아서, 여기서는 **코드가 실제로 허용하는 설정면**만 정리합니다.

---

## 1. 공통 설정 규칙

### 기본 구조

```toml
[[filters]]
type = "RSI"
filter_type = "Overbought"
consecutive_n = 1
p = 0
```

### `type` 값

실제 `TechnicalFilterConfig` 가 받는 값은 아래와 같습니다.

- `RSI`
- `MACD`
- `BOLLINGER_BAND`
- `BOX_RANGE`
- `ADX`
- `MOVING_AVERAGE`
- `PRICE_REFERENCE_GAP`
- `ICHIMOKU`
- `VWAP`
- `COPYS`
- `ATR`
- `SUPERTREND`
- `VOLUME`
- `THREERSI`
- `CANDLEPATTERN`
- `SUPPORTRESISTANCE`
- `MOMENTUM`
- `DONCHIAN`
- `KELTNER`
- `OBV`
- `MFI`
- `AROON`
- `CHOPPINESS`
- `KAMA`
- `CHAIKIN`
- `PPO`
- `PARABOLIC_SAR`
- `SLOPE`

### `filter_type` 입력 규칙

- 대부분의 필터는 `filter_type` 에 **enum 문자열** 또는 **0부터 시작하는 정수 인덱스**를 넣을 수 있습니다.
- `SLOPE` 도 현재 구현과 테스트 기준으로 정수 인덱스 입력을 지원합니다. 예: `filter_type = 3` 은 `StrengthAboveThreshold` 입니다.
- 이 문서는 가독성을 위해 **문자열 enum 이름 기준**으로 설명합니다.
- 숫자 인덱스는 enum 선언 순서에 강하게 묶이므로, 장기 유지보수용 설정에는 문자열 사용을 권장합니다.

### `type` alias 와 legacy shorthand

`TechnicalFilterConfig` 의 `type` 은 serde tag 이므로 TOML/JSON 에서는 아래 공식 이름을 쓰는 것이 가장 안전합니다. 내부 `FromStr` 는 일부 대소문자/언더스코어 차이를 허용하지만, 역직렬화 tag 는 `#[serde(rename = ...)]` 기준으로 동작합니다.

| 권장 `type` | 허용 의도 | 주의 |
| --- | --- | --- |
| `BOLLINGER_BAND` | BollingerBand 계열 | legacy shorthand `BB` 는 거부 |
| `BOX_RANGE` | BoxRange 계열 | `BOXRANGE` 는 `FromStr` 에서는 허용되지만 설정 tag 로는 `BOX_RANGE` 권장 |
| `MOVING_AVERAGE` | MovingAverage 계열 | legacy shorthand `MA` 는 거부 |
| `PRICE_REFERENCE_GAP` | PriceReferenceGap 계열 | `reference_source.type = "MOVING_AVERAGE"` 에서도 legacy `MA` 는 거부 |
| `PARABOLIC_SAR` | ParabolicSAR 계열 | `PARABOLICSAR` 는 `FromStr` 에서는 허용되지만 설정 tag 로는 `PARABOLIC_SAR` 권장 |

> 구현은 `deny_unknown_fields` 를 사용합니다. 오타가 난 필드는 조용히 무시되지 않고 역직렬화 단계에서 실패합니다.

### 공통 필드

| 필드            | 의미                                  |
| --------------- | ------------------------------------- |
| `filter_type`   | 필터별 enum 이름                      |
| `consecutive_n` | 조건을 연속으로 만족해야 하는 캔들 수 |
| `p`             | 현재 캔들 기준 과거 오프셋            |

`p` 와 `consecutive_n` 해석:

- analyzer/filter 내부 데이터는 보통 **최신 데이터가 index 0** 인 형태로 저장됩니다.
- `p = 0` 은 최신 기준값, `p = 1` 은 한 캔들 전 기준값을 의미합니다.
- `consecutive_n = N` 은 `p` 부터 시작해 `N`개 연속 데이터가 조건을 만족해야 한다는 뜻입니다.
- 교차/상승/하락처럼 이전 데이터와 비교하는 필터는 내부적으로 `p + consecutive_n + 1` 개 이상의 분석 결과를 요구할 수 있습니다.
- 가격 비교형 필터에서 `p` 는 기준 지표/캔들 선택에만 영향을 주고, 비교 대상인 `current_price` 자체를 과거 가격으로 바꾸지는 않습니다.

검증 규칙 요약:

| 규칙 | 대표 필드 |
| --- | --- |
| 기간은 0보다 커야 함 | `period`, `fast_period`, `slow_period`, `signal_period`, `lookback_period` |
| `consecutive_n` 은 0보다 커야 함 | 모든 `consecutive_n` |
| 비율 임계값은 보통 유한한 0 이상 숫자 | `sideways_threshold`, `threshold`, `edge_threshold` 등 |
| `PriceReferenceGap.gap_threshold` 는 `0.0..=1.0` | `gap_threshold` |
| ADX/RSI 계열 퍼센트 임계값은 `0.0..=100.0` | `ADX.threshold`, `RSI.cross_threshold`, `oversold`, `overbought` |
| 순서가 있는 기간은 `left < right` | `MACD fast_period < slow_period`, `KAMA fast_period < slow_period`, `PPO fast_period < slow_period`, `ParabolicSAR step < max_step` |
| 빈 배열은 허용하지 않음 | `MovingAverage.periods`, `ThreeRSI.rsi_periods`, Slope RSI `ma_periods` |

### 중첩 값 표기 규칙

- `reference_source.type`: `MOVING_AVERAGE`, `VWAP`, `HIGHEST_HIGH`, `LOWEST_LOW`
- `indicator_type.type`: `ClosePrice`, `HighPrice`, `LowPrice`, `MovingAverage`, `RSI`, `MACD`, `MACDLine`, `MACDSignalLine`, `MACDHistogram`
- `ma_type`: `EMA`, `SMA`, `WMA`
  - 단, `PriceReferenceGap` 의 `reference_source = { type = "MOVING_AVERAGE", ... }` 는 현재 `EMA` 와 `SMA` 만 허용합니다.

> 아래 최소 필요 캔들 수는 각 필터 함수의 **상위 guard** 기준입니다. 일부 교차/패턴 계열은 내부에서 추가 히스토리를 더 확인합니다.

---

## 2. 빠른 참조

| 필터              | `type` 값             | `filter_type` 수 | 최소 필요 캔들 수                                     |
| ----------------- | --------------------- | ---------------: | ----------------------------------------------------- |
| RSI               | `RSI`                 |               23 | `period + consecutive_n`                              |
| MACD              | `MACD`                |               21 | `slow_period + signal_period + consecutive_n`         |
| BollingerBand     | `BOLLINGER_BAND`      |               31 | `period`                                              |
| BoxRange          | `BOX_RANGE`           |               12 | 필터 타입에 따라 다름                                 |
| ADX               | `ADX`                 |               31 | `period * 2 + consecutive_n`                          |
| MovingAverage     | `MOVING_AVERAGE`      |               23 | `max(periods)`                                        |
| Ichimoku          | `ICHIMOKU`            |               13 | `senkou_span_b_period + kijun_period + consecutive_n` |
| VWAP              | `VWAP`                |               12 | `period + consecutive_n`                              |
| PriceReferenceGap | `PRICE_REFERENCE_GAP` |                6 | 참조 소스에 따라 다름                                 |
| CopyS             | `COPYS`               |               16 | `60`                                                  |
| ATR               | `ATR`                 |                7 | `max(period, consecutive_n)`                          |
| SuperTrend        | `SUPERTREND`          |                9 | `max(period, consecutive_n)`                          |
| Volume            | `VOLUME`              |               21 | `max(period, consecutive_n)`                          |
| ThreeRSI          | `THREERSI`            |               28 | `max(ma_period, consecutive_n)`                       |
| CandlePattern     | `CANDLEPATTERN`       |               41 | `max(pattern_history_length, consecutive_n)`          |
| SupportResistance | `SUPPORTRESISTANCE`   |               10 | `max(lookback_period, consecutive_n)`                 |
| Momentum          | `MOMENTUM`            |               21 | `max(history_length, consecutive_n)`                  |
| Donchian          | `DONCHIAN`            |                7 | `period + p + consecutive_n`, breakout은 이전 캔들 추가 |
| Keltner           | `KELTNER`             |                7 | `period + p + consecutive_n`, breakout은 이전 캔들 추가 |
| OBV               | `OBV`                 |                2 | 이전 캔들 비교 필요                                    |
| MFI               | `MFI`                 |                4 | `period + 1 + p + consecutive_n - 1`                  |
| Aroon             | `AROON`               |                2 | `period + p + consecutive_n - 1`                      |
| Choppiness        | `CHOPPINESS`          |                2 | `period + p + consecutive_n - 1`                      |
| KAMA              | `KAMA`                |                8 | `period + 1`, 교차/상승/하락은 이전 캔들 추가          |
| Chaikin           | `CHAIKIN`             |                4 | `max(cmf_period, slow_period) + p + consecutive_n - 1` |
| PPO               | `PPO`                 |                6 | `slow_period + signal_period + p + consecutive_n - 1` |
| ParabolicSAR      | `PARABOLIC_SAR`       |                5 | 최소 2개, 교차/반전은 이전 캔들 추가                  |
| Slope             | `SLOPE`               |                9 | `period + consecutive_n`                              |

PriceReferenceGap 최소 필요 캔들 수:

- `MOVING_AVERAGE`, `VWAP`: `period + p + consecutive_n - 1`
- `HIGHEST_HIGH`, `LOWEST_LOW` + `include_current_candle = true`: `lookback_period + p + consecutive_n - 1`
- `HIGHEST_HIGH`, `LOWEST_LOW` + `include_current_candle = false`: `lookback_period + 1 + p + consecutive_n - 1`

BoxRange 최소 필요 캔들 수:

- 일반 박스권/현재 박스 비교/폭 비율 필터: `period + p + consecutive_n - 1`
- `BreakoutAbove`, `BreakoutBelow`, `HighBreakThroughUpperBox`, `LowBreakThroughLowerBox`: `period + p + consecutive_n`
- `BoxRangeStart`: `period + p + consecutive_n - 1 + prior_box_count`

실시간 가격 비교가 필요한 필터는 캔들 저장소의 최신 `close` 를 현재가로 사용하지 않습니다. 필터 평가 시 외부에서 전달한 `current_price` 로 가격 조건을 판단하고, `p` 는 캔들/지표 기준값을 선택하는 오프셋으로만 사용합니다.

---

## 2.1 TOML 작성 패턴

필터는 보통 selector 또는 risk management 설정 안의 배열로 사용합니다. 배열 이름은 사용하는 애플리케이션 설정 구조에 따라 달라질 수 있지만, 각 원소의 shape 은 `TechnicalFilterConfig` 와 같습니다.

```toml
[[selector.technical_filters]]
type = "MACD"
fast_period = 18
slow_period = 26
signal_period = 10
filter_type = "ZeroLineCrossAbove"
consecutive_n = 2
p = 0
threshold = 0.01
overbought_threshold = 0.02
oversold_threshold = 0.02
sideways_threshold = 0.05

[[risk_management.technical_filters]]
type = "PRICE_REFERENCE_GAP"
filter_type = "GapBelowReferenceThreshold"
gap_threshold = 0.05
consecutive_n = 1
p = 0

[risk_management.technical_filters.reference_source]
type = "LOWEST_LOW"
lookback_period = 20
include_current_candle = false
```

같은 `reference_source` 는 inline table 로도 쓸 수 있습니다.

```toml
[[filters]]
type = "PRICE_REFERENCE_GAP"
filter_type = "GapAboveReferenceThreshold"
gap_threshold = 0.0
reference_source = { type = "MOVING_AVERAGE", ma_type = "EMA", period = 20 }
```

숫자 인덱스도 가능하지만 문자열을 권장합니다.

```toml
[[filters]]
type = "RSI"
filter_type = 0 # Overbought. enum 순서가 바뀌면 의미도 바뀔 수 있으므로 문자열 권장.
```

---

## 3. 필터별 레퍼런스

아래 `filter_type` 목록은 모두 **실제 enum 선언 순서**입니다. 숫자 인덱스를 써야 한다면 이 순서의 **0-based index** 를 사용하면 됩니다.

### RSI

- 기본값: `period=14`, `oversold=30.0`, `overbought=70.0`, `filter_type="Overbought"`, `consecutive_n=1`, `p=0`, `sideways_threshold=0.02`, `momentum_threshold=3.0`, `cross_threshold=50.0`
- 최소 필요 캔들 수: `period + consecutive_n`
- `filter_type`:

```text
Overbought, Oversold, NormalRange, CrossAboveThreshold, CrossBelowThreshold,
CrossAbove, CrossBelow, RisingTrend, FallingTrend, Sideways,
StrongRisingMomentum, StrongFallingMomentum, NeutralRange, Above40, Below60,
Above50, Below50, Divergence, Convergence, Stable, NeutralTrend, Bullish, Bearish
```

메모:

- `CrossAboveThreshold` / `CrossBelowThreshold` 는 `(oversold + overbought) / 2` 를 기준으로 동작합니다.
- `CrossAbove` / `CrossBelow` 가 `cross_threshold` 를 사용합니다.
- `Above50` / `Below50` 는 단순 `50 초과/미만` 이 아니라 **5개 RSI 값을 이용한 패턴 체크**입니다.
- `Bullish` 는 `60~80`, `Bearish` 는 `20~40` 범위 유지 체크입니다.

### MACD

- 기본값: `fast_period=12`, `slow_period=26`, `signal_period=9`, `filter_type="MacdAboveSignal"`, `consecutive_n=1`, `threshold=0.0`, `p=0`, `overbought_threshold=0.02`, `oversold_threshold=0.02`, `sideways_threshold=0.05`
- 최소 필요 캔들 수: `slow_period + signal_period + consecutive_n`
- `filter_type`:

```text
MacdAboveSignal, MacdBelowSignal, SignalCrossAbove, SignalCrossBelow,
HistogramAboveThreshold, HistogramBelowThreshold, ZeroLineCrossAbove,
ZeroLineCrossBelow, HistogramNegativeTurn, HistogramPositiveTurn,
StrongUptrend, StrongDowntrend, MacdRising, MacdFalling,
HistogramExpanding, HistogramContracting, Divergence, Convergence,
Overbought, Oversold, Sideways
```

### BollingerBand

- 기본값: `period=20`, `dev_mult=2.0`, `filter_type="AboveUpperBand"`, `consecutive_n=1`, `p=0`, `squeeze_threshold=0.02`, `medium_threshold=0.05`, `large_threshold=0.1`, `squeeze_breakout_period=5`, `enhanced_narrowing_period=3`, `enhanced_squeeze_period=2`, `upper_touch_threshold=0.99`, `lower_touch_threshold=1.01`
- 최소 필요 캔들 수: `period`
- `filter_type`:

```text
AboveUpperBand, BelowLowerBand, InsideBand, OutsideBand, AboveMiddleBand,
BelowMiddleBand, BandWidthSufficient, BreakThroughLowerBand, SqueezeBreakout,
EnhancedSqueezeBreakout, SqueezeState, BandWidthNarrowing, SqueezeExpansionStart,
BreakThroughUpperBand, BreakThroughLowerBandFromBelow, BandWidthExpanding,
MiddleBandSideways, UpperBandSideways, LowerBandSideways, BandWidthSideways,
UpperBandTouch, LowerBandTouch, BandWidthThresholdBreakthrough,
PriceMovingToUpperFromMiddle, PriceMovingToLowerFromMiddle,
BandConvergenceThenDivergence, BandDivergenceThenConvergence,
PriceMovingToUpperWithinBand, PriceMovingToLowerWithinBand,
LowVolatility, HighVolatility
```

메모: `BreakThroughLowerBand` 와 `BreakThroughLowerBandFromBelow` 는 현재 같은 구현을 사용합니다.

### BoxRange

- 기본값: `period=20`, `max_width_ratio=0.05`, `filter_type="IsBoxRange"`, `consecutive_n=1`, `p=0`, `prior_box_count=1`, `width_ratio_threshold=0.05`
- `max_width_ratio` 검증: 유한한 양수
- `width_ratio_threshold` 검증: 유한한 0 이상 숫자
- 최소 필요 캔들 수: 필터 타입에 따라 위 빠른 참조의 BoxRange 계산식 참고
- `filter_type`:

```text
IsBoxRange, InsideBox, OutsideBox, AboveUpperBox, BelowLowerBox,
BoxRangeStart, BreakoutAbove, BreakoutBelow, HighBreakThroughUpperBox,
LowBreakThroughLowerBox, WidthRatioBelowThreshold, WidthRatioAboveThreshold
```

메모:

- 박스권은 최근 `period`개 캔들의 `max(high)` / `min(low)` 로 상단과 하단을 만들고, 폭 비율이 `max_width_ratio` 이하이면 박스권으로 봅니다.
- `InsideBox`, `OutsideBox`, `AboveUpperBox`, `BelowLowerBox`, `BreakoutAbove`, `BreakoutBelow` 는 외부에서 전달한 `current_price` 기준으로 평가합니다.
- `BreakoutAbove` / `BreakoutBelow` 는 직전 박스권 경계와 `current_price` 를 비교합니다.
- `HighBreakThroughUpperBox` / `LowBreakThroughLowerBox` 는 현재 캔들의 `high` / `low` 가 직전 박스권 경계를 돌파했는지 확인하는 캔들 기반 필터입니다.
- `BoxRangeStart` 는 현재 구간이 박스권 조건을 만족하고, 그 이전 `prior_box_count` 구간은 박스권이 아니었는지 확인합니다.

### ADX

- 기본값: `period=14`, `threshold=25.0`, `filter_type="BelowThreshold"`, `consecutive_n=1`, `p=0`
- 최소 필요 캔들 수: `period * 2 + consecutive_n`
- `threshold` 검증 범위: `0.0..=100.0`
- `filter_type`:

```text
BelowThreshold, AboveThreshold, PDIAboveMDI, MDIAbovePDI, StrongUptrend,
StrongDowntrend, ADXRising, ADXFalling, DIGapExpanding, DIGapContracting,
ExtremeHigh, ExtremeLow, MiddleLevel, PDICrossAboveMDI, MDICrossAbovePDI,
Sideways, Surge, Crash, StrongDirectionality, WeakDirectionality,
TrendStrengthHigherThanDirection, ADXHigherThanMDI, PDIHigherThanADX,
MDIHigherThanADX, TrendReversalDown, TrendReversalUp, DICrossover,
ExtremePDI, ExtremeMDI, Stable, Unstable
```

### MovingAverage

- 기본값: `periods=[5,20]`, `filter_type="PriceAboveFirstMA"`, `consecutive_n=1`, `p=0`, `sideways_threshold=0.02`, `crossover_threshold=0.005`
- 최소 필요 캔들 수: `max(periods)`
- `filter_type`:

```text
PriceAboveFirstMA, PriceAboveLastMA, RegularArrangement, FirstMAAboveLastMA,
FirstMABelowLastMA, GoldenCross, PriceBetweenMA, MAConvergence,
MADivergence, AllMAAbove, AllMABelow, ReverseArrangement, DeadCross,
MASideways, StrongUptrend, StrongDowntrend, PriceCrossingMA,
ConvergenceDivergence, DivergenceConvergence, ParallelMovement,
NearCrossover, PriceBelowFirstMA, PriceBelowLastMA
```

메모: 현재 구현은 내부에서 **항상 `SMA`** 를 사용합니다. `ma_type` 설정은 없습니다.

### Ichimoku

- 기본값: `tenkan_period=9`, `kijun_period=26`, `senkou_span_b_period=52`, `filter_type="PriceAboveCloud"`, `consecutive_n=1`, `p=0`
- 최소 필요 캔들 수: `senkou_span_b_period + kijun_period + consecutive_n`
- `filter_type`:

```text
PriceAboveCloud, PriceBelowCloud, TenkanAboveKijun, GoldenCross, DeadCross,
CloudBreakoutUp, CloudBreakdown, BuySignal, SellSignal, CloudThickening,
PerfectAlignment, PerfectReverseAlignment, StrongBuySignal
```

메모: `StrongBuySignal` 은 현재 `BuySignal` 과 같은 구현을 사용합니다.

### VWAP

- 기본값: `period=20`, `filter_type="PriceAboveVWAP"`, `consecutive_n=1`, `threshold=0.05`, `p=0`
- 최소 필요 캔들 수: `period + consecutive_n`
- `filter_type`:

```text
PriceAboveVWAP, PriceBelowVWAP, PriceNearVWAP, VWAPBreakoutUp, VWAPBreakdown,
VWAPRebound, DivergingFromVWAP, ConvergingToVWAP, StrongUptrend,
StrongDowntrend, TrendStrengthening, TrendWeakening
```

메모:

- `StrongUptrend` = `PriceAboveVWAP`
- `StrongDowntrend` = `PriceBelowVWAP`
- `TrendStrengthening` = `DivergingFromVWAP`
- `TrendWeakening` = `ConvergingToVWAP`

### PriceReferenceGap

- 기본값: `reference_source={ type="MOVING_AVERAGE", ma_type="SMA", period=20 }`, `filter_type="GapAboveThreshold"`, `gap_threshold=0.02`, `consecutive_n=1`, `p=0`
- `gap_threshold` 검증 범위: `0.0..=1.0`
- `filter_type`:

```text
GapAboveThreshold, GapBelowThreshold,
GapAboveReferenceThreshold, GapBelowReferenceThreshold,
GapBelowReferenceUpperThreshold, GapAboveReferenceLowerThreshold
```

`reference_source`:

- `{ type = "MOVING_AVERAGE", ma_type = "EMA" | "SMA", period = N }`
- `{ type = "VWAP", period = N }`
- `{ type = "HIGHEST_HIGH", lookback_period = N, include_current_candle = true | false }`
- `{ type = "LOWEST_LOW", lookback_period = N, include_current_candle = true | false }`

메모:

- `include_current_candle` 기본값은 `true` 입니다. `false` 로 두면 기준 고가/저가 산출에서 현재 캔들을 제외하고, 최소 필요 캔들 수가 1개 늘어납니다.
- `reference_source.type = "MOVING_AVERAGE"` 에서는 `ma_type = "EMA" | "SMA"` 만 검증을 통과합니다. `WMA` 와 legacy shorthand `MA` 는 거부됩니다.
- `GapAboveThreshold` / `GapBelowThreshold` 는 **절대 괴리율** 기준입니다.
- `GapAboveReferenceThreshold` 는 `gap_ratio >= threshold` 입니다.
- `GapBelowReferenceThreshold` 는 `gap_ratio <= -threshold` 입니다.
- `GapBelowReferenceUpperThreshold` 는 `gap_ratio <= threshold` 입니다.
- `GapAboveReferenceLowerThreshold` 는 `gap_ratio >= -threshold` 입니다.
- `current_price` 는 필터 평가자가 외부에서 전달하는 실시간 가격입니다. 캔들 저장소의 최신 `close` 값을 현재가로 대체해서 쓰지 않습니다.
- `p` 는 reference window 와 지표/캔들 기준값 선택에만 영향을 줍니다. `p` 값이 바뀌어도 가격 비교에 쓰는 `current_price` 자체는 이동하지 않습니다.

`gap_ratio` 계산식:

```text
gap_ratio = (current_price - reference_price) / reference_price
```

여기서 `reference_price` 는 `p` 와 `reference_source` 로 선택한 캔들/지표 기준값이고, `current_price` 는 호출자가 매 평가마다 전달하는 외부 현재가입니다.

`filter_type` 판단식:

| filter_type | 조건 | 의미 |
| --- | --- | --- |
| `GapAboveThreshold` | `abs(gap_ratio) >= threshold` | 기준가와의 괴리율이 임계값 이상 |
| `GapBelowThreshold` | `abs(gap_ratio) <= threshold` | 기준가와의 괴리율이 임계값 이하 |
| `GapAboveReferenceThreshold` | `gap_ratio >= threshold` | 기준가보다 임계값 이상 높음 |
| `GapBelowReferenceThreshold` | `gap_ratio <= -threshold` | 기준가보다 임계값 이상 낮음 |
| `GapBelowReferenceUpperThreshold` | `gap_ratio <= threshold` | 기준가 대비 상단 임계값 이하 |
| `GapAboveReferenceLowerThreshold` | `gap_ratio >= -threshold` | 기준가 대비 하단 임계값 이상 |

방향별 threshold 안쪽 범위를 확인하려면 필터를 조합합니다.

- 기준가 이상이면서 상단 `threshold` 이내: `GapAboveReferenceThreshold(gap_threshold=0.0)` + `GapBelowReferenceUpperThreshold(gap_threshold=N)`
- 기준가 이하이면서 하단 `threshold` 이내: `GapBelowReferenceThreshold(gap_threshold=0.0)` + `GapAboveReferenceLowerThreshold(gap_threshold=N)`

예시:

```toml
[[filters]]
type = "PRICE_REFERENCE_GAP"
filter_type = "GapAboveReferenceThreshold"
gap_threshold = 0.0
consecutive_n = 1
reference_source = { type = "MOVING_AVERAGE", ma_type = "EMA", period = 20 }

[[filters]]
type = "PRICE_REFERENCE_GAP"
filter_type = "GapBelowThreshold"
gap_threshold = 0.02
consecutive_n = 1
reference_source = { type = "MOVING_AVERAGE", ma_type = "EMA", period = 20 }

# 기준가 이상이면서 +5% 이내
[[filters]]
type = "PRICE_REFERENCE_GAP"
filter_type = "GapAboveReferenceThreshold"
gap_threshold = 0.0
consecutive_n = 1
reference_source = { type = "MOVING_AVERAGE", ma_type = "EMA", period = 20 }

[[filters]]
type = "PRICE_REFERENCE_GAP"
filter_type = "GapBelowReferenceUpperThreshold"
gap_threshold = 0.05
consecutive_n = 1
reference_source = { type = "MOVING_AVERAGE", ma_type = "EMA", period = 20 }
```

### CopyS

- 기본값: `rsi_period=14`, `rsi_upper=70.0`, `rsi_lower=30.0`, `filter_type="BasicBuySignal"`, `consecutive_n=1`, `p=0`, `bband_period=20`, `bband_multiplier=2.0`, `ma_periods=[5,20,60,120,200,240]`
- 최소 필요 캔들 수: `60`
- `filter_type`:

```text
BasicBuySignal, BasicSellSignal, RSIOversold, RSIOverbought, BBandLowerTouch,
BBandUpperTouch, MASupport, MAResistance, StrongBuySignal, StrongSellSignal,
WeakBuySignal, WeakSellSignal, RSINeutral, BBandInside,
MARegularArrangement, MAReverseArrangement
```

메모: CopyS 는 내부 MA 컨텍스트를 현재 **EMA 고정**으로 사용합니다.

### ATR

- 기본값: `period=14`, `threshold=0.01`, `filter_type="AboveThreshold"`, `consecutive_n=1`, `p=0`
- 최소 필요 캔들 수: `max(period, consecutive_n)`
- `filter_type`: `AboveThreshold`, `VolatilityExpanding`, `VolatilityContracting`, `HighVolatility`, `LowVolatility`, `VolatilityIncreasing`, `VolatilityDecreasing`

메모:

- `AboveThreshold`, `HighVolatility`, `LowVolatility` 는 현재 ATR 값과 `threshold` 를 비교합니다.
- expanding/contracting/increasing/decreasing 계열은 이전 분석 결과와 비교하므로 충분한 히스토리가 없으면 `false` 입니다.

### SuperTrend

- 기본값: `period=10`, `multiplier=3.0`, `filter_type="AllUptrend"`, `consecutive_n=1`, `p=0`
- 최소 필요 캔들 수: `max(period, consecutive_n)`
- `filter_type`: `AllUptrend`, `AllDowntrend`, `PriceAboveSupertrend`, `PriceBelowSupertrend`, `PriceCrossingAbove`, `PriceCrossingBelow`, `TrendChanged`, `Uptrend`, `Downtrend`

### Volume

- 기본값: `period=20`, `threshold=1.5`, `filter_type="VolumeAboveAverage"`, `consecutive_n=1`, `p=0`, `stable_min_threshold=0.1`
- 최소 필요 캔들 수: `max(period, consecutive_n)`
- `filter_type`:

```text
VolumeAboveAverage, VolumeBelowAverage, VolumeSurge, VolumeDecline,
VolumeSignificantlyAbove, BullishWithIncreasedVolume, BearishWithIncreasedVolume,
IncreasingVolumeInUptrend, DecreasingVolumeInDowntrend, VolumeSharpDecline,
VolumeStable, VolumeVolatile, BullishWithDecreasedVolume,
BearishWithDecreasedVolume, VolumeDoubleAverage, VolumeHalfAverage,
VolumeConsecutiveIncrease, VolumeConsecutiveDecrease, VolumeSideways,
VolumeExtremelyHigh, VolumeExtremelyLow
```

메모:

- `VolumeStable` 은 `threshold` 와 `stable_min_threshold` 중 큰 값을 사용합니다.
- `VolumeSharpDecline` 은 현재 `VolumeDecline` 과 같습니다.
- `VolumeVolatile` 은 현재 `VolumeSurge` 와 같습니다.

### ThreeRSI

- 기본값: `rsi_periods=[7,14,21]`, `ma_type="SMA"`, `ma_period=20`, `adx_period=14`, `filter_type="AllRSILessThan50"`, `consecutive_n=1`, `p=0`, `cross_threshold=50.0`
- 최소 필요 캔들 수: `max(ma_period, consecutive_n)`
- `filter_type`:

```text
AllRSILessThan50, AllRSIGreaterThan50, RSIReverseArrangement,
RSIRegularArrangement, CandleLowBelowMA, CandleHighAboveMA, ADXGreaterThan20,
AllRSILessThan30, AllRSIGreaterThan70, RSIStableRange, RSIBullishRange,
RSIBearishRange, RSIOverboughtRange, RSIOversoldRange, RSICrossAbove,
RSICrossBelow, RSISideways, RSIBullishMomentum, RSIBearishMomentum,
RSIDivergence, RSIConvergence, RSIDoubleBottom, RSIDoubleTop,
RSIOverboughtReversal, RSIOversoldReversal, RSINeutralTrend,
RSIExtremeOverbought, RSIExtremeOversold
```

메모:

- 런타임에서 `ma_type` 은 `EMA`, `WMA` 를 명시하면 그 값으로 사용하고, 그 외 문자열은 `SMA` 로 처리합니다.
- `ma_type` 은 enum 이 아니라 문자열 필드입니다. 오타도 검증 실패가 아니라 SMA fallback 으로 이어질 수 있으므로 설정 리뷰 시 주의하세요.
- 여러 고급 이름이 현재는 `regular_arrangement` / `reverse_arrangement` / `sideways` 같은 기존 체크를 재사용합니다.

### CandlePattern

- 기본값: `min_body_ratio=0.3`, `min_shadow_ratio=0.3`, `pattern_history_length=5`, `threshold=0.5`, `filter_type="StrongBullishPattern"`, `consecutive_n=1`, `p=0`
- 최소 필요 캔들 수: `max(pattern_history_length, consecutive_n)`
- `pattern_history_length` 는 0일 수 없습니다.
- `filter_type` (41개):

```text
StrongBullishPattern, StrongBearishPattern, ReversalPattern, ContinuationPattern,
VolumeConfirmedPattern, HighReliabilityPattern, ContextAlignedPattern,
StrongReversalSignal, HighConfidenceSignal, VolumeConfirmedSignal,
PatternClusteringSignal, HammerPattern, ShootingStarPattern, DojiPattern,
SpinningTopPattern, MarubozuPattern, MorningStarPattern, EveningStarPattern,
EngulfingPattern, PiercingPattern, DarkCloudPattern, HaramiPattern,
TweezerPattern, TriStarPattern, AdvanceBlockPattern, DeliberanceBlockPattern,
BreakawayPattern, ConcealmentPattern, CounterattackPattern,
DarkCloudCoverPattern, RisingWindowPattern, FallingWindowPattern,
HighBreakoutPattern, LowBreakoutPattern, GapPattern, GapFillPattern,
DoubleBottomPattern, DoubleTopPattern, TrianglePattern, FlagPattern,
PennantPattern
```

메모: 여러 고급 패턴 이름이 현재는 continuation/reversal/strong bullish/strong bearish 같은 공통 신호를 재사용합니다.

### SupportResistance

- 기본값: `lookback_period=20`, `touch_threshold=0.01`, `min_touch_count=2`, `threshold=0.05`, `filter_type="SupportBreakdown"`, `consecutive_n=1`, `p=0`
- 최소 필요 캔들 수: `max(lookback_period, consecutive_n)`
- `min_touch_count` 는 0일 수 없습니다.
- `filter_type`: `SupportBreakdown`, `ResistanceBreakout`, `SupportBounce`, `ResistanceRejection`, `NearStrongSupport`, `NearStrongResistance`, `AboveSupport`, `BelowResistance`, `NearSupport`, `NearResistance`

### Momentum

- 기본값: `rsi_period=14`, `stoch_period=14`, `williams_period=14`, `roc_period=10`, `cci_period=20`, `momentum_period=10`, `history_length=50`, `threshold=0.5`, `filter_type="StrongPositiveMomentum"`, `consecutive_n=1`, `p=0`
- 최소 필요 캔들 수: `max(history_length, consecutive_n)`
- `filter_type`:

```text
StrongPositiveMomentum, StrongNegativeMomentum, AcceleratingMomentum,
DeceleratingMomentum, Overbought, Oversold, MomentumDivergence,
BullishDivergence, BearishDivergence, PersistentMomentum, StableMomentum,
MomentumReversalSignal, MomentumSideways, MomentumSurge, MomentumCrash,
MomentumConvergence, MomentumDivergencePattern, MomentumParallel,
MomentumCrossover, MomentumSupportTest, MomentumResistanceTest
```

메모: 여러 이름이 현재는 같은 analyzer 체크를 재사용합니다. 예를 들어 `MomentumSurge` 는 `StrongPositiveMomentum`, `MomentumCrash` 는 `StrongNegativeMomentum` 과 같은 구현입니다.

RSI 계산 메모:

- Momentum 내부 RSI 는 `src/indicator/rsi.rs` 의 `RSIBuilder` 를 재사용합니다.
- 따라서 단순 window 평균이 아니라 Wilder smoothing 기반 RSI 로 계산됩니다.
- `history_length` 는 MomentumAnalyzer 히스토리 길이 guard 이고, 개별 하위 지표(`rsi_period`, `stoch_period`, `roc_period` 등)의 warm-up 과는 별개입니다.

### Donchian

- `type="DONCHIAN"`
- 기본값: `period=20`, `filter_type="BreakoutAbove"`, `consecutive_n=1`, `p=0`, `edge_threshold=0.1`
- `filter_type`: `BreakoutAbove`, `BreakoutBelow`, `InsideChannel`, `AboveMiddle`, `BelowMiddle`, `NearUpperEdge`, `NearLowerEdge`
- 가격 비교는 외부 `current_price` 기준이며 breakout은 이전 Donchian channel 경계와 비교합니다.

### Keltner

- `type="KELTNER"`
- 기본값: `period=20`, `multiplier=2.0`, `filter_type="BreakoutAbove"`, `consecutive_n=1`, `p=0`, `edge_threshold=0.1`
- `filter_type`: `BreakoutAbove`, `BreakoutBelow`, `InsideChannel`, `AboveMiddle`, `BelowMiddle`, `NearUpperEdge`, `NearLowerEdge`
- 가격 비교는 외부 `current_price` 기준이며 breakout은 이전 Keltner channel 경계와 비교합니다.

### OBV

- `type="OBV"`
- 기본값: `filter_type="Rising"`, `consecutive_n=1`, `p=0`
- `filter_type`: `Rising`, `Falling`

### MFI

- `type="MFI"`
- 기본값: `period=14`, `filter_type="Overbought"`, `consecutive_n=1`, `p=0`, `overbought=80.0`, `oversold=20.0`, `threshold=50.0`
- `filter_type`: `Overbought`, `Oversold`, `AboveThreshold`, `BelowThreshold`

### Aroon

- `type="AROON"`
- 기본값: `period=25`, `filter_type="BullishTrend"`, `consecutive_n=1`, `p=0`, `strong_threshold=70.0`, `weak_threshold=30.0`
- `filter_type`: `BullishTrend`, `BearishTrend`

### Choppiness

- `type="CHOPPINESS"`
- 기본값: `period=14`, `filter_type="Trending"`, `consecutive_n=1`, `p=0`, `trending_threshold=38.2`, `ranging_threshold=61.8`
- `filter_type`: `Trending`, `Ranging`

### KAMA

- `type="KAMA"`
- 기본값: `period=10`, `fast_period=2`, `slow_period=30`, `filter_type="PriceAbove"`, `consecutive_n=1`, `p=0`, `er_threshold=0.6`, `er_low_threshold=0.2`
- `filter_type`: `PriceAbove`, `PriceBelow`, `PriceCrossAbove`, `PriceCrossBelow`, `Rising`, `Falling`, `ERAboveThreshold`, `ERBelowThreshold`
- 가격 위치/교차는 외부 `current_price` 기준으로 평가하고, 교차는 이전 캔들 close와 이전 KAMA 값을 비교합니다.

### Chaikin

- `type="CHAIKIN"`
- 기본값: `cmf_period=20`, `fast_period=3`, `slow_period=10`, `filter_type="CMFPositive"`, `consecutive_n=1`, `p=0`, `cmf_threshold=0.05`
- `filter_type`: `CMFPositive`, `CMFNegative`, `ADOSCPositive`, `ADOSCNegative`

### PPO

- `type="PPO"`
- 기본값: `fast_period=12`, `slow_period=26`, `signal_period=9`, `filter_type="AboveSignal"`, `consecutive_n=1`, `p=0`
- `filter_type`: `AboveSignal`, `BelowSignal`, `AboveZero`, `BelowZero`, `HistogramPositive`, `HistogramNegative`

### ParabolicSAR

- `type="PARABOLIC_SAR"`
- 기본값: `step=0.02`, `max_step=0.2`, `filter_type="Bullish"`, `consecutive_n=1`, `p=0`
- `filter_type`: `Bullish`, `Bearish`, `Reversal`, `PriceCrossAbove`, `PriceCrossBelow`
- 가격 교차는 외부 `current_price` 기준으로 평가하고, 이전 캔들 close와 이전 SAR 값을 비교합니다.

### Slope

- 기본값: `indicator_type=ClosePrice`, `period=20`, `filter_type="Upward"`, `consecutive_n=1`, `p=0`, `use_linear_regression=null`, `strength_threshold=null`, `r_squared_threshold=null`, `short_period=null`
- 최소 필요 캔들 수: `period + consecutive_n`
- `filter_type`: `Upward`, `Downward`, `Sideways`, `StrengthAboveThreshold`, `Accelerating`, `Decelerating`, `StrongUpward`, `StrongDownward`, `HighRSquared`

유효 기본값:

- `use_linear_regression`: 기본 `false`
- `strength_threshold`: 기본 `0.02` (`Upward`, `Downward`, `StrongUpward`, `StrongDownward`), 기본 `0.01` (`StrengthAboveThreshold`)
- `r_squared_threshold`: 기본 `0.7`
- `short_period`: 기본 `period / 2`

`indicator_type`:

```toml
{ type = "ClosePrice" }
{ type = "HighPrice" }
{ type = "LowPrice" }
{ type = "MovingAverage", ma_type = "EMA", period = 20 }
{ type = "RSI", period = 14, ma_type = "SMA", ma_periods = [14] }
{ type = "MACD", fast_period = 12, slow_period = 26, signal_period = 9 }
{ type = "MACDLine", fast_period = 12, slow_period = 26, signal_period = 9 }
{ type = "MACDSignalLine", fast_period = 12, slow_period = 26, signal_period = 9 }
{ type = "MACDHistogram", fast_period = 12, slow_period = 26, signal_period = 9 }
```

메모:

- `indicator_type = { type = "RSI", ... }` 는 `period` 만으로는 부족하고 `ma_type`, `ma_periods` 도 필요합니다.
- 현재 구현에서 `consecutive_n` 은 주로 상위 최소 캔들 수 계산에만 반영되고, 각 `filter_type` 판단식에는 직접 쓰이지 않는 경우가 많습니다.
- `filter_type` 정수 인덱스도 지원하지만, 다른 필터와 마찬가지로 문자열 사용을 권장합니다.

---

## 4. 실전 조합 팁

### 현재가 기반 돌파/이탈 필터

외부 `current_price` 로 평가되는 필터는 같은 캔들 상태에서도 tick 가격만 바꿔 재평가할 수 있습니다.

대표 예시:

- `PRICE_REFERENCE_GAP`: 모든 gap 판단
- `BOX_RANGE`: `InsideBox`, `OutsideBox`, `AboveUpperBox`, `BelowLowerBox`, `BreakoutAbove`, `BreakoutBelow`
- `MOVING_AVERAGE`: 가격과 MA 비교 계열
- `VWAP`: 가격과 VWAP 비교 계열
- `SUPPORTRESISTANCE`: support/resistance 근접·돌파 계열
- `DONCHIAN`, `KELTNER`, `KAMA`, `PARABOLIC_SAR`, `SUPERTREND`: 가격 위치/교차 계열
- `COPYS`, `THREERSI`, `VOLUME`: 일부 variant 에서 현재가를 캔들 open/MA/복합 신호와 비교

반대로 `MFI`, `OBV`, `AROON`, `CHOPPINESS`, `CHAIKIN`, `PPO`, `ATR`, 대부분의 `MOMENTUM` 처럼 지표 값 자체만 비교하는 필터는 `current_price` 를 받더라도 직접 사용하지 않는 타입이 많습니다.

### AND 조합으로 범위 만들기

필터 배열은 일반적으로 모두 만족해야 통과하는 AND 조건으로 쓰입니다. 예를 들어 “EMA 20 위이지만 +5% 이내”는 아래처럼 두 필터를 함께 둡니다.

```toml
[[filters]]
type = "PRICE_REFERENCE_GAP"
filter_type = "GapAboveReferenceThreshold"
gap_threshold = 0.0
reference_source = { type = "MOVING_AVERAGE", ma_type = "EMA", period = 20 }

[[filters]]
type = "PRICE_REFERENCE_GAP"
filter_type = "GapBelowReferenceUpperThreshold"
gap_threshold = 0.05
reference_source = { type = "MOVING_AVERAGE", ma_type = "EMA", period = 20 }
```

### `consecutive_n` 과 교차 필터

교차 필터는 “현재와 직전”을 비교합니다. `consecutive_n > 1` 로 설정하면 각 offset 지점마다 교차 조건을 반복 확인하기 때문에, 일반적인 “최근 한 번 교차 발생” 감지보다 훨씬 엄격해질 수 있습니다. 단발 교차 이벤트를 찾을 때는 보통 `consecutive_n = 1` 로 시작하세요.

### 다중 필터 평가의 실패 처리

`TechnicalFilter::matches_filters` 와 `TechnicalFilterContext::matches_filters` 는 배열의 모든 필터가 통과해야 `true` 를 반환합니다. 개별 필터가 `false` 이거나, 검증/실행 중 에러가 발생하면 로그를 남기고 전체 결과를 `Ok(false)` 로 반환합니다. 즉 다중 필터 평가는 **fail-closed** 동작이며, 에러를 호출자에게 그대로 전파하지 않습니다.

개별 필터 하나만 평가하는 `matches_filter` 계열은 `filter.validate()?` 를 먼저 수행하므로 잘못된 설정이면 `Err` 를 반환할 수 있습니다.

### 같은 구현을 공유하는 이름들

일부 `filter_type` 이름은 의미를 넓게 표현하기 위해 별도 enum variant 로 존재하지만, 현재 구현은 같은 체크를 재사용합니다.

| 필터 | 같은 구현을 공유하는 예 |
| --- | --- |
| BollingerBand | `BreakThroughLowerBand`, `BreakThroughLowerBandFromBelow` |
| VWAP | `StrongUptrend = PriceAboveVWAP`, `StrongDowntrend = PriceBelowVWAP`, `TrendStrengthening = DivergingFromVWAP`, `TrendWeakening = ConvergingToVWAP` |
| Volume | `VolumeSharpDecline = VolumeDecline`, `VolumeVolatile = VolumeSurge` |
| Ichimoku | `StrongBuySignal = BuySignal` |
| Momentum | `MomentumSurge = StrongPositiveMomentum`, `MomentumCrash = StrongNegativeMomentum` |

---

## 5. 구현상 주의할 점

1. `technical_filter.md` 에서 과거에 사용하던 일부 count/설명은 실제 코드와 달랐습니다. 이 문서는 코드 기준으로 다시 맞춘 버전입니다.
2. sample 문서(`ta_filter_sample/`)는 `filter_type` 에 숫자를 쓰는 경우가 많지만, 이 문서는 enum 문자열 기준으로 설명합니다.
3. `PriceReferenceGap` 은 절대 괴리와 방향성 괴리가 섞여 있으므로 이름을 정확히 구분해서 써야 합니다.
4. `MovingAverage`, `CopyS`, `ThreeRSI`, `CandlePattern`, `Momentum`, `VWAP` 은 일부 enum 이름이 내부적으로 같은 체크를 공유합니다.
5. `TechnicalFilter::matches_filter`, `TechnicalFilter::matches_filters`, 각 필터의 내부 `matches_filter` 는 외부 `current_price` 를 마지막 인자로 받습니다. `filter_*` 래퍼는 모듈별 추가 인자 때문에 순서가 다를 수 있지만, `matches_filter` 계열 API 에서는 마지막 인자 규칙을 유지합니다.
6. 같은 캔들 상태를 반복 평가할 때는 `TechnicalFilterContext` 로 `CandleStore` 기반 상태를 유지하고, 매 tick 마다 바뀌는 `current_price` 만 전달해서 재평가할 수 있습니다.
7. `type` 과 `filter_type` 은 대소문자/alias 허용 범위가 서로 다릅니다. 설정 파일에는 이 문서의 권장 `type` 과 문자열 enum 이름을 쓰는 것이 안전합니다.
8. 새 필터 타입이 추가되면 **반드시 `src/filter/mod.rs` 와 실제 `src/filter/*.rs` 구현을 함께 기준으로 문서를 갱신**해야 합니다.

---

## 6. 샘플 조합 문서

실전 조합 예시는 아래 문서를 참고하세요.

- `ta_filter_sample/ta_filter_sample01.md`
- `ta_filter_sample/ta_filter_sample02.md`
- `ta_filter_sample/ta_filter_sample04.md`
- `ta_filter_sample/ta_filter_simple02.md`
