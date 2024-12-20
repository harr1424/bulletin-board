use log::error;
use std::str::FromStr;

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Langs {
    English,
    Spanish,
    French,
    Italian,
    Portuguese,
    German,
}

impl Langs {
    fn from_str_internal(s: &str) -> Option<Self> {
        match s {
            "English" => Some(Langs::English),
            "Spanish" => Some(Langs::Spanish),
            "French" => Some(Langs::French),
            "Italian" => Some(Langs::Italian),
            "Portuguese" => Some(Langs::Portuguese),
            "German" => Some(Langs::German),
            _ => None,
        }
    }
}

impl ToString for Langs {
    fn to_string(&self) -> String {
        match self {
            Langs::English => "English",
            Langs::Spanish => "Spanish",
            Langs::French => "French",
            Langs::Italian => "Italian",
            Langs::Portuguese => "Portuguese",
            Langs::German => "German",
        }
        .to_string()
    }
}

impl From<String> for Langs {
    fn from(s: String) -> Self {
        Langs::from_str_internal(&s).unwrap_or_else(|| {
            error!("Unknown language found in table: {}", s);
            Langs::English 
        })
    }
}

impl FromStr for Langs {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Langs::from_str_internal(s).ok_or_else(|| format!("Unknown language: {}", s))
    }
}
