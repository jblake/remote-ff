use marksman_escape::Escape;
use parse;
use time;

pub fn compile(url: &str, story: &parse::StoryInfo, chapters: &Vec<parse::ChapterInfo>) -> String {
    let mut fb2 = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<FictionBook xmlns=\"http://www.gribuser.ru/xml/fictionbook/2.0\">\n<description>\n<title-info>\n<author><nickname>".to_string();
    let now = time::now_utc();
    let updated = time::at(time::Timespec::new(story.updated, 0));
    fb2.push_str(&*String::from_utf8(Escape::new(story.author.bytes()).collect()).unwrap());
    fb2.push_str("</nickname></author>\n<book-title>");
    fb2.push_str(&*String::from_utf8(Escape::new(story.title.bytes()).collect()).unwrap());
    fb2.push_str("</book-title>\n<date>");
    fb2.push_str(&*String::from_utf8(Escape::new(time::strftime("%Y-%m-%d %H:%M:%S UTC", &updated).unwrap().bytes()).collect()).unwrap());
    fb2.push_str("</date>\n</title-info>\n<document-info>\n<author><nickname>jblake</nickname></author>\n<date>");
    fb2.push_str(&*String::from_utf8(Escape::new(time::strftime("%Y-%m-%d %H:%M:%S UTC", &now).unwrap().bytes()).collect()).unwrap());
    fb2.push_str("</date>\n<program-used>remote-ff 3</program-used>\n<src-url>");
    fb2.push_str(&*String::from_utf8(Escape::new(url.bytes()).collect()).unwrap());
    fb2.push_str("</src-url>\n</document-info>\n</description>\n<body>\n");
    for chapter in chapters {
        fb2.push_str("<section>\n<title><p>");
        fb2.push_str(&*String::from_utf8(Escape::new(chapter.title.bytes()).collect()).unwrap());
        fb2.push_str("</p></title>\n");
        fb2.push_str(&*chapter.content);
        fb2.push_str("</section>\n");
    }
    fb2.push_str("</body>\n</FictionBook>\n");
    return fb2;
}
