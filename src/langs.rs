#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum Langs {
    English,
    Spanish,
    French,
    Italian,
    Portuguese,
    German,
}