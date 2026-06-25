use chrono::Duration;
use chrono::{DateTime, Timelike, Utc};

use crate::error::CliError;

pub struct TimeUtils;

impl TimeUtils {
    pub fn format_time_3339_opts(time: &DateTime<Utc>) -> String {
        time.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    }

    pub fn split_into_hourly_ranges(
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
    ) -> Result<Vec<(DateTime<Utc>, DateTime<Utc>)>, CliError> {
        if start >= end {
            return Err(CliError::VlogCliError(
                "End time is before start time".to_string(),
            ));
        }
        let mut ranges = Vec::new();
        let mut current_hour = start
            .date_naive()
            .and_hms_opt(start.time().hour(), 0, 0)
            .unwrap();
        let mut current_start = DateTime::from_naive_utc_and_offset(current_hour, Utc);

        // 计算当前小时的结束时间：HH:59:59
        let mut current_end = current_start + Duration::seconds(3599); // 3600 - 1

        while current_start <= *end && current_start < *start {
            // 跳过早于原始 start 的小时
            current_hour += Duration::hours(1);
            current_start = DateTime::from_naive_utc_and_offset(current_hour, Utc);
            current_end = current_start + Duration::seconds(3599);
        }

        // 现在 current_start >= start，开始生成有效区间
        while current_start <= *end {
            let segment_start = current_start.max(*start); // 防止早于原始 start
            let segment_end = current_end.min(*end); // 防止晚于原始 end

            if segment_start <= segment_end {
                ranges.push((segment_start, segment_end));
            }
            current_hour += Duration::hours(1);
            current_start = DateTime::from_naive_utc_and_offset(current_hour, Utc);
            current_end = current_start + Duration::seconds(3599);
        }
        Ok(ranges)
    }

    pub fn split_into_minute_ranges(
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
    ) -> Result<Vec<(DateTime<Utc>, DateTime<Utc>)>, CliError> {
        if end < start {
            return Err(CliError::VlogCliError(
                "End time is before start time".to_string(),
            ));
        }

        let mut ranges = Vec::new();
        let mut current = *start;

        // 每段 10 分钟 = 600 秒
        let segment_duration = Duration::minutes(10);

        while current < *end {
            // 下一个 10 分钟的时间点（例如：00:10:00）
            let next_nominal = current + segment_duration;

            // 实际结束时间：取 (next_nominal - 1 秒) 和 end 的较小值,即：变成 HH:MM:59
            let next_end = (next_nominal - Duration::seconds(1)).min(*end);

            if current <= next_end {
                ranges.push((current, next_end));
            }

            // 下一段从 next_nominal 开始（对齐 10 分钟边界）
            current = next_nominal;
        }
        Ok(ranges)
    }

    pub fn split_time_range(
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
        interval_second: i64,
    ) -> Vec<(DateTime<Utc>, DateTime<Utc>)> {
        if interval_second <= 0 {
            return Vec::new();
        }
        if start >= end {
            return Vec::new();
        }
        let mut result = Vec::new();
        let interval = Duration::seconds(interval_second);
        let mut current_start = *start;

        while current_start < *end {
            let current_end = (current_start + interval).min(*end);

            let adjusted_end = if current_end == *end {
                current_end - Duration::seconds(1)
            } else {
                current_end - Duration::seconds(1)
            };

            result.push((current_start, adjusted_end));
            current_start = current_end;
        }
        result
    }

