use crate::error::VmCliError;
use async_compression::tokio::bufread::GzipDecoder;
use async_compression::tokio::write::GzipEncoder;
use std::ffi::OsStr;
use std::path::PathBuf;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio_tar::{Archive, Builder};

pub struct TarUtils;

impl TarUtils {
    // 压缩成 tar.gz 文件
    pub async fn compress_file_to_tar(
        src: &Vec<PathBuf>,
        dest: &PathBuf,
    ) -> Result<(), VmCliError> {
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
                return Err(VmCliError::VlogVmCliError(message));
            }
            let mut file = tokio::fs::File::open(&src_path).await?;
            let file_name = &src_path
                .file_name()
                .ok_or_else(|| VmCliError::VlogVmCliError("无法获取文件名".to_string()))?;

            tar_builder.append_file(file_name, &mut file).await?;
            tokio::fs::remove_file(&src_path).await?;
        }
        tar_builder.into_inner().await?.shutdown().await?;
        Ok(())
    }

    // 解压 tar 文件
    pub async fn decompress_tar(src: &PathBuf, dest: &PathBuf) -> Result<PathBuf, VmCliError> {
        if !src.exists() {
            return Err(VmCliError::VlogVmCliError("文件不存在".into()));
        }
        if !src.is_file() {
            return Err(VmCliError::VlogVmCliError(
                "当前解压的内容不是一个文件".into(),
            ));
        }
        let metadata = tokio::fs::metadata(src).await?;
        if metadata.len() == 0 {
            return Err(VmCliError::VlogVmCliError(
                "文件为空，可能下载不完整".into(),
            ));
        }
        // 检查文件名是否以 .tar.gz 结尾
        let file_name = src
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or("Cannot get file name")?;

        if !file_name.ends_with(".tar.gz") {
            return Err(VmCliError::VlogVmCliError(
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
mod tar_test {
    use crate::{error::VmCliError, utils::tar_utils::TarUtils};
    use std::path::PathBuf;

    #[tokio::test]
    async fn compress_test() -> Result<(), VmCliError> {
        let src = PathBuf::from("/Users/zhoujianing/vlogcli/a.json");
        let mut dest = PathBuf::from("/Users/zhoujianing/vlogcli/b.tar.gz");

        let mut list = vec![src];
        TarUtils::compress_file_to_tar(&mut list, &mut dest).await?;
        Ok(())
    }

    #[tokio::test]
    async fn decompress_test() -> Result<(), VmCliError> {
        let src = PathBuf::from("/Users/zhoujianing/vlogcli/a/2025-08-18T16:00:01Z.tar.gz");
        let dest = PathBuf::from("/Users/zhoujianing/vlogcli/a/b");
        let decompress_path = TarUtils::decompress_tar(&src, &dest).await?;

        println!("{:?}", decompress_path);
        Ok(())
    }
}
