use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Hash, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Top,
    Jungle,
    Mid,
    Bot,
    Support,
    #[default]
    Other,
}

impl From<&str> for Role {
    fn from(value: &str) -> Self {
        // Match to values from Riot API
        match value.to_lowercase().as_str() {
            "top" => Role::Top,
            "jungle" => Role::Jungle,
            "middle" => Role::Mid,
            "bottom" => Role::Bot,
            "utility" => Role::Support,
            _ => Role::Other,
        }
    }
}
