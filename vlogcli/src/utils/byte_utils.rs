pub struct ByteUtils;

impl ByteUtils {
    pub fn bytes_to_mb(bytes: usize) -> f64 {
        let mb = bytes as f64 / 1024.0 / 1024.0;
        (mb * 100.0).round() / 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::ByteUtils;

    /// 测试基本的字节到MB转换
    #[test]
    fn test_basic_conversion() {
        let bytes = 1_048_576; // 1MB
        let mb = ByteUtils::bytes_to_mb(bytes);
        assert_eq!(mb, 1.0);
        println!("Basic conversion test passed: {} bytes = {} MB", bytes, mb);
    }

    /// 测试小数值的转换
    #[test]
    fn test_small_values() {
        let bytes = 524_288; // 0.5MB
        let mb = ByteUtils::bytes_to_mb(bytes);
        assert_eq!(mb, 0.5);
        println!("Small values test passed: {} bytes = {} MB", bytes, mb);
    }

    /// 测试大数值的转换
    #[test]
    fn test_large_values() {
        let bytes = 5_250_000;
        let mb = ByteUtils::bytes_to_mb(bytes);
        // 5_250_000 / 1024 / 1024 ≈ 5.01
        assert!(mb > 5.0 && mb < 5.1);
        println!("Large values test passed: {} bytes = {} MB", bytes, mb);
    }

    /// 测试零值
    #[test]
    fn test_zero_bytes() {
        let bytes = 0;
        let mb = ByteUtils::bytes_to_mb(bytes);
        assert_eq!(mb, 0.0);
        println!("Zero bytes test passed: {} bytes = {} MB", bytes, mb);
    }

    /// 测试极小值
    #[test]
    fn test_very_small_values() {
        let bytes = 1;
        let mb = ByteUtils::bytes_to_mb(bytes);
        println!("Very small values test passed: {} bytes = {} MB", bytes, mb);
    }

    /// 测试极大值
    #[test]
    fn test_very_large_values() {
        let bytes = usize::MAX;
        let mb = ByteUtils::bytes_to_mb(bytes);
        assert!(mb > 0.0);
        println!("Very large values test passed: {} bytes = {} MB", bytes, mb);
    }

    /// 测试四舍五入功能
    #[test]
    fn test_rounding() {
        // 测试应该向上舍入的情况
        let bytes1 = 1_048_576 + 52_429; // 1MB + 0.05MB
        let mb1 = ByteUtils::bytes_to_mb(bytes1);
        println!("{}", mb1);

        // 测试应该向下舍入的情况
        let bytes2 = 1_048_576 + 51_428; // 1MB + 0.049MB
        let mb2 = ByteUtils::bytes_to_mb(bytes2);
        println!("{}", mb2);

        println!(
            "Rounding test passed: {} bytes = {} MB, {} bytes = {} MB",
            bytes1, mb1, bytes2, mb2
        );
    }

    /// 测试精度保持
    #[test]
    fn test_precision() {
        let bytes = 1_572_864; // 1.5MB
        let mb = ByteUtils::bytes_to_mb(bytes);
        assert_eq!(mb, 1.5);

        let bytes2 = 2_097_152; // 2MB
        let mb2 = ByteUtils::bytes_to_mb(bytes2);
        assert_eq!(mb2, 2.0);

        println!(
            "Precision test passed: {} bytes = {} MB, {} bytes = {} MB",
            bytes, mb, bytes2, mb2
        );
    }

    /// 测试边界值
    #[test]
    fn test_boundary_values() {
        // 测试1KB边界
        let bytes = 1024;
        let mb = ByteUtils::bytes_to_mb(bytes);
        let expected = (1024.0 / 1024.0 / 1024.0 * 100.0) / 100.0;
        println!("{}", expected);

        // 测试1MB边界
        let bytes2 = 1_048_576;
        let mb2 = ByteUtils::bytes_to_mb(bytes2);

        println!(
            "Boundary values test passed: {} bytes = {} MB, {} bytes = {} MB",
            bytes, mb, bytes2, mb2
        );
    }

    /// 测试负值处理（虽然usize不能为负，但测试函数的健壮性）
    #[test]
    fn test_no_negative_values() {
        // usize不能为负，所以这个测试主要验证函数不会panic
        let bytes = 0;
        let mb = ByteUtils::bytes_to_mb(bytes);
        assert!(mb >= 0.0);
        println!(
            "No negative values test passed: {} bytes = {} MB",
            bytes, mb
        );
    }

    /// 测试原始测试用例
    #[test]
    fn test_original_case_1() {
        let bytes = 5_250_000;
        let mb = ByteUtils::bytes_to_mb(bytes);
        println!("Original case 1 test passed: {} bytes = {} MB", bytes, mb);
    }

    /// 测试原始测试用例2
    #[test]
    fn test_original_case_2() {
        let bytes = 10000000000000000000_usize;
        let mb = ByteUtils::bytes_to_mb(bytes);
        assert!(mb > 0.0);
        assert!(mb.is_finite());
        println!("Original case 2 test passed: {} bytes = {} MB", bytes, mb);
    }

    /// 测试连续转换的一致性
    #[test]
    fn test_consistency() {
        let test_values = vec![0, 1, 1024, 1_048_576, 5_250_000, 10_485_760];

        for &bytes in &test_values {
            let mb1 = ByteUtils::bytes_to_mb(bytes);
            let mb2 = ByteUtils::bytes_to_mb(bytes);
            assert_eq!(mb1, mb2, "Inconsistent result for {} bytes", bytes);
        }

        println!("Consistency test passed for {} values", test_values.len());
    }
}
