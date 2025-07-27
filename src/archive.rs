use anyhow::{Context, Result};
use std::fs::File;
use std::path::Path;

/// 基于 tar -xzf 算法实现的解压函数
///
/// # 参数
/// * `archive_path` - 压缩文件路径 (.tar.gz 文件)
/// * `output_dir` - 解压目标目录
///
/// # 示例
/// ```
/// use orion_variate::archive::decompress;
///
/// // decompress("archive.tar.gz", "/tmp/extract").unwrap();
/// ```
pub fn decompress(archive_path: impl AsRef<Path>, output_dir: impl AsRef<Path>) -> Result<()> {
    let archive_path = archive_path.as_ref();
    let output_dir = output_dir.as_ref();

    // 确保输出目录存在
    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("创建输出目录失败: {}", output_dir.display()))?;

    // 打开 tar.gz 文件
    let file = File::open(archive_path)
        .with_context(|| format!("无法打开压缩文件: {}", archive_path.display()))?;

    // 创建 Gzip 解码器
    let decoder = flate2::read::GzDecoder::new(file);

    // 创建 tar 归档读取器
    let mut archive = tar::Archive::new(decoder);

    // 解压到目标目录
    archive
        .unpack(output_dir)
        .with_context(|| format!("解压文件失败: {}", archive_path.display()))?;

    Ok(())
}

/// 压缩目录为 tar.gz 文件
///
/// # 参数
/// * `source_dir` - 要压缩的目录
/// * `output_path` - 输出的 .tar.gz 文件路径
pub fn compress(source_dir: impl AsRef<Path>, output_path: impl AsRef<Path>) -> Result<()> {
    let source_dir = source_dir.as_ref();
    let output_path = output_path.as_ref();

    // 确保源目录存在
    if !source_dir.exists() || !source_dir.is_dir() {
        anyhow::bail!("源目录不存在: {}", source_dir.display());
    }

    // 创建输出文件
    let file = File::create(output_path)
        .with_context(|| format!("创建输出文件失败: {}", output_path.display()))?;

    // 创建 Gzip 编码器
    let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());

    // 创建 tar 归档写入器
    let mut tar = tar::Builder::new(encoder);

    // 递归添加目录内容
    tar.append_dir_all(".", source_dir)
        .with_context(|| format!("添加目录到压缩文件失败: {}", source_dir.display()))?;

    // 确保所有数据都写入完成
    tar.finish().with_context(|| "完成压缩文件写入失败")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_compress_decompress_roundtrip() {
        let temp_dir = tempdir().unwrap();
        let source_dir = temp_dir.path().join("source");
        let archive_path = temp_dir.path().join("test.tar.gz");
        let extract_dir = temp_dir.path().join("extract");

        // 创建测试文件
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("test.txt"), "Hello, World!").unwrap();
        fs::create_dir_all(source_dir.join("subdir")).unwrap();
        fs::write(source_dir.join("subdir").join("nested.txt"), "Nested file").unwrap();

        // 测试压缩
        compress(&source_dir, &archive_path).unwrap();
        assert!(archive_path.exists());

        // 测试解压
        decompress(&archive_path, &extract_dir).unwrap();

        // 验证解压结果
        assert!(extract_dir.join("test.txt").exists());
        assert!(extract_dir.join("subdir").join("nested.txt").exists());

        assert_eq!(
            fs::read_to_string(extract_dir.join("test.txt")).unwrap(),
            "Hello, World!"
        );
        assert_eq!(
            fs::read_to_string(extract_dir.join("subdir").join("nested.txt")).unwrap(),
            "Nested file"
        );
    }

    #[test]
    fn test_decompress_nonexistent_file() {
        let temp_dir = tempdir().unwrap();
        let result = decompress("nonexistent.tar.gz", temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_compress_nonexistent_directory() {
        let temp_dir = tempdir().unwrap();
        let archive_path = temp_dir.path().join("test.tar.gz");
        let result = compress("nonexistent_directory", &archive_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_compress_empty_directory() {
        let temp_dir = tempdir().unwrap();
        let source_dir = temp_dir.path().join("empty");
        let archive_path = temp_dir.path().join("empty.tar.gz");
        let extract_dir = temp_dir.path().join("extract_empty");

        // 创建空目录
        fs::create_dir_all(&source_dir).unwrap();

        // 测试压缩空目录
        compress(&source_dir, &archive_path).unwrap();
        assert!(archive_path.exists());

        // 测试解压
        decompress(&archive_path, &extract_dir).unwrap();

        // 验证解压结果
        assert!(extract_dir.exists());
    }

    #[test]
    fn test_compress_with_special_files() {
        let temp_dir = tempdir().unwrap();
        let source_dir = temp_dir.path().join("special");
        let archive_path = temp_dir.path().join("special.tar.gz");
        let extract_dir = temp_dir.path().join("extract_special");

        // 创建测试文件和特殊文件
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("normal.txt"), "Normal file").unwrap();

        // 创建子目录
        fs::create_dir_all(source_dir.join("subdir")).unwrap();
        fs::write(source_dir.join("subdir").join("file.txt"), "Subdir file").unwrap();

        // 测试压缩
        compress(&source_dir, &archive_path).unwrap();
        assert!(archive_path.exists());

        // 测试解压
        decompress(&archive_path, &extract_dir).unwrap();

        // 验证解压结果
        assert!(extract_dir.join("normal.txt").exists());
        assert!(extract_dir.join("subdir").exists());
        assert!(extract_dir.join("subdir").join("file.txt").exists());

        assert_eq!(
            fs::read_to_string(extract_dir.join("normal.txt")).unwrap(),
            "Normal file"
        );
        assert_eq!(
            fs::read_to_string(extract_dir.join("subdir").join("file.txt")).unwrap(),
            "Subdir file"
        );
    }

    #[test]
    fn test_decompress_to_existing_directory() {
        let temp_dir = tempdir().unwrap();
        let source_dir = temp_dir.path().join("source");
        let archive_path = temp_dir.path().join("test.tar.gz");
        let extract_dir = temp_dir.path().join("existing");

        // 创建源文件
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("test.txt"), "Test content").unwrap();

        // 创建已存在的目标目录
        fs::create_dir_all(&extract_dir).unwrap();
        fs::write(extract_dir.join("existing.txt"), "Existing content").unwrap();

        // 压缩
        compress(&source_dir, &archive_path).unwrap();

        // 解压到已存在的目录
        decompress(&archive_path, &extract_dir).unwrap();

        // 验证解压结果
        assert!(extract_dir.join("test.txt").exists());
        assert!(extract_dir.join("existing.txt").exists());

        assert_eq!(
            fs::read_to_string(extract_dir.join("test.txt")).unwrap(),
            "Test content"
        );
        assert_eq!(
            fs::read_to_string(extract_dir.join("existing.txt")).unwrap(),
            "Existing content"
        );
    }
}
