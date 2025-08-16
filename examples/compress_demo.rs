use orion_variate::archive::compress;
use std::fs;

use tempfile::tempdir;

fn main() -> anyhow::Result<()> {
    println!("🗜️  压缩进度指示器演示程序");
    println!("{}", "=".repeat(50));

    // 创建一个临时目录作为源目录
    let temp_dir = tempdir()?;
    let source_dir = temp_dir.path().join("demo_source");
    let output_archive = temp_dir.path().join("demo_archive.tar.gz");

    // 创建源目录
    fs::create_dir_all(&source_dir)?;

    // 创建一些示例文件来展示进度
    println!("📁 创建示例文件...");

    // 创建一些小文件
    for i in 1..=20 {
        let file_path = source_dir.join(format!("file_{i}.txt"));
        let content = format!(
            "示例文件 #{i}\n\
            这是第 {i} 个文件，用于演示压缩进度指示器。\n\
            Lorem ipsum dolor sit amet, consectetur adipiscing elit.\n\
            重复内容行 1: {i}\n\
            重复内容行 2: {i}\n\
            重复内容行 3: {i}\n\
            重复内容行 4: {i}\n\
            重复内容行 5: {i}"
        );
        fs::write(&file_path, content)?;
    }

    // 创建一个大文件来展示基于字节的进度效果
    println!("📄 创建大文件以展示字节级进度...");
    let large_file_path = source_dir.join("large_file.dat");
    let large_file_content = create_large_file_content(1024 * 50); // 50KB
    fs::write(&large_file_path, large_file_content)?;

    // 创建一些子目录和文件
    println!("📂 创建子目录结构...");
    for subdir_num in 1..=5 {
        let subdir = source_dir.join(format!("subdir_{subdir_num}"));
        fs::create_dir_all(&subdir)?;

        for file_num in 1..=3 {
            let file_path = subdir.join(format!("nested_{file_num}.txt"));
            let content = format!(
                "嵌套文件 - 目录 {subdir_num} 文件 {file_num}\n\
                这是一个嵌套在子目录中的文件，用于测试递归压缩。\n\
                Subdirectory {subdir_num} nested file number {file_num}"
            );
            fs::write(&file_path, content)?;
        }
    }

    // 创建一些空目录
    for i in 1..=3 {
        fs::create_dir_all(source_dir.join(format!("empty_dir_{i}")))?;
    }

    println!("\n🚀 开始压缩...");
    println!("源目录: {}", source_dir.display());
    println!("输出文件: {}", output_archive.display());
    println!("💡 注意：进度现在基于字节数而非文件数量");

    // 调用带进度指示器的压缩函数
    compress(&source_dir, &output_archive)?;

    // 验证压缩结果
    if output_archive.exists() {
        let metadata = fs::metadata(&output_archive)?;
        println!("\n✅ 压缩完成！");
        println!("📊 压缩文件大小: {} 字节", metadata.len());

        // 统计源文件数量和总大小
        let mut total_files = 0;
        let mut total_size = 0;
        for entry in walkdir::WalkDir::new(&source_dir)
            .into_iter()
            .filter_entry(|e| !is_hidden(e))
        {
            let entry = entry?;
            let path = entry.path();
            if path != source_dir && entry.file_type().is_file() {
                total_files += 1;
                total_size += entry.metadata()?.len();
            }
        }

        println!("📈 源文件统计:");
        println!("   文件数量: {total_files}");
        println!("   总大小: {total_size} 字节");
        println!(
            "   压缩率: {:.2}%",
            (metadata.len() as f64 / total_size as f64) * 100.0
        );
    } else {
        println!("\n❌ 压缩失败：输出文件不存在");
    }

    // 询问是否保留临时文件
    println!("\n💡 临时文件位于: {}", temp_dir.path().display());
    println!("   这些文件将在程序退出时自动删除");

    println!("\n演示完成！按 Enter 键退出...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();

    Ok(())
}

/// 创建大文件内容用于测试基于字节的进度指示器
fn create_large_file_content(size_in_bytes: usize) -> String {
    let mut content = String::with_capacity(size_in_bytes);

    // 添加文件头信息
    content.push_str("这是一个大文件，用于测试基于字节的压缩进度指示器。\n");
    content.push_str("文件大小：");
    content.push_str(&size_in_bytes.to_string());
    content.push_str(" 字节\n");
    content.push_str(&"=".repeat(80));
    content.push('\n');

    // 添加重复内容以达到目标大小
    let base_content = "这是一行测试内容，用于填充大文件。ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789\n";
    let base_len = base_content.len();

    while content.len() + base_len <= size_in_bytes {
        content.push_str(base_content);
    }

    // 填充剩余空间
    let remaining = size_in_bytes - content.len();
    if remaining > 0 {
        content.push_str(&"=".repeat(remaining));
    }

    content
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}
