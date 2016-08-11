use rustc_serialize::json;

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
    parts: Vec<Part>
}


impl Structure {
    pub fn from_json(js: &str) -> Structure {
        json::decode::<Structure>(js).unwrap()
    }

    pub fn get_all_files(&self) -> Vec<String> {
        self.parts.iter().map(|ref part| {
            part.chapters.iter().map(|ref chap| {
                &chap.files
            }).fold(Vec::new(), |mut acc, f| {
                acc.extend_from_slice(&f[..]); acc
            })
        }).fold(Vec::new(), |mut acc, f| {
            acc.extend_from_slice(&f[..]); acc
        })
    }
}



#[test]
fn test_structure_1() {
    let js = r#"
    {
        "title": "My Book",
        "parts": [{
            "title": "Part 1",
            "chapters": [{
                "title": "Chap 1",
                "files": ["alpha", "beta"]
            }, {
                "title": "Chap 2",
                "files": ["gamma"]
            }]
        }, {
            "title": "Part 2",
            "chapters": [{
                "title": "Chap 3",
                "files": ["delta"]
            }, {
                "title": "Chap 4",
                "files": ["epsilon", "eta"]
            }]
        }]
    }"#;
    let st = Structure::from_json(&js);
    let fs = st.get_all_files();
    assert!(fs.len() == 6);
}
