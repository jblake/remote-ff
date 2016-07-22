use hyper::client::Client;
use parse::*;
use regex::Regex;
use rusqlite;
use rustc_serialize::json;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
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
        static ref INVALID_PATH_ELEMENTS : Regex = Regex::new(r"[^a-zA-Z0-9]+").unwrap();
        static ref UNDERSCORE_PREFIX : Regex = Regex::new(r"^_+").unwrap();
        static ref UNDERSCORE_SUFFIX : Regex = Regex::new(r"_+$").unwrap();
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
    let fb2path = Path::new(fb2path);
    for entry in db.iter_mut() {
        let path = fb2path.join(&*entry.filename);
        if entry.pruned {
            // Ignoring result of remove_file because e.g. the file may already not exist.
            let _ = std::fs::remove_file(&path);
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
    }
}

pub fn sync(db: &Vec<Metadata>, fb2path: &str, peer: &str) {
    let peerdbpath = Path::new(peer).join("data/data/com.flyersoft.moonreaderp/databases/mrbooks.db");
    let peerfb2path = Path::new(peer).join("storage/emulated/0/Books");
    let peerprefix = Path::new("/sdcard/Books");
    let fb2path = Path::new(fb2path);
    let peerdb = rusqlite::Connection::open(peerdbpath).unwrap();
    for entry in db.iter() {
        let peerpath = peerfb2path.join(&*entry.filename);
        let selfpath = fb2path.join(&*entry.filename);
        let sqlpath = peerprefix.join(&*entry.filename);
        let sqlpathstr = sqlpath.to_str().unwrap().to_string();
        let sqlpathlstr = sqlpathstr.to_lowercase();
        let timestr = format!("{}", entry.info.updated);
        let urlstr = match entry.site {
            Sitename::Ffn => Ffn::get_url(&*entry.id),
            Sitename::Hpffa => Hpffa::get_url(&*entry.id),
        };
        if entry.pruned {
            // Ignoring result of remove_file because e.g. the file may already not exist.
            if let Ok(_) = std::fs::remove_file(&peerpath) {
                peerdb.execute("DELETE FROM books WHERE lowerFilename = ?", &[&sqlpathlstr]).unwrap();
                println!("Pruned {}", entry.filename);
            }
        } else {
            let mut stmt = peerdb.prepare("SELECT addTime FROM books WHERE lowerFilename = ?").unwrap();
            let mut rows = stmt.query(&[&sqlpathlstr]).unwrap();
            if let Some(row) = rows.next() {
                let timestring : String = row.unwrap().get(0);
                let time : i64 = timestring.parse().unwrap();
                if time < entry.info.updated {
                    std::fs::copy(&selfpath, &peerpath).unwrap();
                    peerdb.execute("UPDATE books SET book = ?, author = ?, addTime = ?, favorite = 'default_fav', downloadUrl = ? where lowerFilename = ?", &[&entry.info.title, &entry.info.author, &timestr, &urlstr, &sqlpathlstr]).unwrap();
                    println!("Updated {}", entry.filename);
                }
            } else {
                std::fs::copy(&selfpath, &peerpath).unwrap();
                peerdb.execute("INSERT INTO books (book, filename, lowerFilename, author, description, category, thumbFile, coverFile, addTime, favorite, downloadUrl, rate, bak1, bak2) values (?, ?, ?, ?, '', '', '', '', ?, 'default_fav', ?, '', '', '')", &[&entry.info.title, &sqlpathstr, &sqlpathlstr, &entry.info.author, &timestr, &urlstr]).unwrap();
                println!("Created {}", entry.filename);
            }
        }
    }
}
