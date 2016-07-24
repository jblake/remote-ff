use fb2;
use hyper::client::Client;

mod aooo;
mod ffn;
mod hpffa;

pub use self::aooo::*;
pub use self::ffn::*;
pub use self::hpffa::*;

#[derive(Clone, Debug, Hash, Eq, PartialEq, RustcDecodable, RustcEncodable)]
pub struct StoryInfo {
    pub title: String,
    pub author: String,
    pub chapters: u32,
    pub updated: i64,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct ChapterInfo {
    pub title: String,
    pub content: String,
}

pub trait Site {
    fn recognize(url: &str) -> Option<String>;
    fn get_url(id: &str) -> String;
    fn get_info(client: &Client, id: &str) -> Option<StoryInfo>;
    fn get_chapter(client: &Client, id: &str, chapter: u32) -> Option<ChapterInfo>;

    fn compile(client: &Client, id: &str, info: &StoryInfo) -> Option<String> {
        let mut chapters = Vec::<ChapterInfo>::new();
        for c in 1 .. info.chapters {
            if let Some(chapter) = Self::get_chapter(client, id, c) {
                chapters.push(chapter);
            } else if let Some(chapter) = Self::get_chapter(client, id, c) {
                chapters.push(chapter);
            } else {
                return None;
            }
        }
        return Some(fb2::compile(&*Self::get_url(id), info, &chapters));
    }
}
