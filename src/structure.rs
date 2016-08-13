use rustc_serialize::json;

use std::io::prelude::*;
use std::fs::File;

#[derive(Clone, PartialEq, RustcDecodable, RustcEncodable)]
pub struct Chapter {
    title: String,
    files: Vec<String>
}

#[derive(Clone, PartialEq, RustcDecodable, RustcEncodable)]
pub struct Part {
    title: String,
    chapters: Vec<Chapter>
}

#[derive(Clone, PartialEq, RustcDecodable, RustcEncodable)]
pub struct Structure {
    title: String,
    author: String,
    license: String,
    parts: Vec<Part>
}

impl Structure {
    pub fn from_json(js: &str) -> Result<Structure, json::DecoderError> {
        json::decode::<Structure>(js)
    }

    pub fn get_title(&self) -> &str { &self.title }
}


#[derive(Clone, PartialEq)]
pub struct Content {
    chunks: Vec<String>
}

impl Content {
    fn build_title_page(st: &Structure) -> Result<String, String> {
        let book_header = 
                r#"<div class="book_author">"#.to_string() +
                &st.author +
                r#"</div>"# + 
                r#"<div class="book_title">"# +
                r#"<a id="kos_book_title">"# +
                &st.title +
                "</a></div>\n\n" +
                r#"<div class="license">(C) "# +
                &st.author +
                " - " +
                &st.license +
                "</div>\n\n";
        Ok(book_header)
    }

    fn build_toc(st: &Structure) -> Result<String, String> {
        let mut toc = String::new();
        toc = toc + r#"<div class="toc">"# + "\n\n";
        let mut part_index = 1;
        for part in st.parts.iter() {
            let part_link = format!(
                "- **[{0} {1}](#kos_ref_part_{0})**\n\n", part_index, part.title);
            toc = toc + &part_link;
            let mut chap_index = 1;
            for chap in part.chapters.iter() {
                let chap_link = format!(
                    "   - *[{0}.{1} {2}](#kos_ref_chap_{0}_{1})*\n\n",
                    part_index, chap_index, chap.title);
                toc = toc + &chap_link;
                chap_index += 1;
            }
            part_index += 1;
        }
        toc = toc + "</div>\n\n";
        Ok(toc)
    }

    fn build_chunks(st: &Structure) -> Result<Vec<String>, String> {
        let mut chunks = Vec::new();
        // Book cover first...
        match Content::build_title_page(st) {
            Ok(tp) => { chunks.push(tp); },
            Err(e) => { return Err(e); }
        }
        // Then TOC...
        match Content::build_toc(st) {
            Ok(toc) => { chunks.push(toc); },
            Err(e) => { return Err(e); }
        }
        // Then parts and chapters.
        let mut part_index = 1;
        for part in st.parts.iter() {
            let part_header = 
                r#"<div class="part_title">"#.to_string() + 
                r#"<a id="kos_ref_part_"# +
                &format!("{}", part_index) +
                r#"">"# +
                &part_index.to_string() +
                " " +
                &part.title +
                "</a></div>\n\n";
            chunks.push(part_header);
            let mut chap_index = 1;
            for chap in part.chapters.iter() {
                let chap_header = 
                    r#"# <a id="kos_ref_chap_"#.to_string() +
                    &format!("{}", part_index) +
                    "_" +
                    &format!("{}", chap_index) +
                    r#""> "# +
                    &part_index.to_string() +
                    "." +
                    &chap_index.to_string() +
                    " " +
                    &chap.title +
                    "</a>\n\n";
                chunks.push(chap_header);
                for f in chap.files.iter() {
                    let file_content = match File::open(f) {
                        Ok(mut fread) => {
                            let mut res = String::new();
                            match fread.read_to_string(&mut res) {
                                Ok(_) => (),
                                Err(_) => {
                                    return Err(
                                        "Error reading file ".to_string() +
                                        f + "!\n");
                                }
                            }
                            res
                        },
                        Err(_) => {
                            return Err(
                                "Error reading file ".to_string() + f + "!\n");
                        }
                    };
                    chunks.push(file_content);
                }
                chap_index += 1;
            }
            part_index += 1;
        }
        Ok(chunks)
    }

    pub fn from_structure(st: &Structure) -> Result<Content, String> {
        let chunks = Content::build_chunks(st);
        Ok(Content {
            chunks: match chunks {
                Ok(c) => c,
                Err(e) => { return Err(e); }
            }
        })
    }

    pub fn to_single_string(&self) -> String {
        self.chunks.iter().fold(String::new(), |acc, x| {
            acc + "\n\n" + &x
        })
    }
}
