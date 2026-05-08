/// 기울기 방향
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SlopeDirection {
    /// 상승 기울기
    Upward,
    /// 하락 기울기
    Downward,
    /// 횡보 (기울기가 거의 없음)
    Sideways,
}

/// 기울기 분석 데이터
#[derive(Debug, Clone)]
pub struct SlopeAnalysis {
    /// 기울기 값 (양수: 상승, 음수: 하락)
    pub slope: f64,
    /// 기울기 방향
    pub direction: SlopeDirection,
    /// 기울기 강도 (절대값)
    pub strength: f64,
    /// 선형 회귀의 결정계수 (R²) - 기울기의 신뢰도 (0.0 ~ 1.0)
    pub r_squared: f64,
    /// 시작 값
    pub start_value: f64,
    /// 종료 값
    pub end_value: f64,
    /// 분석 기간
    pub period: usize,
}

impl SlopeAnalysis {
    /// 새 기울기 분석 결과 생성
    pub fn new(
        slope: f64,
        r_squared: f64,
        start_value: f64,
        end_value: f64,
        period: usize,
        threshold: f64,
    ) -> Self {
        let strength = slope.abs();
        let direction = if strength < threshold {
            SlopeDirection::Sideways
        } else if slope > 0.0 {
            SlopeDirection::Upward
        } else {
            SlopeDirection::Downward
        };

        SlopeAnalysis {
            slope,
            direction,
            strength,
            r_squared,
            start_value,
            end_value,
            period,
        }
    }

    /// 기울기가 상승인지 확인
    pub fn is_upward(&self) -> bool {
        matches!(self.direction, SlopeDirection::Upward)
    }

    /// 기울기가 하락인지 확인
    pub fn is_downward(&self) -> bool {
        matches!(self.direction, SlopeDirection::Downward)
    }

    /// 기울기가 횡보인지 확인
    pub fn is_sideways(&self) -> bool {
        matches!(self.direction, SlopeDirection::Sideways)
    }
}

/// 최신 값이 앞에 있는 시계열에서 선형 회귀 기울기를 계산합니다.
pub fn calculate_linear_regression_slope(
    values_newest_first: &[f64],
    period: usize,
    offset: usize,
) -> Option<SlopeAnalysis> {
    if values_newest_first.len() < period + offset {
        return None;
    }

    let values: Vec<f64> = values_newest_first
        .iter()
        .skip(offset)
        .take(period)
        .copied()
        .rev()
        .collect();

    if values.len() < period {
        return None;
    }

    let start_value = *values.first().unwrap();
    let end_value = *values.last().unwrap();

    let n = values.len() as f64;
    let sum_x: f64 = (0..values.len()).map(|i| i as f64).sum();
    let sum_y: f64 = values.iter().sum();
    let sum_xy: f64 = values.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
    let sum_x2: f64 = (0..values.len()).map(|i| (i as f64).powi(2)).sum();

    let mean_x = sum_x / n;
    let mean_y = sum_y / n;

    let slope = if sum_x2 - n * mean_x * mean_x != 0.0 {
        (sum_xy - n * mean_x * mean_y) / (sum_x2 - n * mean_x * mean_x)
    } else {
        0.0
    };

    let ss_res: f64 = values
        .iter()
        .enumerate()
        .map(|(i, &y)| {
            let predicted = mean_y + slope * (i as f64 - mean_x);
            (y - predicted).powi(2)
        })
        .sum();

    let ss_tot: f64 = values.iter().map(|&y| (y - mean_y).powi(2)).sum();

    let r_squared = if ss_tot != 0.0 {
        1.0 - (ss_res / ss_tot)
    } else {
        0.0
    };

    let threshold = (end_value.abs() * 0.01).max(0.0001);
    Some(SlopeAnalysis::new(
        slope,
        r_squared,
        start_value,
        end_value,
        period,
        threshold,
    ))
}

/// 최신 값이 앞에 있는 시계열에서 단순 차이 기반 기울기를 계산합니다.
pub fn calculate_simple_slope(
    values_newest_first: &[f64],
    period: usize,
    offset: usize,
) -> Option<SlopeAnalysis> {
    if values_newest_first.len() < period + offset {
        return None;
    }

    let start_value = *values_newest_first.get(offset + period - 1)?;
    let end_value = *values_newest_first.get(offset)?;

    let slope = (end_value - start_value) / period as f64;
    let strength = slope.abs();
    let threshold = (end_value.abs() * 0.01).max(0.0001);

    let direction = if strength < threshold {
        SlopeDirection::Sideways
    } else if slope > 0.0 {
        SlopeDirection::Upward
    } else {
        SlopeDirection::Downward
    };

    Some(SlopeAnalysis {
        slope,
        direction,
        strength,
        r_squared: 0.0,
        start_value,
        end_value,
        period,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_linear_regression_slope_from_newest_first_values() {
        let values = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        let analysis = calculate_linear_regression_slope(&values, 5, 0).unwrap();

        assert_eq!(analysis.start_value, 1.0);
        assert_eq!(analysis.end_value, 5.0);
        assert!((analysis.slope - 1.0).abs() < 1e-10);
        assert_eq!(analysis.direction, SlopeDirection::Upward);
    }

    #[test]
    fn calculates_simple_slope_with_offset() {
        let values = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        let analysis = calculate_simple_slope(&values, 3, 1).unwrap();

        assert_eq!(analysis.start_value, 4.0);
        assert_eq!(analysis.end_value, 8.0);
        assert!((analysis.slope - (4.0 / 3.0)).abs() < 1e-10);
    }
}
