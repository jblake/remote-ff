use hyper::client::Client;
use parse::*;
use rustc_serialize::json;
use std::fs::File;
use std::io::prelude::*;

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
    if let Some(id) = Ffn::recognize(url) {
        for entry in db.iter() {
            if entry.site == Sitename::Ffn && entry.id == id {
                return;
            }
        }
        if let Some(info) = Ffn::get_info(client, &*id) {
            db.push(Metadata {
                site: Sitename::Ffn,
                id: id.to_string(),
                info: info,
                pruned: false,
            });
        }
    } else if let Some(id) = Hpffa::recognize(url) {
        for entry in db.iter() {
            if entry.site == Sitename::Hpffa && entry.id == id {
                return;
            }
        }
        if let Some(info) = Hpffa::get_info(client, &*id) {
            db.push(Metadata {
                site: Sitename::Hpffa,
                id: id.to_string(),
                info: info,
                pruned: false,
            });
        }
    } else {
        println!("Did not recognize URL: {}", url);
    }
}
