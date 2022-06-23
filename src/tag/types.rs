use std::fmt::Display;

#[derive(PartialEq, Debug, Clone)]
pub struct Tag {
    pub name: String,
    pub value: Option<String>,
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value.as_ref() {
            Some(value) => write!(f, "{}:{}", self.name, value),
            None => write!(f, "{}:", self.name),
        }
    }
}
