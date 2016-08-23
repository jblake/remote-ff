use marksman_escape::Escape;
use scraper::element_ref::ElementRef;
use scraper::node::Node;
use std::collections::VecDeque;

fn newp(buf: &mut String, inp: bool, formatting: &VecDeque<&str>) {
    for node in formatting.iter().rev() {
        buf.push_str("</");
        buf.push_str(*node);
        buf.push_str(">");
    }
    if inp {
        buf.push_str("</p>\n");
    }
    buf.push_str("<p>");
    for node in formatting.iter() {
        buf.push_str("<");
        buf.push_str(*node);
        buf.push_str(">");
    }
}

fn sanitize_into(root: &ElementRef, buf: &mut String, inp: bool, formatting: &mut VecDeque<&str>) {
    assert!(inp || formatting.is_empty(), "Non-empty formatting table without being inside paragraph: inp={:?} {:?}", inp, formatting);
    for node in root.children() {
        match node.value() {
            &Node::Document => panic!("unexpected document"),
            &Node::Fragment => panic!("unexpected fragment"),
            &Node::Doctype(_) => {},
            &Node::Comment(_) => {},
            &Node::Text(ref text) => {
                let blank = text.trim() == "";
                if !blank && !inp {
                    buf.push_str("<p>");
                }
                if !blank || inp {
                    buf.push_str(&*String::from_utf8(Escape::new(text.bytes()).collect()).unwrap());
                }
                if !blank && !inp {
                    buf.push_str("</p>\n");
                }
            },
            &Node::Element(ref elem) => {
                match elem.name() {
                    "a" => {
                        sanitize_into(&ElementRef::wrap(node).unwrap(), buf, inp, formatting);
                    },
                    "img" => {
                        sanitize_into(&ElementRef::wrap(node).unwrap(), buf, inp, formatting);
                    },
                    "center" => {
                        newp(buf, inp, formatting);
                        sanitize_into(&ElementRef::wrap(node).unwrap(), buf, true, formatting);
                        if !inp {
                            buf.push_str("</p>\n");
                        }
                    },
                    "p" => {
                        newp(buf, inp, formatting);
                        sanitize_into(&ElementRef::wrap(node).unwrap(), buf, true, formatting);
                        if !inp {
                            buf.push_str("</p>\n");
                        }
                    },
                    "blockquote" => {
                        newp(buf, inp, formatting);
                        sanitize_into(&ElementRef::wrap(node).unwrap(), buf, true, formatting);
                        if !inp {
                            buf.push_str("</p>\n");
                        }
                    },
                    "li" => {
                        newp(buf, inp, formatting);
                        sanitize_into(&ElementRef::wrap(node).unwrap(), buf, true, formatting);
                        if !inp {
                            buf.push_str("</p>\n");
                        }
                    },
                    "hr" => {
                        for node in formatting.iter().rev() {
                            buf.push_str("</");
                            buf.push_str(*node);
                            buf.push_str(">");
                        }
                        if inp {
                            buf.push_str("</p>\n");
                        }
                        buf.push_str("<empty-line/>\n");
                        if inp {
                            buf.push_str("<p>");
                        }
                        for node in formatting.iter().rev() {
                            buf.push_str("<");
                            buf.push_str(*node);
                            buf.push_str(">");
                        }
                    },
                    "br" => {
                        newp(buf, inp, formatting);
                    },
                    "b" => {
                        if ! inp {
                            buf.push_str("<p>");
                        }
                        buf.push_str("<strong>");
                        formatting.push_back("strong");
                        sanitize_into(&ElementRef::wrap(node).unwrap(), buf, true, formatting);
                        formatting.pop_back();
                        buf.push_str("</strong>");
                        if ! inp {
                            buf.push_str("</p>\n");
                        }
                    },
                    "strong" => {
                        if ! inp {
                            buf.push_str("<p>");
                        }
                        buf.push_str("<strong>");
                        formatting.push_back("strong");
                        sanitize_into(&ElementRef::wrap(node).unwrap(), buf, true, formatting);
                        formatting.pop_back();
                        buf.push_str("</strong>");
                        if ! inp {
                            buf.push_str("</p>\n");
                        }
                    },
                    "i" => {
                        if ! inp {
                            buf.push_str("<p>");
                        }
                        buf.push_str("<emphasis>");
                        formatting.push_back("emphasis");
                        sanitize_into(&ElementRef::wrap(node).unwrap(), buf, true, formatting);
                        formatting.pop_back();;
                        buf.push_str("</emphasis>");
                        if ! inp {
                            buf.push_str("</p>\n");
                        }
                    },
                    "em" => {
                        if ! inp {
                            buf.push_str("<p>");
                        }
                        buf.push_str("<emphasis>");
                        formatting.push_back("emphasis");
                        sanitize_into(&ElementRef::wrap(node).unwrap(), buf, true, formatting);
                        formatting.pop_back();
                        buf.push_str("</emphasis>");
                        if ! inp {
                            buf.push_str("</p>\n");
                        }
                    },
                    "span" => {
                        if ! inp {
                            buf.push_str("<p>");
                        }
                        buf.push_str("<strong><emphasis>");
                        formatting.push_back("strong");
                        formatting.push_back("emphasis");
                        sanitize_into(&ElementRef::wrap(node).unwrap(), buf, true, formatting);
                        formatting.pop_back();
                        formatting.pop_back();
                        buf.push_str("</emphasis></strong>");
                        if ! inp {
                            buf.push_str("</p>\n");
                        }
                    },
                    "u" => {
                        if ! inp {
                            buf.push_str("<p>");
                        }
                        buf.push_str("<strong><emphasis>");
                        formatting.push_back("strong");
                        formatting.push_back("emphasis");
                        sanitize_into(&ElementRef::wrap(node).unwrap(), buf, true, formatting);
                        formatting.pop_back();
                        formatting.pop_back();
                        buf.push_str("</emphasis></strong>");
                        if ! inp {
                            buf.push_str("</p>\n");
                        }
                    },
                    "ol" => {
                        sanitize_into(&ElementRef::wrap(node).unwrap(), buf, inp, formatting);
                    },
                    "ul" => {
                        sanitize_into(&ElementRef::wrap(node).unwrap(), buf, inp, formatting);
                    },
                    x => panic!("Unrecognized tag: {}\n{:#?}\n", x, elem),
                }
            },
        }
    }
}

pub fn sanitize(root: &ElementRef) -> String {
    let mut buf = String::new();
    let mut formatting = VecDeque::new();
    sanitize_into(root, &mut buf, false, &mut formatting);
    assert!(formatting.is_empty(), "Somehow wound up with nonempty formatting stack: {:?}", formatting);
    return buf;
}
