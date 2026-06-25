use chrono::Duration;
use chrono::{DateTime, Timelike, Utc};

use crate::error::VmCliError;

pub struct TimeUtils;

impl TimeUtils {
    pub fn format_time_3339_opts(time: &DateTime<Utc>) -> String {
        time.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    }

    pub fn split_into_hourly_ranges(
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
    ) -> Result<Vec<(DateTime<Utc>, DateTime<Utc>)>, VmCliError> {
        if end < start {
            return Err(VmCliError::VlogVmCliError(
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
    ) -> Result<Vec<(DateTime<Utc>, DateTime<Utc>)>, VmCliError> {
        if end < start {
            return Err(VmCliError::VlogVmCliError(
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
    ) -> Result<(DateTime<Utc>, DateTime<Utc>), VmCliError> {
        if range.is_empty() {
            return Err(VmCliError::VlogVmCliError(
                "Cannot pass an empty list for obtaining boundary time".into(),
            ));
        }

        let start = range.first().ok_or("Start Error")?.0;
        let end = range.last().ok_or("End Error")?.1;
        Ok((start, end))
    }
}

#[cfg(test)]
mod time_test {

    use crate::utils::time_utils::TimeUtils;
    use chrono::{DateTime, Utc};

    #[test]
    fn test_hour() -> Result<(), Box<dyn std::error::Error>> {
        // 定义时间范围
        let start_str = "2025-08-12T00:00:00Z";
        let end_str = "2025-08-12T23:59:59Z";

        // 解析时间字符串
        let start = DateTime::parse_from_rfc3339(start_str)
            .expect("Invalid start time format")
            .with_timezone(&Utc);
        let end = DateTime::parse_from_rfc3339(end_str)
            .expect("Invalid end time format")
            .with_timezone(&Utc);

        let value = TimeUtils::split_into_hourly_ranges(&start, &end)?;
        for (start, end) in value {
            println!(
                "{} - {}",
                TimeUtils::format_time_3339_opts(&start),
                TimeUtils::format_time_3339_opts(&end)
            );
        }
        Ok(())
    }

    #[test]
    fn test_minute() -> Result<(), Box<dyn std::error::Error>> {
        // 定义时间范围
        let start_str = "2025-08-12T00:00:00Z";
        let end_str = "2025-08-12T23:59:59Z";

        // 解析时间字符串
        let start = DateTime::parse_from_rfc3339(start_str)
            .expect("Invalid start time format")
            .with_timezone(&Utc);
        let end = DateTime::parse_from_rfc3339(end_str)
            .expect("Invalid end time format")
            .with_timezone(&Utc);

        let value = TimeUtils::split_into_minute_ranges(&start, &end)?;
        for (start, end) in value {
            println!(
                "{} - {}",
                TimeUtils::format_time_3339_opts(&start),
                TimeUtils::format_time_3339_opts(&end)
            );
        }
        Ok(())
    }

    #[test]
    fn test_range() -> Result<(), Box<dyn std::error::Error>> {
        // 解析时间字符串
        let start = DateTime::parse_from_rfc3339("2025-08-18T16:00:00Z")
            .expect("Invalid start time format")
            .with_timezone(&Utc);
        let end = DateTime::parse_from_rfc3339("2025-08-18T16:10:00Z")
            .expect("Invalid end time format")
            .with_timezone(&Utc);

        let list = TimeUtils::split_time_range(&start, &end, 300);
        list.iter().for_each(|(s, e)| {
            println!("{}-{}", s, e);
        });
        Ok(())
    }
}
