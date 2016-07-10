use hyper;
use parse::*;
use sanitize::*;
use scraper::{Html, Selector};
use std::io::Read;

pub struct Ffn { }

impl Site for Ffn {
    fn get_url(id: &str) -> String { format!("https://m.fanfiction.net/s/{}", id) }

    fn get_info(client: &hyper::client::Client, id: &str) -> Option<StoryInfo> {
        println!("Fetching info for ffn:{}", id);

        let url = format!("https://m.fanfiction.net/s/{}", id);
        let mut res = client.get(&url).send().unwrap();
        if res.status != hyper::Ok {
            return None;
        }

        let mut html = String::new();
        res.read_to_string(&mut html).unwrap();

        let doc = Html::parse_document(&html);

        let title_elem = doc.select(&Selector::parse("#content b").unwrap()).next();
        if title_elem.is_none() {
            return None;
        }

        let title = title_elem.unwrap().text().next().unwrap();
        let author = doc.select(&Selector::parse("#content a").unwrap()).next().unwrap().text().next().unwrap();
        let chapters = match doc.select(&Selector::parse("hr+ div > a:nth-child(1)").unwrap()).next() {
            Some(x) => x.text().next().unwrap(),
            None => "1",
        };
        let updated = match doc.select(&Selector::parse("#content span+ span").unwrap()).next() {
            Some(x) => x.value().attr("data-xutime"),
            None => None,
        };
        let published = doc.select(&Selector::parse("#content span").unwrap()).next().unwrap().value().attr("data-xutime");

        return Some(StoryInfo {
            title: title.to_string(),
            author: author.to_string(),
            chapters: chapters.parse::<u32>().unwrap(),
            updated: updated.or(published).unwrap().parse::<i64>().unwrap(),
        });
    }

    fn get_chapter(client: &hyper::client::Client, id: &str, chapter: u32) -> Option<ChapterInfo> {
        println!("Fetching chapter ffn:{}:{}", id, chapter);

        let url = format!("https://m.fanfiction.net/s/{}/{}", id, chapter);
        let mut res = client.get(&url).send().unwrap();
        if res.status != hyper::Ok {
            return None;
        }

        let mut html = String::new();
        res.read_to_string(&mut html).unwrap();

        let doc = Html::parse_document(&html);

        let title = doc.select(&Selector::parse("#content").unwrap()).next().unwrap().text().last().unwrap().trim();
        let content = doc.select(&Selector::parse("#storycontent").unwrap()).next().unwrap();

        return Some(ChapterInfo {
            title: if title == "" { format!("Chapter {}", chapter) } else { title.to_string() },
            content: sanitize(&content),
        });
    }
}
