#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum Outbound {
    Result { output: String },
}
