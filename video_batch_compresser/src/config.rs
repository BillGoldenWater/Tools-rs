use std::collections::HashMap;

use self::target::Target;

pub mod target;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub ffmpeg_args_presets: HashMap<String, Vec<String>>,
    pub target: Vec<Target>,
}
