use std::path::PathBuf;

use self::{cached_regex::CachedRegex, regex_replace::RegexReplace};

pub mod cached_regex;
pub mod regex_replace;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Target {
    /// input dir
    pub input: PathBuf,
    /// output dir
    pub output: PathBuf,
    /// a regex, only file that name matches will be compressed
    pub filter: CachedRegex,
    /// input is the name of the file, require output rfc3339 timestamp
    pub time_extractor: RegexReplace,
    /// a duration in day(s), only file that has time before now-$filter_before will be compressed
    pub filter_before: u32,
    /// to get associated files that need copy to the dst, input is the name(with ext) of video file
    /// non exists file will be ignore
    pub associated_file_extractor: Vec<RegexReplace>,
    /// args to put between input and output
    pub ffmpeg_args: Option<Vec<String>>,
    /// args template id
    pub ffmpeg_args_id: Option<String>,
}
