use orion_variate::{EnvChecker, EnvDict, EnvEvaluable, ValueType};

fn main() {
    let json_multiline = r#"{
    "database_url": "${DB_URL}",
    "api_key": "${API_KEY}",
    "timeout": 30
}"#;

    println!("原始 JSON:");
    println!("{}", json_multiline);
    println!();

    let mut env_dict = EnvDict::new();
    env_dict.insert("DB_URL", ValueType::from("postgresql://localhost/mydb"));
    env_dict.insert("API_KEY", ValueType::from("secret-key-123"));

    println!("环境变量:");
    println!("  DB_URL = postgresql://localhost/mydb");
    println!("  API_KEY = secret-key-123");
    println!();

    let result = json_multiline.to_string().env_eval(&env_dict);

    println!("替换后的 JSON:");
    println!("{}", result);
    println!();

    println!("needs_env_eval: {}", result.needs_env_eval());
    if result.needs_env_eval() {
        println!("未定义变量: {:?}", result.list_env_vars());
    }
}
