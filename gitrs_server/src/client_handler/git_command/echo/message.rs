#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Outbound {
    Output(String),
}
