use parse::StoryInfo;
use rustc_serialize::json;

pub fn load(path: &str) -> Vec<StoryInfo> {
    return json::decode("[]").unwrap();
}
