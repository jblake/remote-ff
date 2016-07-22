extern crate clap;
extern crate hyper;
#[macro_use] extern crate lazy_static;
extern crate marksman_escape;
extern crate regex;
extern crate rustc_serialize;
extern crate scraper;
extern crate time;

mod db;
mod fb2;
mod sanitize;
mod parse;

use clap::{App,Arg,SubCommand};

fn main() {
    let args = App::new("remote-ff")
        .author("Julian Blake Kongslie <jblake@jblake.org>")
        .about("Synchronization of ebooks with moonreader")
        .arg(Arg::with_name("fb2path")
             .short("b")
             .long("books")
             .takes_value(true)
             .value_name("DIR")
             .help("Directory to store .fb2 files (default \"books\")")
             )
        .arg(Arg::with_name("dbpath")
             .short("d")
             .long("database")
             .takes_value(true)
             .value_name("FILE")
             .help("Database file for book metadata (default \"db.json\")")
             )
        .subcommand(SubCommand::with_name("add")
            .about("Add a new ebook")
            .arg(Arg::with_name("url")
                 .required(true)
                 .multiple(true)
                 .takes_value(true)
                 .value_name("URL")
                 .help("Url to add")
                 )
            )
        .subcommand(SubCommand::with_name("download")
            .about("Check for updates and rebuild .fb2 files")
            )
        .subcommand(SubCommand::with_name("prune")
            .about("Stop checking for updates for an ebook")
            .arg(Arg::with_name("id")
                 .required(true)
                 .multiple(true)
                 .takes_value(true)
                 .value_name("ID")
                 .help("Story ID to prune")
                 )
            )
        .subcommand(SubCommand::with_name("sync")
            .about("Synchronize with a moonreader instance")
            .arg(Arg::with_name("mrpath")
                 .required(true)
                 .takes_value(true)
                 .value_name("MOUNTPOINT")
                 .help("Mountpoint of your android device's filesystem root")
                 )
            )
        .subcommand(SubCommand::with_name("webapi")
            .about("Web API endpoint")
            )
        .get_matches();

    let dbpath = args.value_of("dbpath").unwrap_or("db.json");
    let mut meta = db::load(dbpath);

    let mut http = hyper::client::Client::new();
    http.set_redirect_policy(hyper::client::RedirectPolicy::FollowAll);
    http.set_read_timeout(Some(std::time::Duration::new(5, 0)));

    if let Some(subargs) = args.subcommand_matches("add") {
        for url in subargs.values_of("url").unwrap() {
            db::add(&mut meta, &url, &http);
        }
    } else if let Some(subargs) = args.subcommand_matches("download") {
        db::download(&mut meta, args.value_of("fb2path").unwrap_or("books"), &http);
    } else if let Some(subargs) = args.subcommand_matches("prune") {
        for n in subargs.values_of("id").unwrap() {
            meta[n.parse::<usize>().unwrap()].pruned = true;
        }
    } else if let Some(subargs) = args.subcommand_matches("sync") {
        // XXX
    } else if let Some(subargs) = args.subcommand_matches("webapi") {
        // XXX
    } else {
        println!("You must specify a subcommand! (try \"help\")");
        std::process::exit(1);
    }

    db::save(dbpath, &meta);
}
