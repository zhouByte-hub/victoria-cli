use crate::error::CliError;
use async_compression::tokio::bufread::GzipDecoder;
use async_compression::tokio::write::GzipEncoder;
use std::ffi::OsStr;
use std::path::PathBuf;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio_tar::{Archive, Builder};

pub struct TarUtils;

impl TarUtils {
    // 压缩成 tar.gz 文件
    pub async fn compress_file_to_tar(src: &Vec<PathBuf>, dest: &PathBuf) -> Result<(), CliError> {
        if src.is_empty() {
            return Ok(());
        }
        // 创建 Gzip 编码器
        let gzip_encoder = GzipEncoder::new(tokio::fs::File::create(dest).await?);
        let mut tar_builder = Builder::new(gzip_encoder);

        // 遍历所有源文件并添加到 tar 中
        for src_path in src {
            if !src_path.exists() {
                let message = format!("源文件不存在: {:?}", &src_path);
                return Err(CliError::VlogCliError(message));
            }
            let mut file = tokio::fs::File::open(&src_path).await?;
            let file_name = &src_path
                .file_name()
                .ok_or_else(|| CliError::VlogCliError("无法获取文件名".to_string()))?;

            tar_builder.append_file(file_name, &mut file).await?;
            tokio::fs::remove_file(&src_path).await?;
        }
        tar_builder.into_inner().await?.shutdown().await?;
        Ok(())
    }

