use std::fmt::Display;

#[derive(PartialEq, Debug, Clone)]
pub struct Description {
    pub payee: Option<String>,
    pub note: Option<String>,
}

impl Display for Description {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Description {
                payee: Some(p),
                note: Some(n),
            } => write!(f, "{} | {}", p, n),
            Description {
                payee: None,
                note: Some(n),
            } => write!(f, "{}", n),
            Description {
                payee: Some(p),
                note: None,
            } => write!(f, "{} |", p),
            Description {
                payee: None,
                note: None,
            } => write!(f, ""),
        }
    }
}
