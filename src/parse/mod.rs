extern crate hyper;
extern crate time;

mod ffn;
mod hpffa;

pub use self::ffn::*;
pub use self::hpffa::*;

pub struct StoryInfo {
    pub title: String,
    pub author: String,
    pub chapters: u32,
    pub updated: time::Tm,
}

pub struct ChapterInfo {
    pub title: String,
    pub content: String,
}

pub trait Site {
    fn get_url(id: &str) -> String;
    fn get_info(client: &hyper::client::Client, id: &str) -> Option<StoryInfo>;
    fn get_chapter(client: &hyper::client::Client, id: &str, chapter: u32) -> Option<ChapterInfo>;
}