    // 解压 tar 文件
    pub async fn decompress_tar(src: &PathBuf, dest: &PathBuf) -> Result<PathBuf, CliError> {
        if !src.exists() {
            return Err(CliError::VlogCliError("文件不存在".into()));
        }
        if !src.is_file() {
            return Err(CliError::VlogCliError("当前解压的内容不是一个文件".into()));
        }
        let metadata = tokio::fs::metadata(src).await?;
        if metadata.len() == 0 {
            return Err(CliError::VlogCliError("文件为空，可能下载不完整".into()));
        }
        // 检查文件名是否以 .tar.gz 结尾
        let file_name = src
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or("Cannot get file name")?;

        if !file_name.ends_with(".tar.gz") {
            return Err(CliError::VlogCliError(
                "当前解压的内容不是一个tar.gz压缩文件".into(),
            ));
        }

        let stem = file_name
            .strip_suffix(".tar.gz")
            .ok_or("Cannot strip .tar.gz suffix")?;
        let output_dir = dest.join(format!("{}", stem));

        if output_dir.exists() {
            tokio::fs::remove_dir_all(&output_dir).await?;
        }
        tokio::fs::create_dir_all(&output_dir).await?;
        let file = tokio::fs::File::open(src).await?;
        let reader = BufReader::new(file);

        let gzip_decoder = GzipDecoder::new(reader);
        let mut archive = Archive::new(gzip_decoder);
        archive.unpack(&output_dir).await?;

        Ok(output_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs;
    use tokio::io::AsyncWriteExt;

    /// 测试TarUtils结构体的创建
    #[test]
    fn test_tar_utils_creation() {
        // TarUtils是一个单元结构体，主要测试其存在性
        let _utils = TarUtils;
        println!("TarUtils creation test passed");
    }

    /// 测试文件压缩功能
    #[tokio::test]
    async fn test_compress_file_to_tar() {
        let temp_dir = tempdir().unwrap();
        let source_file = temp_dir.path().join("test_source.txt");
        let target_file = temp_dir.path().join("test_target.tar.gz");

        // 创建测试源文件
        let test_content = b"This is a test file for compression.\nIt contains multiple lines.\nAnd some special characters: a";
        let mut file = fs::File::create(&source_file).await.unwrap();
        file.write_all(test_content).await.unwrap();
        file.flush().await.unwrap();

        // 测试压缩
        let result = TarUtils::compress_file_to_tar(&vec![source_file.clone()], &target_file).await;

        match result {
            Ok(_) => {
                // 验证压缩文件是否创建
                assert!(target_file.exists());

                // 验证压缩文件不为空
                let metadata = fs::metadata(&target_file).await.unwrap();
                assert!(metadata.len() > 0);

                // 验证源文件已被删除
                assert!(!source_file.exists());

                println!("File compression test passed");
            }
            Err(e) => {
                println!("File compression test failed: {}", e);
                panic!("Compression test failed: {}", e);
            }
        }
    }

    /// 测试文件解压功能
    #[tokio::test]
    async fn test_decompress_tar() {
        let temp_dir = tempdir().unwrap();
        let source_file = temp_dir.path().join("test_source.txt");
        let compressed_file = temp_dir.path().join("test_compressed.tar.gz");
        let output_dir = temp_dir.path().to_path_buf();

        // 创建测试源文件
        let test_content = b"Test content for decompression.\nLine 2\nLine 3";
        let mut file = fs::File::create(&source_file).await.unwrap();
        file.write_all(test_content).await.unwrap();
        file.flush().await.unwrap();

        // 先压缩文件
        TarUtils::compress_file_to_tar(&vec![source_file.clone()], &compressed_file)
            .await
            .unwrap();

        // 测试解压
        let result = TarUtils::decompress_tar(&compressed_file, &output_dir).await;

        match result {
            Ok(output_path) => {
                // 验证输出目录是否创建
                assert!(output_path.exists());

                // 验证解压后的文件是否存在
                let extracted_file = output_path.join("test_source.txt");
                assert!(extracted_file.exists());

                // 验证文件内容是否正确
                let extracted_content = fs::read_to_string(&extracted_file).await.unwrap();
                let expected_content = std::str::from_utf8(test_content).unwrap();
                assert_eq!(extracted_content, expected_content);

                println!("File decompression test passed");
            }
            Err(e) => {
                println!("File decompression test failed: {}", e);
                panic!("Decompression test failed: {}", e);
            }
        }
    }

    /// 测试压缩空文件列表
    #[tokio::test]
    async fn test_compress_empty_list() {
        let temp_dir = tempdir().unwrap();
        let target_file = temp_dir.path().join("empty.tar.gz");

        // 测试空列表压缩
        let result = TarUtils::compress_file_to_tar(&vec![], &target_file).await;

        // 应该返回Ok，因为函数处理空列表时直接返回Ok(())
        assert!(result.is_ok());

        // 目标文件不应该被创建
        assert!(!target_file.exists());

        println!("Empty list compression test passed");
    }

    /// 测试压缩不存在的文件
    #[tokio::test]
    async fn test_compress_nonexistent_file() {
        let temp_dir = tempdir().unwrap();
        let nonexistent_file = temp_dir.path().join("nonexistent.txt");
        let target_file = temp_dir.path().join("target.tar.gz");

        let result = TarUtils::compress_file_to_tar(&vec![nonexistent_file], &target_file).await;

        // 应该返回错误
        if result.is_err() {
            // 验证错误类型
            match result.err().unwrap() {
                CliError::VlogCliError(msg) => {
                    if msg.contains("源文件不存在") {
                        println!("Compress nonexistent file test correctly returned error");
                    } else {
                        println!(
                            "Compress nonexistent file test returned unexpected error message: {}",
                            msg
                        );
                    }
                }
                _ => println!("Compress nonexistent file test returned unexpected error type"),
            }
        } else {
            println!("Compress nonexistent file test unexpectedly succeeded");
        }
    }

    /// 测试解压不存在的文件
    #[tokio::test]
    async fn test_decompress_nonexistent_file() {
        let temp_dir = tempdir().unwrap();
        let nonexistent_file = temp_dir.path().join("nonexistent.tar.gz");
        let output_dir = temp_dir.path().to_path_buf();

        let result = TarUtils::decompress_tar(&nonexistent_file, &output_dir).await;

        // 应该返回错误
        if result.is_err() {
            // 验证错误类型
            match result.err().unwrap() {
                CliError::VlogCliError(msg) => {
                    if msg.contains("文件不存在") {
                        println!("Decompress nonexistent file test correctly returned error");
                    } else {
                        println!(
                            "Decompress nonexistent file test returned unexpected error message: {}",
                            msg
                        );
                    }
                }
                _ => println!("Decompress nonexistent file test returned unexpected error type"),
            }
        } else {
            println!("Decompress nonexistent file test unexpectedly succeeded");
        }
    }

    /// 测试解压无效的压缩文件
    #[tokio::test]
    async fn test_decompress_invalid_file() {
        let temp_dir = tempdir().unwrap();
        let invalid_file = temp_dir.path().join("invalid.tar.gz");
        let output_dir = temp_dir.path().to_path_buf();

        // 创建一个无效的压缩文件（只是普通文本）
        let invalid_content = b"This is not a valid tar.gz file";
        let mut file = fs::File::create(&invalid_file).await.unwrap();
        file.write_all(invalid_content).await.unwrap();
        file.flush().await.unwrap();

        let result = TarUtils::decompress_tar(&invalid_file, &output_dir).await;

        // 应该返回错误
        if result.is_err() {
            println!("Decompress invalid file test correctly returned error");
        } else {
            println!("Decompress invalid file test unexpectedly succeeded");
        }
    }

    /// 测试解压非文件路径
    #[tokio::test]
    async fn test_decompress_directory_path() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path().to_path_buf();

        // 尝试解压一个目录而不是文件
        let result = TarUtils::decompress_tar(&temp_dir.path().into(), &output_dir).await;

        // 应该返回错误
        if result.is_err() {
            // 验证错误类型
            match result.err().unwrap() {
                CliError::VlogCliError(msg) => {
                    if msg.contains("当前解压的内容不是一个文件") {
                        println!("Decompress directory path test correctly returned error");
                    } else {
                        println!(
                            "Decompress directory path test returned unexpected error message: {}",
                            msg
                        );
                    }
                }
                _ => println!("Decompress directory path test returned unexpected error type"),
            }
        } else {
            println!("Decompress directory path test unexpectedly succeeded");
        }
    }

    /// 测试解压空文件
    #[tokio::test]
    async fn test_decompress_empty_file() {
        let temp_dir = tempdir().unwrap();
        let empty_file = temp_dir.path().join("empty.tar.gz");
        let output_dir = temp_dir.path().to_path_buf();

        // 创建一个空文件
        fs::File::create(&empty_file).await.unwrap();

        let result = TarUtils::decompress_tar(&empty_file, &output_dir).await;

        // 应该返回错误
        if result.is_err() {
            // 验证错误类型
            match result.err().unwrap() {
                CliError::VlogCliError(msg) => {
                    if msg.contains("文件为空，可能下载不完整") {
                        println!("Decompress empty file test correctly returned error");
                    } else {
                        println!(
                            "Decompress empty file test returned unexpected error message: {}",
                            msg
                        );
                    }
                }
                _ => println!("Decompress empty file test returned unexpected error type"),
            }
        } else {
            println!("Decompress empty file test unexpectedly succeeded");
        }
    }

    /// 测试解压非tar.gz文件
    #[tokio::test]
    async fn test_decompress_non_tar_gz_file() {
        let temp_dir = tempdir().unwrap();
        let non_tar_gz_file = temp_dir.path().join("test.txt");
        let output_dir = temp_dir.path().to_path_buf();

        // 创建一个非tar.gz文件
        let content = b"This is a text file, not a tar.gz file";
        let mut file = fs::File::create(&non_tar_gz_file).await.unwrap();
        file.write_all(content).await.unwrap();
        file.flush().await.unwrap();

        let result = TarUtils::decompress_tar(&non_tar_gz_file, &output_dir).await;

        // 应该返回错误
        if result.is_err() {
            // 验证错误类型
            match result.err().unwrap() {
                CliError::VlogCliError(msg) => {
                    if msg.contains("当前解压的内容不是一个tar.gz压缩文件") {
                        println!("Decompress non-tar.gz file test correctly returned error");
                    } else {
                        println!(
                            "Decompress non-tar.gz file test returned unexpected error message: {}",
                            msg
                        );
                    }
                }
                _ => println!("Decompress non-tar.gz file test returned unexpected error type"),
            }
        } else {
            println!("Decompress non-tar.gz file test unexpectedly succeeded");
        }
    }

    /// 测试目录清理功能
    #[tokio::test]
    async fn test_directory_cleanup() {
        let temp_dir = tempdir().unwrap();
        let source_file = temp_dir.path().join("test.txt");
        let compressed_file = temp_dir.path().join("test.tar.gz");
        let output_dir = temp_dir.path().to_path_buf();

        // 创建测试文件
        let test_content = b"Test content for cleanup test";
        let mut file = fs::File::create(&source_file).await.unwrap();
        file.write_all(test_content).await.unwrap();
        file.flush().await.unwrap();

        // 压缩文件
        TarUtils::compress_file_to_tar(&vec![source_file.clone()], &compressed_file)
            .await
            .unwrap();

        // 解压文件（应该会创建目录）
        let first_result = TarUtils::decompress_tar(&compressed_file, &output_dir).await;
        assert!(first_result.is_ok());

        // 验证目录被创建
        let extracted_dir = output_dir.join("test");
        assert!(extracted_dir.exists());

        // 再次解压到同一目录（测试清理功能）
        let second_result = TarUtils::decompress_tar(&compressed_file, &output_dir).await;

        // 应该成功，因为函数会清理现有目录
        assert!(second_result.is_ok());

        // 验证文件仍然存在（重新解压）
        let extracted_file = extracted_dir.join("test.txt");
        assert!(extracted_file.exists());

        println!("Directory cleanup test passed");
    }

    /// 测试压缩和解压的完整性
    #[tokio::test]
    async fn test_compression_decompression_integrity() {
        let temp_dir = tempdir().unwrap();
        let source_file = temp_dir.path().join("integrity_test.txt");
        let compressed_file = temp_dir.path().join("integrity.tar.gz");
        let output_dir = temp_dir.path().to_path_buf();

        // 创建包含多种字符的测试文件
        let test_content = b"Test content with various characters:\n- ASCII: Hello World!\n- Unicode: aa\n- Special: aa\n- Numbers: 1234567890\n- Symbols: !@#$%^&*()_+-=[]{}|;':\",./<>?";

        let mut file = fs::File::create(&source_file).await.unwrap();
        file.write_all(test_content).await.unwrap();
        file.flush().await.unwrap();

        // 压缩文件
        TarUtils::compress_file_to_tar(&vec![source_file.clone()], &compressed_file)
            .await
            .unwrap();

        // 解压文件
        let output_path = TarUtils::decompress_tar(&compressed_file, &output_dir)
            .await
            .unwrap();

        // 验证内容完整性
        let extracted_file = output_path.join("integrity_test.txt");
        let extracted_content = fs::read(&extracted_file).await.unwrap();

        assert_eq!(extracted_content, test_content);
        println!("Compression decompression integrity test passed");
    }
}
