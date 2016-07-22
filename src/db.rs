use hyper::client::Client;
use parse::*;
use regex::Regex;
use rustc_serialize::json;
use std::fs::File;
use std::io::prelude::*;
use std;

#[derive(Clone, Debug, Hash, Eq, Ord, PartialEq, PartialOrd, RustcDecodable, RustcEncodable)]
pub enum Sitename {
    Ffn,
    Hpffa,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, RustcDecodable, RustcEncodable)]
pub struct Metadata {
    pub site: Sitename,
    pub id: String,
    pub info: StoryInfo,
    pub pruned: bool,
    pub filename: String,
}

pub fn load(path: &str) -> Vec<Metadata> {
    if let Ok(mut fh) = File::open(path) {
        let mut data = String::new();
        fh.read_to_string(&mut data).unwrap();
        return json::decode(&*data).unwrap();
    } else {
        return Vec::new();
    }
}

pub fn save(path: &str, data: &Vec<Metadata>) {
    let mut fh = File::create(path).unwrap();
    fh.write_all(json::encode(data).unwrap().as_bytes()).unwrap();
}

pub fn add(db: &mut Vec<Metadata>, url: &str, client: &Client) {
    lazy_static! {
        static ref INVALID_PATH_ELEMENTS: Regex = Regex::new(r"[^a-zA-Z0-9]+").unwrap();
        static ref UNDERSCORE_PREFIX: Regex = Regex::new(r"^_+").unwrap();
        static ref UNDERSCORE_SUFFIX: Regex = Regex::new(r"_+$").unwrap();
    }
    if let Some(id) = Ffn::recognize(url) {
        for entry in db.iter() {
            if entry.site == Sitename::Ffn && entry.id == id {
                return;
            }
        }
        if let Some(info) = Ffn::get_info(client, &*id) {
            let filetitle = UNDERSCORE_PREFIX.replace_all(&*UNDERSCORE_SUFFIX.replace_all(&*INVALID_PATH_ELEMENTS.replace_all(&*info.title, "_"), ""), "");
            let fileauthor = UNDERSCORE_PREFIX.replace_all(&*UNDERSCORE_SUFFIX.replace_all(&*INVALID_PATH_ELEMENTS.replace_all(&*info.author, "_"), ""), "");
            let filename = format!("{}.ffn{}.{}.{}.fb2", db.len(), id, filetitle, fileauthor);
            db.push(Metadata {
                site: Sitename::Ffn,
                id: id.to_string(),
                info: info,
                pruned: false,
                filename: filename,
            });
        } else {
            println!("Recognized URL as Ffn, but could not retrieve metadata: {}", url);
        }
    } else if let Some(id) = Hpffa::recognize(url) {
        for entry in db.iter() {
            if entry.site == Sitename::Hpffa && entry.id == id {
                return;
            }
        }
        if let Some(info) = Hpffa::get_info(client, &*id) {
            let filetitle = UNDERSCORE_PREFIX.replace_all(&*UNDERSCORE_SUFFIX.replace_all(&*INVALID_PATH_ELEMENTS.replace_all(&*info.title, "_"), ""), "");
            let fileauthor = UNDERSCORE_PREFIX.replace_all(&*UNDERSCORE_SUFFIX.replace_all(&*INVALID_PATH_ELEMENTS.replace_all(&*info.author, "_"), ""), "");
            let filename = format!("{}.hpffa{}.{}.{}.fb2", db.len(), id, filetitle, fileauthor);
            db.push(Metadata {
                site: Sitename::Hpffa,
                id: id.to_string(),
                info: info,
                pruned: false,
                filename: filename,
            });
        } else {
            println!("Recognized URL as Hpffa, but could not retrieve metadata: {}", url);
        }
    } else {
        println!("Did not recognize URL: {}", url);
    }
}

pub fn download(db: &mut Vec<Metadata>, fb2path: &str, client: &Client) {
    std::fs::DirBuilder::new()
        .recursive(true)
        .create(fb2path)
        .unwrap();
    for entry in db.iter_mut() {
        let path = std::path::Path::new(fb2path);
        let path = path.join(&*entry.filename);
        if entry.pruned {
            // Ignoring result of remove_file because e.g. the file may already not exist.
            let _ = std::fs::remove_file(path.to_str().unwrap());
            continue;
        }
        let info = match entry.site {
            Sitename::Ffn => Ffn::get_info(client, &*entry.id),
            Sitename::Hpffa => Hpffa::get_info(client, &*entry.id),
        };
        if let Some(info) = info {
            if info != entry.info || ! path.exists() {
                let book = match entry.site {
                    Sitename::Ffn => Ffn::compile(client, &*entry.id, &info),
                    Sitename::Hpffa => Hpffa::compile(client, &*entry.id, &info),
                };
                if let Some(book) = book {
                    let mut fh = File::create(path).unwrap();
                    fh.write_all(book.as_bytes()).unwrap();
                    entry.info = info;
                    println!("Wrote new version of {}", entry.filename);
                } else {
                    println!("Could not download updated chapters for {}", entry.filename);
                }
            }
        } else {
            println!("Could not download info for {}", entry.filename);
        }
    };
}
