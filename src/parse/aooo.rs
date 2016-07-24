use hyper;
use parse::*;
use regex::Regex;
use sanitize::*;
use scraper::{Html, Selector};
use std::io::Read;

pub struct Aooo { }

impl Site for Aooo {
    fn recognize(url: &str) -> Option<String> {
        lazy_static! {
            static ref RE : Regex = Regex::new(r"^https?://archiveofourown\.org/works/(\d+)").unwrap();
        }
        match RE.captures(url) {
            None => return None,
            Some(m) => match m.at(1) {
                None => return None,
                Some(s) => return Some(s.to_string()),
            },
        }
    }

    fn get_url(id: &str) -> String { format!("http://archiveofourown.org/works/{}", id) }

    fn get_info(client: &hyper::client::Client, id: &str) -> Option<StoryInfo> {
        return None;
        /* XXX Page differs significantly between "warning" form and "safe" form, need to use a different URL
        lazy_static! {
            static ref CHAPTERS_RE : Regex = Regex::new(r"(\d+)/").unwrap();
        }
        let url = format!("http://archiveofourown.org/works/{}", id);
        let mut res = match client.get(&url).send() {
            Ok(x) => x,
            _ => return None,
        };
        if res.status != hyper::Ok {
            return None;
        }

        let mut rawhtml = Vec::<u8>::new();
        res.read_to_end(&mut rawhtml).unwrap();
        let html = String::from_utf8_lossy(&rawhtml);

        let doc = Html::parse_document(&html);

        let title_elem = doc.select(&Selector::parse("#main .heading a:nth-child(1)").unwrap()).next();
        if title_elem.is_none() {
            return None;
        }

        let title = title_elem.unwrap().text().next().unwrap();
        let author = doc.select(&Selector::parse("h4 a+ a").unwrap()).next().unwrap().text().next().unwrap();
        let chapterspair = doc.select(&Selector::parse("dd.chapters").unwrap()).next().unwrap().text().next().unwrap();
        let chapters = CHAPTERS_RE.captures(chapterspair).unwrap().at(1).unwrap();
        let updated = doc.select(&Selector::parse(".datetime").unwrap()).next().unwrap().text().next().unwrap();

        return Some(StoryInfo {
            title: title.to_string(),
            author: author.to_string(),
            chapters: chapters.parse().unwrap(),
            updated: updated.parse().unwrap(),
        });
        */
    }

    fn get_chapter(client: &hyper::client::Client, id: &str, chapter: u32) -> Option<ChapterInfo> {
        return None;
        /* XXX Need to work out how to generate chapter URLs
        let url = format!("https://m.fanfiction.net/s/{}/{}", id, chapter);
        let mut res = match client.get(&url).send() {
            Ok(x) => x,
            _ => return None,
        };
        if res.status != hyper::Ok {
            return None;
        }

        let mut rawhtml = Vec::<u8>::new();
        res.read_to_end(&mut rawhtml).unwrap();
        let html = String::from_utf8_lossy(&rawhtml);

        let doc = Html::parse_document(&html);

        let title = doc.select(&Selector::parse("#content").unwrap()).next().unwrap().text().last().unwrap().trim();
        let content = doc.select(&Selector::parse("#storycontent").unwrap()).next().unwrap();

        return Some(ChapterInfo {
            title: if title == "" { format!("Chapter {}", chapter) } else { title.to_string() },
            content: sanitize(&content),
        });
        */
    }
}
