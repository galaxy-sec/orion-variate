use std::fs;

fn main() {
    println!("=== 调试中文字符编码问题 ===");

    // 读取输入文件
    let input_content =
        fs::read_to_string("tests/data/yml/case4_in.yml").expect("无法读取输入文件");

    // 读取期望输出文件
    let expected_content =
        fs::read_to_string("tests/data/yml/case4_out.yml").expect("无法读取期望输出文件");

    println!("=== 输入文件内容 ===");
    println!("{}", input_content);
    println!("\n=== 期望输出文件内容 ===");
    println!("{}", expected_content);

    // 检查文件内容是否相同
    if input_content == expected_content {
        println!("\n✓ 输入文件和期望输出文件内容完全相同");
    } else {
        println!("\n✗ 输入文件和期望输出文件内容不同");
    }

    // 分析中文字符的UTF-8字节序列
    println!("\n=== 中文字符分析 ===");
    let chinese_chars = vec![
        "失败", "错误", "注意", "警告", "消息", "信息", "调试", "追踪",
    ];

    for ch in chinese_chars {
        println!("字符: '{}'", ch);
        println!("UTF-8字节: {:?}", ch.as_bytes());
        println!("Unicode转义: {:?}", ch.escape_unicode().to_string());
        println!("Debug格式: {:?}", ch);
        println!();
    }
}