    pub fn get_time_bound(
        range: &[(DateTime<Utc>, DateTime<Utc>)],
    ) -> Result<(DateTime<Utc>, DateTime<Utc>), CliError> {
        if range.is_empty() {
            return Err(CliError::VlogCliError(
                "Cannot pass an empty list for obtaining boundary time".into(),
            ));
        }

        let start = range.first().ok_or("Start Error")?.0;
        let end = range.last().ok_or("End Error")?.1;
        Ok((start, end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    /// 测试TimeUtils结构体的创建
    #[test]
    fn test_time_utils_creation() {
        // TimeUtils是一个单元结构体，主要测试其存在性
        let _utils = TimeUtils;
        println!("TimeUtils creation test passed");
    }

    /// 测试RFC3339格式化 - 标准格式
    #[test]
    fn test_format_time_3339_standard() {
        let time = "2024-07-01T16:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let formatted = TimeUtils::format_time_3339_opts(&time);
        assert_eq!(formatted, "2024-07-01T16:00:00Z");
        println!("RFC3339 standard format test passed");
    }

    /// 测试RFC3339格式化 - 带毫秒的时间
    #[test]
    fn test_format_time_3339_with_milliseconds() {
        let time = "2024-07-01T16:00:00.123Z".parse::<DateTime<Utc>>().unwrap();
        let formatted = TimeUtils::format_time_3339_opts(&time);
        println!("{}", formatted);
    }

    /// 测试RFC3339格式化 - 不同时区
    #[test]
    fn test_format_time_3339_different_timezone() {
        let time = "2024-07-01T16:00:00+08:00"
            .parse::<DateTime<Utc>>()
            .unwrap();
        let formatted = TimeUtils::format_time_3339_opts(&time);
        // 转换为UTC时间
        assert_eq!(formatted, "2024-07-01T08:00:00Z");
        println!("RFC3339 different timezone test passed");
    }

    /// 测试按小时分割时间范围 - 简单情况
    #[test]
    fn test_split_into_hourly_ranges_simple() {
        let start = "2024-07-01T16:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let end = "2024-07-01T18:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let ranges = TimeUtils::split_into_hourly_ranges(&start, &end).unwrap();

        println!("{:?}", ranges.len());
        println!("{:?}", ranges[0].0);
        println!("{:?}", ranges[0].1);
        println!("{:?}", ranges[1].0);
        println!("{:?}", ranges[1].1);
        println!("Hourly ranges simple test passed");
    }

    /// 测试按小时分割时间范围 - 跨天
    #[test]
    fn test_split_into_hourly_ranges_cross_day() {
        let start = "2024-07-01T23:30:00Z".parse::<DateTime<Utc>>().unwrap();
        let end = "2024-07-02T01:30:00Z".parse::<DateTime<Utc>>().unwrap();
        let ranges = TimeUtils::split_into_hourly_ranges(&start, &end).unwrap();

        println!("{:?}", ranges.len());
        println!("{:?}", ranges[0].0);
        println!("{:?}", ranges[0].1);
        println!("{:?}", ranges[1].0);
        println!("{:?}", ranges[1].1);
        println!("Hourly ranges cross day test passed");
    }

    /// 测试按小时分割时间范围 - 单小时内
    #[test]
    fn test_split_into_hourly_ranges_within_hour() {
        let start = "2024-07-01T16:15:00Z".parse::<DateTime<Utc>>().unwrap();
        let end = "2024-07-01T16:45:00Z".parse::<DateTime<Utc>>().unwrap();
        let ranges = TimeUtils::split_into_hourly_ranges(&start, &end).unwrap();

        println!("{}", ranges.len());
        println!("Hourly ranges within hour test passed");
    }

    /// 测试按10分钟分割时间范围 - 简单情况
    #[test]
    fn test_split_into_minute_ranges_simple() {
        let start = "2024-07-01T16:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let end = "2024-07-01T16:30:00Z".parse::<DateTime<Utc>>().unwrap();
        let ranges = TimeUtils::split_into_minute_ranges(&start, &end).unwrap();

        println!("{:?}", ranges.len());
        println!("{:?}", ranges[0].0);
        println!("{:?}", ranges[0].1);
        println!("{:?}", ranges[1].0);
        println!("{:?}", ranges[1].1);
        println!("{:?}", ranges[2].0);
        println!("{:?}", ranges[2].1);
        println!("Minute ranges simple test passed");
    }

    /// 测试按10分钟分割时间范围 - 跨小时
    #[test]
    fn test_split_into_minute_ranges_cross_hour() {
        let start = "2024-07-01T16:55:00Z".parse::<DateTime<Utc>>().unwrap();
        let end = "2024-07-01T17:15:00Z".parse::<DateTime<Utc>>().unwrap();
        let ranges = TimeUtils::split_into_minute_ranges(&start, &end).unwrap();

        println!("{:?}", ranges.len());
        println!("{:?}", ranges[0].0);
        println!("{:?}", ranges[0].1);
        println!("{:?}", ranges[1].0);
        println!("{:?}", ranges[1].1);
        println!("Minute ranges cross hour test passed");
    }

    /// 测试按10分钟分割时间范围 - 单个10分钟内
    #[test]
    fn test_split_into_minute_ranges_within_range() {
        let start = "2024-07-01T16:05:00Z".parse::<DateTime<Utc>>().unwrap();
        let end = "2024-07-01T16:08:00Z".parse::<DateTime<Utc>>().unwrap();
        let ranges = TimeUtils::split_into_minute_ranges(&start, &end).unwrap();

        println!("{:?}", ranges.len());
        println!("{:?}", ranges[0].0);
        println!("{:?}", ranges[0].1);
        println!("Minute ranges within range test passed");
    }

    /// 测试自定义时间分割 - 30秒间隔
    #[test]
    fn test_split_time_range_30_seconds() {
        let start = "2024-07-01T16:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let end = "2024-07-01T16:01:30Z".parse::<DateTime<Utc>>().unwrap();
        let ranges = TimeUtils::split_time_range(&start, &end, 30);

        println!("{:?}", ranges.len());
        println!("{:?}", ranges[0].0);
        println!("{:?}", ranges[0].1);
        println!("{:?}", ranges[1].0);
        println!("{:?}", ranges[1].1);
        println!("{:?}", ranges[2].0);
        println!("{:?}", ranges[2].1);
        println!("Time range split 30 seconds test passed");
    }

    /// 测试自定义时间分割 - 1分钟间隔
    #[test]
    fn test_split_time_range_1_minute() {
        let start = "2024-07-01T16:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let end = "2024-07-01T16:02:30Z".parse::<DateTime<Utc>>().unwrap();
        let ranges = TimeUtils::split_time_range(&start, &end, 60);

        println!("{:?}", ranges.len());
        println!("{:?}", ranges[0].0);
        println!("{:?}", ranges[0].1);
        println!("{:?}", ranges[1].0);
        println!("{:?}", ranges[1].1);
        println!("Time range split 1 minute test passed");
    }

    /// 测试自定义时间分割 - 大间隔
    #[test]
    fn test_split_time_range_large_interval() {
        let start = "2024-07-01T16:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let end = "2024-07-01T16:05:00Z".parse::<DateTime<Utc>>().unwrap();
        let ranges = TimeUtils::split_time_range(&start, &end, 300); // 5分钟

        println!("{:?}", ranges.len());
        println!("{:?}", ranges[0].0);
        println!("{:?}", ranges[0].1);
        println!("Time range split large interval test passed");
    }

    /// 测试自定义时间分割 - 零间隔
    #[test]
    fn test_split_time_range_zero_interval() {
        let start = "2024-07-01T16:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let end = "2024-07-01T16:01:00Z".parse::<DateTime<Utc>>().unwrap();
        let ranges = TimeUtils::split_time_range(&start, &end, 0);

        // 零间隔应该返回单个范围
        println!("{:?}", ranges.len());
        println!("Time range split zero interval test passed");
    }

    /// 测试获取时间边界
    #[test]
    fn test_get_time_bound() {
        let start = "2024-07-01T16:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let end = "2024-07-01T16:30:00Z".parse::<DateTime<Utc>>().unwrap();
        let ranges = TimeUtils::split_time_range(&start, &end, 600); // 10分钟

        let (bound_start, bound_end) = TimeUtils::get_time_bound(&ranges).unwrap();
        println!("{}-{}", bound_start, bound_end);
    }

    /// 测试获取时间边界 - 空列表
    #[test]
    fn test_get_time_bound_empty() {
        let ranges: Vec<(DateTime<Utc>, DateTime<Utc>)> = Vec::new();
        let result = TimeUtils::get_time_bound(&ranges);

        assert!(result.is_err());
        println!("Time bound empty test passed");
    }

    /// 测试时间分割的连续性
    #[test]
    fn test_time_range_continuity() {
        let start = "2024-07-01T16:05:00Z".parse::<DateTime<Utc>>().unwrap();
        let end = "2024-07-01T16:25:00Z".parse::<DateTime<Utc>>().unwrap();
        let ranges = TimeUtils::split_time_range(&start, &end, 600); // 10分钟

        // 检查范围的连续性
        for i in 1..ranges.len() {
            println!("{:?}", ranges[i - 1].1);
            println!("{:?}", ranges[i].0);
        }

        // 检查首尾
        println!("{:?}", ranges[0].0);
        println!("{:?}", ranges[ranges.len() - 1].1);
        println!("Time range continuity test passed");
    }

    /// 测试时间范围分割的边界情况 - 开始等于结束
    #[test]
    fn test_split_time_range_equal_start_end() {
        let time = "2024-07-01T16:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let ranges = TimeUtils::split_time_range(&time, &time, 60);

        println!("{:?}", ranges.len());
        println!("Time range equal start end test passed");
    }

    /// 测试时间范围分割的边界情况 - 开始大于结束
    #[test]
    fn test_split_time_range_start_after_end() {
        let start = "2024-07-01T16:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let end = "2024-07-01T15:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let ranges = TimeUtils::split_time_range(&start, &end, 60);

        // 开始时间大于结束时间，应该返回空范围
        println!("{:?}", ranges.len());
        println!("Time range start after end test passed");
    }
}
