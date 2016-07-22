use hyper;
use parse::*;
use regex::Regex;
use sanitize::*;
use scraper::{Html, Selector};
use std::io::Read;
use time;

pub struct Hpffa { }

impl Site for Hpffa {
    fn recognize(url: &str) -> Option<String> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^http://www.hpfanficarchive.com/.*sid=(\d+)").unwrap();
        }
        match RE.captures(url) {
            None => return None,
            Some(m) => match m.at(1) {
                None => return None,
                Some(s) => return Some(s.to_string()),
            },
        }
    }

    fn get_url(id: &str) -> String { format!("http://www.hpfanficarchive.com/stories/viewstory.php?sid={}", id) }

    fn get_info(client: &hyper::client::Client, id: &str) -> Option<StoryInfo> {
        let url = format!("http://www.hpfanficarchive.com/stories/viewstory.php?action=printable&sid={}", id);
        let mut res = client.get(&url).send().unwrap();
        if res.status != hyper::Ok {
            return None;
        }

        let mut html = String::new();
        res.read_to_string(&mut html).unwrap();

        let doc = Html::parse_document(&html);

        let title_elem = doc.select(&Selector::parse("#pagetitle a:nth-child(1)").unwrap()).next();
        if title_elem.is_none() {
            return None;
        }

        let title = title_elem.unwrap().text().next().unwrap();
        let author = doc.select(&Selector::parse("#pagetitle a+ a").unwrap()).next().unwrap().text().next().unwrap();
        let chapters = *&doc.select(&Selector::parse(".label:nth-last-of-type(6)").unwrap()).next().unwrap().next_sibling().unwrap().value().as_text().unwrap().trim();
        let updated = *&doc.select(&Selector::parse(".label:nth-last-of-type(1)").unwrap()).next().unwrap().next_sibling().unwrap().value().as_text().unwrap().trim();

        return Some(StoryInfo {
            title: title.to_string(),
            author: author.to_string(),
            chapters: chapters.parse().unwrap(),
            updated: time::strptime(*&updated, "%B %d, %Y").unwrap().to_timespec().sec,
        });
    }

    fn get_chapter(client: &hyper::client::Client, id: &str, chapter: u32) -> Option<ChapterInfo> {
        let url = format!("http://www.hpfanficarchive.com/stories/viewstory.php?action=printable&sid={}&chapter={}", id, chapter);
        let mut res = client.get(&url).send().unwrap();
        if res.status != hyper::Ok {
            return None;
        }

        let mut html = String::new();
        res.read_to_string(&mut html).unwrap();

        let doc = Html::parse_document(&html);

        let title = doc.select(&Selector::parse(".chaptertitle").unwrap()).next().unwrap().text().next().unwrap().trim();
        let content = doc.select(&Selector::parse(".chapter").unwrap()).next().unwrap();

        return Some(ChapterInfo {
            title: if title == "" { format!("Chapter {}", chapter) } else { title.to_string() },
            content: sanitize(&content),
        });
    }
}
