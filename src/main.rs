extern crate clap;
extern crate hyper;
extern crate marksman_escape;
extern crate rustc_serialize;
extern crate scraper;
extern crate time;

mod db;
mod fb2;
mod sanitize;
mod parse;

use clap::{App,Arg,SubCommand};
use parse::Site;

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
        .subcommand(SubCommand::with_name("download")
            .about("Check for updates and rebuild .fb2 files")
            )
        .get_matches();

    let dbpath = args.value_of("dbpath").unwrap_or("db.json");
    let mut meta = db::load(dbpath);
    println!("meta = {:#?}", meta);

    if let Some(subargs) = args.subcommand_matches("download") {
        let mut http = hyper::client::Client::new();
        http.set_redirect_policy(hyper::client::RedirectPolicy::FollowAll);
        http.set_read_timeout(Some(std::time::Duration::new(5, 0)));

        // XXX
    } else {
        println!("You must specify a subcommand! (try \"help\")");
        std::process::exit(1);
    }

    let mut http = hyper::client::Client::new();

    http.set_redirect_policy(hyper::client::RedirectPolicy::FollowAll);
    http.set_read_timeout(Some(std::time::Duration::new(5, 0)));

    let id = "1677";

    match parse::Hpffa::get_info(&http, id) {
        None => println!("Story is not valid, or hpffa is down."),
        Some(story) => {
            let url = parse::Hpffa::get_url(id);
            let mut chapters = Vec::<parse::ChapterInfo>::new();
            for n in 1 .. story.chapters+1 {
                chapters.push(parse::Hpffa::get_chapter(&http, id, n).unwrap());
            }
            print!("{}", fb2::compile(&url, &story, &chapters));
        },
    }
}
