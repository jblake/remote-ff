use marksman_escape::Escape;
use scraper::element_ref::ElementRef;
use scraper::node::Node;

fn sanitize_into(root: &ElementRef, buf: &mut String, inp: bool) {
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
                    buf.push_str("</p>");
                }
            },
            &Node::Element(ref elem) => {
                match elem.name() {
                    "p" => {
                        if inp {
                            buf.push_str("</p>\n");
                        }
                        buf.push_str("<p>");
                        sanitize_into(&ElementRef::wrap(node).unwrap(), buf, true);
                        buf.push_str("</p>\n");
                        if inp {
                            buf.push_str("<p>");
                        }
                    },
                    "hr" => {
                        if inp {
                            //buf.push_str("</p>\n<empty-line/>\n<p>");
                            buf.push_str("<!-- bad hr inside p -->");
                        } else {
                            buf.push_str("<empty-line/>\n");
                        }
                    },
                    "br" => {
                        if inp {
                            //buf.push_str("</p>\n<empty-line/>\n<p>");
                            buf.push_str("<!-- bad br inside p -->");
                        } else {
                            buf.push_str("<empty-line/>\n");
                        }
                    },
                    "b" => {
                        if ! inp {
                            buf.push_str("<p>");
                        }
                        buf.push_str("<strong>");
                        sanitize_into(&ElementRef::wrap(node).unwrap(), buf, true);
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
                        sanitize_into(&ElementRef::wrap(node).unwrap(), buf, true);
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
                        sanitize_into(&ElementRef::wrap(node).unwrap(), buf, true);
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
                        sanitize_into(&ElementRef::wrap(node).unwrap(), buf, true);
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
                        sanitize_into(&ElementRef::wrap(node).unwrap(), buf, true);
                        buf.push_str("</emphasis></strong>");
                        if ! inp {
                            buf.push_str("</p>\n");
                        }
                    },
                    x => panic!("Unrecognized tag: {}\n{:#?}\n", x, elem),
                }
            },
        }
    }
}

pub fn sanitize(root: &ElementRef) -> String {
    let mut buf = String::new();
    sanitize_into(root, &mut buf, false);
    return buf;
}
