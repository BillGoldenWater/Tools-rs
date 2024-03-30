use self::target::Target;

pub mod target;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub target: Vec<Target>,
}
