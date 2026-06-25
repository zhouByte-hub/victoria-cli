use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DetectionResponse {
    #[serde(rename = "count(*)", deserialize_with = "parse_count_from_str")]
    pub count: u64,
}

fn parse_count_from_str<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    /// 测试DetectionResponse结构体的创建
    #[test]
    fn test_detection_response_creation() {
        let response = DetectionResponse { count: 42 };
        assert_eq!(response.count, 42);
        println!("DetectionResponse creation test passed");
    }

    /// 测试从JSON字符串解析DetectionResponse - 标准情况
    #[test]
    fn test_detection_response_from_json_standard() {
        let json = r#"{"count(*)": "42"}"#;
        let response: DetectionResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.count, 42);
        println!("DetectionResponse from JSON standard test passed");
    }

    /// 测试从JSON字符串解析DetectionResponse - 零值
    #[test]
    fn test_detection_response_from_json_zero() {
        let json = r#"{"count(*)": "0"}"#;
        let response: DetectionResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.count, 0);
        println!("DetectionResponse from JSON zero test passed");
    }

    /// 测试从JSON字符串解析DetectionResponse - 大数值
    #[test]
    fn test_detection_response_from_json_large_number() {
        let json = r#"{"count(*)": "999999999"}"#;
        let response: DetectionResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.count, 999999999);
        println!("DetectionResponse from JSON large number test passed");
    }

    /// 测试从JSON字符串解析DetectionResponse - 无效数值
    #[test]
    fn test_detection_response_from_json_invalid_number() {
        let json = r#"{"count(*)": "invalid"}"#;
        let result: Result<DetectionResponse, _> = serde_json::from_str(json);

        // 应该返回错误，因为"invalid"不是有效的u64数字
        if result.is_err() {
            println!("DetectionResponse from JSON invalid number test correctly returned error");
        } else {
            println!("DetectionResponse from JSON invalid number test unexpectedly succeeded");
        }
    }

    /// 测试从JSON字符串解析DetectionResponse - 负数值
    #[test]
    fn test_detection_response_from_json_negative_number() {
        let json = r#"{"count(*)": "-42"}"#;
        let result: Result<DetectionResponse, _> = serde_json::from_str(json);

        // 应该返回错误，因为u64不能为负数
        if result.is_err() {
            println!("DetectionResponse from JSON negative number test correctly returned error");
        } else {
            println!("DetectionResponse from JSON negative number test unexpectedly succeeded");
        }
    }

    /// 测试从JSON字符串解析DetectionResponse - 浮点数
    #[test]
    fn test_detection_response_from_json_float() {
        let json = r#"{"count(*)": "42.5"}"#;
        let result: Result<DetectionResponse, _> = serde_json::from_str(json);

        // 应该返回错误，因为u64不支持浮点数
        if result.is_err() {
            println!("DetectionResponse from JSON float test correctly returned error");
        } else {
            println!("DetectionResponse from JSON float test unexpectedly succeeded");
        }
    }

    /// 测试从JSON字符串解析DetectionResponse - 空字符串
    #[test]
    fn test_detection_response_from_json_empty_string() {
        let json = r#"{"count(*)": ""}"#;
        let result: Result<DetectionResponse, _> = serde_json::from_str(json);

        // 应该返回错误，因为空字符串不是有效的u64数字
        if result.is_err() {
            println!("DetectionResponse from JSON empty string test correctly returned error");
        } else {
            println!("DetectionResponse from JSON empty string test unexpectedly succeeded");
        }
    }

    /// 测试从JSON字符串解析DetectionResponse - 缺少count字段
    #[test]
    fn test_detection_response_from_json_missing_field() {
        let json = r#"{"other_field": "value"}"#;
        let result: Result<DetectionResponse, _> = serde_json::from_str(json);

        // 应该返回错误，因为缺少必需的count字段
        if result.is_err() {
            println!("DetectionResponse from JSON missing field test correctly returned error");
        } else {
            println!("DetectionResponse from JSON missing field test unexpectedly succeeded");
        }
    }

    /// 测试从JSON字符串解析DetectionResponse - 额外字段
    #[test]
    fn test_detection_response_from_json_extra_fields() {
        let json = r#"{"count(*)": "42", "extra_field": "value"}"#;
        let response: DetectionResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.count, 42);
        // 额外字段应该被忽略
        println!("DetectionResponse from JSON extra fields test passed");
    }

    /// 测试DetectionResponse序列化为JSON
    #[test]
    fn test_detection_response_to_json() {
        let response = DetectionResponse { count: 42 };
        let json = serde_json::to_string(&response).unwrap();

        // 注意：序列化时count字段会直接输出为数字，而不是字符串
        assert_eq!(json, r#"{"count(*)":42}"#);
        println!("DetectionResponse to JSON test passed");
    }

    /// 测试DetectionResponse的Debug trait实现
    #[test]
    fn test_detection_response_debug() {
        let response = DetectionResponse { count: 42 };
        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("DetectionResponse"));
        assert!(debug_str.contains("count: 42"));
        println!("DetectionResponse debug test passed");
    }

    /// 测试DetectionResponse的Clone trait实现
    #[test]
    fn test_detection_response_clone() {
        let response1 = DetectionResponse { count: 42 };
        let response2 = response1.clone();

        assert_eq!(response1.count, response2.count);
        println!("DetectionResponse clone test passed");
    }

    /// 测试DetectionResponse的PartialEq trait实现
    #[test]
    fn test_detection_response_partial_eq() {
        let response1 = DetectionResponse { count: 42 };
        let response2 = DetectionResponse { count: 42 };
        let response3 = DetectionResponse { count: 43 };

        assert_eq!(response1, response2);
        assert_ne!(response1, response3);
        println!("DetectionResponse partial eq test passed");
    }

    /// 测试parse_count_from_str函数 - 有效数值
    #[test]
    fn test_parse_count_from_str_valid() {
        let count_str = "\"42\""; // 字符串格式的数字
        let mut deserializer = serde_json::Deserializer::from_str(count_str);
        let result = parse_count_from_str(&mut deserializer).unwrap();
        assert_eq!(result, 42);
        println!("parse_count_from_str valid test passed");
    }

    /// 测试parse_count_from_str函数 - 零值
    #[test]
    fn test_parse_count_from_str_zero() {
        let count_str = "\"0\""; // 字符串格式的数字
        let mut deserializer = serde_json::Deserializer::from_str(count_str);
        let result = parse_count_from_str(&mut deserializer).unwrap();
        assert_eq!(result, 0);
        println!("parse_count_from_str zero test passed");
    }

    /// 测试parse_count_from_str函数 - 最大u64值
    #[test]
    fn test_parse_count_from_str_max_u64() {
        let count_str = "\"18446744073709551615\""; // 字符串格式的数字
        let mut deserializer = serde_json::Deserializer::from_str(count_str);
        let result = parse_count_from_str(&mut deserializer).unwrap();
        assert_eq!(result, std::u64::MAX);
        println!("parse_count_from_str max u64 test passed");
    }

    /// 测试parse_count_from_str函数 - 无效数值
    #[test]
    fn test_parse_count_from_str_invalid() {
        let count_str = "invalid";
        let mut deserializer = serde_json::Deserializer::from_str(count_str);
        let result = parse_count_from_str(&mut deserializer);

        // 应该返回错误
        if result.is_err() {
            println!("parse_count_from_str invalid test correctly returned error");
        } else {
            println!("parse_count_from_str invalid test unexpectedly succeeded");
        }
    }

    /// 测试parse_count_from_str函数 - 负数值
    #[test]
    fn test_parse_count_from_str_negative() {
        let count_str = "-42";
        let mut deserializer = serde_json::Deserializer::from_str(count_str);
        let result = parse_count_from_str(&mut deserializer);

        // 应该返回错误
        if result.is_err() {
            println!("parse_count_from_str negative test correctly returned error");
        } else {
            println!("parse_count_from_str negative test unexpectedly succeeded");
        }
    }

    /// 测试parse_count_from_str函数 - 浮点数
    #[test]
    fn test_parse_count_from_str_float() {
        let count_str = "42.5";
        let mut deserializer = serde_json::Deserializer::from_str(count_str);
        let result = parse_count_from_str(&mut deserializer);

        // 应该返回错误
        if result.is_err() {
            println!("parse_count_from_str float test correctly returned error");
        } else {
            println!("parse_count_from_str float test unexpectedly succeeded");
        }
    }

    /// 测试parse_count_from_str函数 - 空字符串
    #[test]
    fn test_parse_count_from_str_empty() {
        let count_str = "";
        let mut deserializer = serde_json::Deserializer::from_str(count_str);
        let result = parse_count_from_str(&mut deserializer);

        // 应该返回错误
        if result.is_err() {
            println!("parse_count_from_str empty test correctly returned error");
        } else {
            println!("parse_count_from_str empty test unexpectedly succeeded");
        }
    }

    /// 测试parse_count_from_str函数 - 超出u64范围的数值
    #[test]
    fn test_parse_count_from_str_overflow() {
        let count_str = "18446744073709551616"; // u64::MAX + 1
        let mut deserializer = serde_json::Deserializer::from_str(count_str);
        let result = parse_count_from_str(&mut deserializer);

        // 应该返回错误
        if result.is_err() {
            println!("parse_count_from_str overflow test correctly returned error");
        } else {
            println!("parse_count_from_str overflow test unexpectedly succeeded");
        }
    }

    /// 测试parse_count_from_str函数 - 前导零
    #[test]
    fn test_parse_count_from_str_leading_zeros() {
        let count_str = "\"00042\""; // 字符串格式的数字
        let mut deserializer = serde_json::Deserializer::from_str(count_str);
        let result = parse_count_from_str(&mut deserializer).unwrap();
        assert_eq!(result, 42);
        println!("parse_count_from_str leading zeros test passed");
    }

    /// 测试parse_count_from_str函数 - 十六进制数值
    #[test]
    fn test_parse_count_from_str_hex() {
        let count_str = "0x2A";
        let mut deserializer = serde_json::Deserializer::from_str(count_str);
        let result = parse_count_from_str(&mut deserializer);

        // 应该返回错误，因为函数不支持十六进制
        if result.is_err() {
            println!("parse_count_from_str hex test correctly returned error");
        } else {
            println!("parse_count_from_str hex test unexpectedly succeeded");
        }
    }
}
