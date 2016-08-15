// Copyright 2016 Dario Domizioli
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate rustc_serialize;
extern crate getopts;
extern crate pulldown_cmark;
extern crate regex;

use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;
use std::env;

mod structure;
mod rules;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn html_prologue(style: &str, title: &str) -> String {
    return r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="generator" content="kosbook">
    <title>"#.to_string() +
    title +
    r#"</title>
    <link rel="stylesheet" type="text/css" href=""# +
    style +
    r#"">
</head>
<body>

"#;
}

fn html_epilogue() -> String {
    return "\n\n</body>\n</html>\n".to_string();
}

fn main() {
    // Command-line options
    let args: Vec<_> = env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print help message and exit");
    opts.optopt("i", "input",
                "specify input structure file (default: ./structure.json)",
                "FILE");
    opts.optopt("o", "output", 
                "specify output file (default: ./output.html)",
                "FILE");
    opts.optflag("p", "pdf",
                 "also invoke 'wkhtmltopdf' to produce a pdf. Note that wkhtmltopdf must be in your PATH.");
    opts.optopt("r", "rules",
                "specify the processing rules file (default: ./rules.json)",
                "FILE");
    opts.optopt("s", "style", 
                "specify custom path to CSS file (default: style.css)",
                "FILE");
    opts.optflag("v", "version", "print version and exit");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("error:   {}", f.to_string());
            std::process::exit(1);
        }
    };
    if matches.opt_present("help") {
        let brief = format!(
            "\nUsage: {} [options]", args[0].clone());
        println!("{}\n\nAll paths in the structure file are relative to the directory the program is invoked in.", opts.usage(&brief));
        return;
    }
    if matches.opt_present("version") {
        println!("kosbook {}\n(C) Dario Domizioli. Licensed under the Apache License Version 2.0.\n", VERSION);
        return;
    }

    // Collect book structure and generate initial content
    println!("Collecting book structure...");
    let mut structure_file = "structure.json".to_string();
    if let Some(filename) = matches.opt_str("input") {
        structure_file = filename;
    }
    let structure_json = match File::open(structure_file) {
        Ok(mut fread) => {
            let mut res = String::new();
            match fread.read_to_string(&mut res) {
                Ok(_) => (),
                Err(_) => {
                    println!("error:   error reading structure file.");
                    std::process::exit(1);
                }
            }
            res
        },
        Err(_) => {
            println!("error:   error opening structure file.");
            std::process::exit(1);
        }
    };
    let structure = match structure::Structure::from_json(&structure_json) {
        Ok(s) => s,
        Err(e) => {
            println!("error:   {}", e);
            std::process::exit(1);
        }
    };
    println!("Generating in-memory content...");
    let mut content = match structure::Content::from_structure(&structure) {
        Ok(x) => x,
        Err(e) => {
            println!("error:   {}", e);
            std::process::exit(1);
        }
    };

    // Process content through rules
    let mut rule_engine = rules::RulesEngine::new();

    println!("Loading rules file...");
    let mut pre_rules_file = "rules.json".to_string();
    if let Some(filename) = matches.opt_str("rules") {
        pre_rules_file = filename;
    }
    let pre_rules_json = match File::open(pre_rules_file) {
        Ok(mut fread) => {
            let mut res = String::new();
            match fread.read_to_string(&mut res) {
                Ok(_) => (),
                Err(_) => {
                    println!("error:   error reading rules file.");
                    std::process::exit(1);
                }
            }
            res
        },
        Err(_) => {
            println!("error:   error opening rules file.");
            std::process::exit(1);
        }
    };
    let rules = match rules::RuleSpecContainer::from_json(&pre_rules_json) {
        Ok(x) => x,
        Err(e) => {
            println!("error:   {}", e);
            std::process::exit(1);
        }
    };
    println!("Applying rules...");
    match rule_engine.apply_rules(&rules, &mut content) {
        Ok(_) => (),
        Err(e) => {
            println!("error:   {}", e);
            std::process::exit(1);
        }
    }

    // Substitute variables
    println!("Substituting variables...");
    match rule_engine.substitute_vars(&mut content) {
        Ok(_) => (),
        Err(e) => {
            println!("error:   {}", e);
            std::process::exit(1);
        }
    }
    
    // Collate all processed content
    println!("Collating...");
    let collected_string = content.to_single_string();

    // Render CommonMark to HTML
    println!("Rendering to HTML...");
    let mut opts = pulldown_cmark::Options::empty();
    opts.insert(pulldown_cmark::OPTION_ENABLE_TABLES);
    opts.insert(pulldown_cmark::OPTION_ENABLE_FOOTNOTES);
    let mut gen_html = String::with_capacity(collected_string.len() * 3 / 2);
    let p = pulldown_cmark::Parser::new_ext(&collected_string, opts);
    pulldown_cmark::html::push_html(&mut gen_html, p);

    // Write output html, wrapped in prologue and epilogue.
    println!("Writing output HTML file...");
    let mut output_file = "output.html".to_string();
    if let Some(filename) = matches.opt_str("output") {
        output_file = filename;
    }
    let mut style_file = "style.css".to_string();
    if let Some(filename) = matches.opt_str("style") {
        style_file = filename;
    }
    match File::create(&output_file) {
        Ok(f) => {
            let mut writer = BufWriter::new(f);
            match writer.write(
                html_prologue(
                    &style_file, structure.get_title()).as_bytes()) {
                Ok(_) => (),
                Err(e) => {
                    println!("error:   {}", e);
                    std::process::exit(1);
                }
            };
            match writer.write(gen_html.as_bytes()) {
                Ok(_) => (),
                Err(e) => {
                    println!("error:   {}", e);
                    std::process::exit(1);
                }
            };
            match writer.write(html_epilogue().as_bytes()) {
                Ok(_) => (),
                Err(e) => {
                    println!("error:   {}", e);
                    std::process::exit(1);
                }
            };
        },
        Err(e) => {
            println!("error:   {}", e);
            std::process::exit(1);
        }
    }

    // Finally do the PDF conversion if required
    if matches.opt_present("pdf") {
        println!("Invoking wkhtmltopdf to create PDF file...");
        let pdf_file = output_file.trim_right_matches(".html").to_string() + 
                       ".pdf";
        let output = std::process::Command::new("wkhtmltopdf")
                     .arg("--page-size")
                     .arg("A4")
                     .arg("-T")
                     .arg("25mm")
                     .arg("-B")
                     .arg("20mm")
                     .arg("-L")
                     .arg("15mm")
                     .arg("-R")
                     .arg("15mm")
                     .arg("--footer-center")
                     .arg("[page]")
                     .arg("--outline-depth")
                     .arg("2")
                     .arg(&output_file)
                     .arg(&pdf_file)
                     .output();
        match output {
            Ok(_) => (),
            Err(e) => {
                println!("error:   {}", e);
                std::process::exit(1);
            }
        }
    }

    println!("Done!");
}
