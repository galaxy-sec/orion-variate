use derive_more::From;
use serde_derive::{Deserialize, Serialize};
use crate::addr::proxy::{auth::Auth, rule::Rule};

#[derive(Debug,Clone,Serialize,Deserialize,)]
pub struct Unit {
    rules : Vec<Rule>,
    auth : Option<Auth>,
}

#[derive(Debug,Clone,Serialize,Deserialize,From)]
pub enum ProxyPath {
    Origin(String),
    Proxy(String,Option<Auth>),
}
impl ProxyPath {
    pub fn path(&self) -> &str {
        match self {
            ProxyPath::Origin(path) => path,
            ProxyPath::Proxy(path, _) => path,
        }
    }
    pub fn is_proxy(&self)  -> bool {
        match self {
            ProxyPath::Origin(_) => false,
            ProxyPath::Proxy(_, _) => true,
        }
    }
}

impl Unit {
    pub fn new(rules: Vec<Rule>, auth: Option<Auth>) -> Self {
        Self { rules, auth }
    }

    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    pub fn auth(&self) -> &Option<Auth> {
        &self.auth
    }

    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    pub fn set_auth(&mut self, auth: Auth) {
        self.auth = Some(auth);
    }


    pub fn proxy(&self,input: &str) -> ProxyPath {
        for rule in &self.rules {
            let result = rule.replace(input);
            if let Some(result) = result {
                return ProxyPath::Proxy(result,self.auth.clone());
            }
        }
        ProxyPath::Origin(input.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_new() {
        let rules = vec![Rule::new("https://github.com/*", "https://mirror.com/")];
        let auth = Some(Auth::new("user".to_string(), "pass".to_string()));
        let unit = Unit::new(rules.clone(), auth.clone());
        
        assert_eq!(unit.rules().len(), 1);
        assert!(unit.auth().is_some());
    }

    #[test]
    fn test_unit_without_auth() {
        let rules = vec![Rule::new("https://github.com/*", "https://mirror.com/")];
        let unit = Unit::new(rules, None);
        
        assert_eq!(unit.rules().len(), 1);
        assert!(unit.auth().is_none());
    }

    #[test]
    fn test_unit_add_rule() {
        let mut unit = Unit::new(vec![], None);
        unit.add_rule(Rule::new("https://github.com/*", "https://mirror.com/"));
        
        assert_eq!(unit.rules().len(), 1);
    }

    #[test]
    fn test_unit_set_auth() {
        let mut unit = Unit::new(vec![], None);
        unit.set_auth(Auth::new("user".to_string(), "pass".to_string()));
        
        assert!(unit.auth().is_some());
    }

    

    #[test]
    fn test_unit_serialize_deserialize() {
        let rules = vec![Rule::new("https://github.com/*", "https://mirror.com/")];
        let auth = Some(Auth::new("user".to_string(), "pass".to_string()));
        let unit = Unit::new(rules, auth);
        
        let serialized = serde_json::to_string(&unit).unwrap();
        let deserialized: Unit = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deserialized.rules().len(), 1);
        assert!(deserialized.auth().is_some());
    }


}