use winnow::{ascii::multispace0, combinator::{alt, separated}, token::{literal, take_while}, Parser, Result};

use crate::vars::{parse::base::{gal_raw_str, take_bool, take_float, take_number, take_string, wn_desc}, ValueObj, ValueType, ValueVec};

pub fn gal_simple_value(data: &mut &str) -> Result<ValueType> {
    alt((
        take_string.map(ValueType::from),
        take_bool.map(ValueType::from),
        take_float.map(ValueType::from),
        take_number.map(ValueType::from),
        gal_raw_str.map(ValueType::from),
    ))
    .parse_next(data)
}

pub fn take_value(data: &mut &str) -> Result<ValueType> {
    multispace0.parse_next(data)?;
    let v = alt((gal_simple_value, take_obj_value, take_vec_value)).parse_next(data)?;
    multispace0.parse_next(data)?;
    Ok(v)
}

pub fn gal_named_value(input: &mut &str) -> Result<(String, ValueType)> {
    multispace0.parse_next(input)?;
    let key = take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '.']))
        .context(wn_desc("<var-name>"))
        .parse_next(input)?;
    multispace0.parse_next(input)?;
    ":".parse_next(input)?;
    let _ = multispace0.parse_next(input)?;

    let val = take_value
        .context(wn_desc("<var-val>"))
        .parse_next(input)?;
    multispace0(input)?;
    Ok((key.to_string(), val))
}


//task obj
// x = { a : "A", b : "B" , c : 1}
pub fn take_obj_value(data: &mut &str) -> Result<ValueType> {
    take_value_map.parse_next(data).map(ValueType::from)
}

pub fn take_vec_value(data: &mut &str) -> Result<ValueType> {
    take_value_vec.parse_next(data).map(ValueType::from)
}
pub fn take_value_vec(data: &mut &str) -> Result<ValueVec> {
    let _ = multispace0.parse_next(data)?;
    literal("[").context(wn_desc("[")).parse_next(data)?;
    let _ = multispace0.parse_next(data)?;
    let items: ValueVec = separated(0.., take_value, ",").parse_next(data)?;
    literal("]").context(wn_desc("]")).parse_next(data)?;
    Ok(items)
}

pub fn take_value_map(data: &mut &str) -> Result<ValueObj> {
    let _ = multispace0.parse_next(data)?;
    literal("{")
        .context(wn_desc("vec start"))
        .parse_next(data)?;
    let _ = multispace0.parse_next(data)?;
    let items: Vec<(String, ValueType)> =
        separated(0.., gal_named_value, ",").parse_next(data)?;
    literal("}").parse_next(data)?;
    let mut obj = ValueObj::new();
    items.into_iter().for_each(|(k, v)| {
        obj.insert(k, v);
    });
    Ok(obj)
}

#[cfg(test)]
mod tests {

    use orion_error::TestAssert;


    use super::*;

    #[test]
    fn test_take_obj_value() -> Result<()> {
        // 测试空对象
        let mut input = "{}";
        assert_eq!(take_value_map(&mut input)?.len(), 0);

        // 测试单键值对对象
        let mut input = "{ key: \"value\" }";
        let obj = take_value_map(&mut input).assert();
        assert_eq!(
            obj.get(&"key".to_string()).assert(),
            &ValueType::from("value".to_string())
        );

        // 测试多键值对对象
        let mut input = "{ a: 1, b: \"two\", c: true,d: 1.1 }";
        let obj = take_value_map(&mut input)?;
        assert_eq!(
            obj.get(&"a".to_string()).unwrap(),
            &ValueType::from(1)
        );
        assert_eq!(
            obj.get(&"b".to_string()).unwrap(),
            &ValueType::from("two".to_string())
        );
        assert_eq!(
            obj.get(&"c".to_string()).unwrap(),
            &ValueType::from(true)
        );

        // 测试嵌套对象
        let mut input = "{ outer: { inner: 42 } }";
        let obj = take_value_map(&mut input).assert();
        if let ValueType::Obj(inner) = obj.get(&"outer".to_string()).assert() {
            assert_eq!(
                inner.get(&"inner".to_string()).unwrap(),
                &ValueType::from(42)
            );
        } else {
            panic!("Expected nested object");
        }

        // 测试缺少闭合括号
        let mut input = "{ key: value";
        assert!(take_obj_value(&mut input).is_err());

        // 测试缺少冒号
        let mut input = "{ key value }";
        assert!(take_obj_value(&mut input).is_err());

        Ok(())
    }
    #[test]
    fn test_take_value_vec() -> Result<()> {
        use super::*;

        // 测试空列表
        let mut input = "[]";
        assert_eq!(take_value_vec(&mut input)?, Vec::<ValueType>::new());

        // 测试单元素列表（字符串）
        let mut input = r#"["hello"]"#;
        assert_eq!(
            take_value_vec(&mut input).assert(),
            vec![ValueType::from("hello".to_string())]
        );
        let mut input = r#"["hello", "hello2"]"#;
        assert_eq!(
            take_value_vec(&mut input).assert(),
            vec![
                ValueType::from("hello".to_string()),
                ValueType::from("hello2".to_string())
            ]
        );

        // 测试多元素列表（混合类型）
        let mut input = r#"[42, "world", true]"#;
        assert_eq!(
            take_value_vec(&mut input).assert(),
            vec![
                ValueType::from(42),
                ValueType::from("world".to_string()),
                ValueType::from(true),
            ]
        );

        // 测试嵌套列表
        let mut input = r#"[[1, 2], ["a", "b"]]"#;
        assert_eq!(
            take_value_vec(&mut input)?,
            vec![
                ValueType::List(vec![ValueType::from(1), ValueType::from(2),]),
                ValueType::List(vec![
                    ValueType::from("a".to_string()),
                    ValueType::from("b".to_string()),
                ]),
            ]
        );

        // 测试带空格的列表
        let mut input = r#"[ 1 ,  "two" ,  false ]"#;
        assert_eq!(
            take_value_vec(&mut input)?,
            vec![
                ValueType::from(1),
                ValueType::from("two".to_string()),
                ValueType::from(false),
            ]
        );

        // 测试无效格式（缺少闭合括号）
        let mut input = r#"[1, 2"#;
        assert!(take_value_vec(&mut input).is_err());

        // 测试无效格式（缺少分隔符）
        let mut input = r#"[1 2]"#;
        assert!(take_value_vec(&mut input).is_err());

        Ok(())
    }
}
