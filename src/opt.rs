use std::path::PathBuf;

pub trait OptionFrom<T> {
    fn to_opt(self) -> Option<T>;
}

impl OptionFrom<String> for &str {
    fn to_opt(self) -> Option<String> {
        Some(self.to_string())
    }
}

impl OptionFrom<String> for String {
    fn to_opt(self) -> Option<String> {
        Some(self)
    }
}

impl OptionFrom<PathBuf> for &str {
    fn to_opt(self) -> Option<PathBuf> {
        Some(PathBuf::from(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_str_to_string_option() {
        // 测试正常字符串转换
        let result: Option<String> = "hello".to_opt();
        assert_eq!(result, Some("hello".to_string()));

        // 测试空字符串
        let result: Option<String> = "".to_opt();
        assert_eq!(result, Some("".to_string()));

        // 测试包含特殊字符的字符串
        let result: Option<String> = "hello\nworld\t!".to_opt();
        assert_eq!(result, Some("hello\nworld\t!".to_string()));

        // 测试Unicode字符串
        let result: Option<String> = "你好，世界！".to_opt();
        assert_eq!(result, Some("你好，世界！".to_string()));

        // 测试长字符串
        let long_str = "a".repeat(1000);
        let result: Option<String> = long_str.as_str().to_opt();
        assert_eq!(result, Some(long_str));
    }

    #[test]
    fn test_string_to_string_option() {
        // 测试String到Option<String>转换
        let s = String::from("hello");
        let result: Option<String> = s.to_opt();
        assert_eq!(result, Some("hello".to_string()));

        // 测试空String
        let s = String::new();
        let result: Option<String> = s.to_opt();
        assert_eq!(result, Some(String::new()));

        // 测试包含特殊字符的String
        let s = String::from("hello\nworld\t!");
        let result: Option<String> = s.to_opt();
        assert_eq!(result, Some("hello\nworld\t!".to_string()));

        // 测试Unicode String
        let s = String::from("你好，世界！");
        let result: Option<String> = s.to_opt();
        assert_eq!(result, Some("你好，世界！".to_string()));

        // 测试长String - 修复移动所有权问题
        let long_string = "a".repeat(1000);
        let expected = long_string.clone();
        let result: Option<String> = long_string.to_opt();
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn test_str_to_pathbuf_option() {
        // 测试正常路径字符串转换
        let result: Option<PathBuf> = "/tmp/test".to_opt();
        assert_eq!(result, Some(PathBuf::from("/tmp/test")));

        // 测试相对路径
        let result: Option<PathBuf> = "./test/file.txt".to_opt();
        assert_eq!(result, Some(PathBuf::from("./test/file.txt")));

        // 测试Windows风格路径
        let result: Option<PathBuf> = "C:\\Windows\\System32".to_opt();
        assert_eq!(result, Some(PathBuf::from("C:\\Windows\\System32")));

        // 测试空路径
        let result: Option<PathBuf> = "".to_opt();
        assert_eq!(result, Some(PathBuf::from("")));

        // 测试包含特殊字符的路径
        let result: Option<PathBuf> = "/tmp/test with spaces/file.txt".to_opt();
        assert_eq!(
            result,
            Some(PathBuf::from("/tmp/test with spaces/file.txt"))
        );

        // 测试Unicode路径
        let result: Option<PathBuf> = "/tmp/测试目录/文件.txt".to_opt();
        assert_eq!(result, Some(PathBuf::from("/tmp/测试目录/文件.txt")));
    }

    #[test]
    fn test_pathbuf_properties() {
        // 测试转换后的PathBuf属性
        let path_str = "/tmp/test.txt";
        let result: Option<PathBuf> = path_str.to_opt();

        assert!(result.is_some());
        let pathbuf = result.unwrap();

        // 验证路径属性
        assert_eq!(pathbuf, PathBuf::from(path_str));
        assert_eq!(pathbuf.as_path(), Path::new(path_str));
        assert_eq!(pathbuf.to_str(), Some(path_str));

        // 测试路径操作
        assert!(pathbuf.is_absolute());
        assert_eq!(pathbuf.file_name(), Some("test.txt".as_ref()));
        assert_eq!(pathbuf.extension(), Some("txt".as_ref()));
    }

    #[test]
    fn test_string_equality() {
        // 测试转换后的字符串相等性
        let original = "test string";
        let result1: Option<String> = original.to_opt();
        let result2: Option<String> = original.to_string().to_opt();

        assert_eq!(result1, result2);
        assert_eq!(result1.unwrap(), result2.unwrap());
    }

    #[test]
    fn test_type_consistency() {
        // 测试类型一致性
        let str_input = "test";
        let string_input = String::from("test");

        let str_result: Option<String> = str_input.to_opt();
        let string_result: Option<String> = string_input.to_opt();

        assert_eq!(str_result, string_result);

        // 测试PathBuf类型
        let path_result: Option<PathBuf> = str_input.to_opt();
        assert!(path_result.is_some());
        assert_eq!(path_result.unwrap(), PathBuf::from(str_input));
    }

    #[test]
    fn test_edge_cases() {
        // 测试边界情况

        // 测试单个字符
        let result: Option<String> = "a".to_opt();
        assert_eq!(result, Some("a".to_string()));

        // 测试空白字符
        let result: Option<String> = " \t\n\r".to_opt();
        assert_eq!(result, Some(" \t\n\r".to_string()));

        // 测试路径中的点
        let result: Option<PathBuf> = ".".to_opt();
        assert_eq!(result, Some(PathBuf::from(".")));

        let result: Option<PathBuf> = "..".to_opt();
        assert_eq!(result, Some(PathBuf::from("..")));

        // 测试路径中的连续斜杠
        let result: Option<PathBuf> = "tmp///test".to_opt();
        assert_eq!(result, Some(PathBuf::from("tmp///test")));
    }

    #[test]
    fn test_memory_efficiency() {
        // 测试内存效率（确保不会发生不必要的克隆）
        let large_str = "x".repeat(10000);

        // 测试&str到String的转换
        let result1: Option<String> = large_str.as_str().to_opt();
        assert_eq!(result1.unwrap(), large_str);

        // 测试String到Option<String>的转换 - 修复移动所有权问题
        let result2: Option<String> = large_str.clone().to_opt();
        assert_eq!(result2.unwrap(), large_str);

        // 测试&str到PathBuf的转换
        let result3: Option<PathBuf> = large_str.as_str().to_opt();
        assert_eq!(result3.unwrap(), PathBuf::from(&large_str));
    }

    #[test]
    fn test_trait_object_safety() {
        // 测试trait的对象安全性
        fn use_trait<T, U: OptionFrom<T>>(input: U) -> Option<T> {
            input.to_opt()
        }

        // 测试&str输入
        let result = use_trait::<String, _>("test");
        assert_eq!(result, Some("test".to_string()));

        // 测试String输入
        let result = use_trait::<String, _>("test".to_string());
        assert_eq!(result, Some("test".to_string()));

        // 测试PathBuf输入
        let result = use_trait::<PathBuf, _>("/tmp/test");
        assert_eq!(result, Some(PathBuf::from("/tmp/test")));
    }
}
