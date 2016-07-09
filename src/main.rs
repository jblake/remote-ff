extern crate hyper;
extern crate marksman_escape;
extern crate scraper;
extern crate time;

mod fb2;
mod sanitize;
mod parse;

use parse::Site;

fn main() {
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
