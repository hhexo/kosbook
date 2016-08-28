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

use rustc_serialize::json;

use std::collections::HashMap;
use regex;

use structure;

#[derive(Clone, PartialEq, RustcDecodable, RustcEncodable)]
pub enum StorageAction {
    StoreSingle,
    StoreVector,
    StoreMapSingle,
    StoreMapVector,
}

#[derive(Clone, PartialEq, RustcDecodable, RustcEncodable)]
pub struct StorageSpec {
    action: StorageAction,
    replace: String,
    variable: String,
    key: String
}

#[derive(Clone, PartialEq, RustcDecodable, RustcEncodable)]
pub struct RuleSpec {
    name: String,
    regex: String,
    replace: String,
    storage: Vec<StorageSpec>
}

#[derive(Clone, PartialEq, RustcDecodable, RustcEncodable)]
pub struct RuleSpecContainer {
    rules: Vec<RuleSpec>
}

impl RuleSpecContainer {
    fn validate_rules(rsc: &RuleSpecContainer) -> Result<(), String> {
        let valid_names = regex::Regex::new("^[_0-9a-zA-Z]+$").unwrap();
        for r in rsc.rules.iter() {
            match regex::Regex::new(&r.regex) {
                Ok(_) => (),
                Err(_) => {
                    return Err("Regular expression '".to_string() + &r.regex +
                               "' in rule '" + &r.name + "' is invalid.");
                }
            }
            for s in r.storage.iter() {
                if !valid_names.is_match(&s.variable) {
                    return Err("Variable name '".to_string() + &s.variable +
                               "' in rule '" + &r.name + "'is invalid. " +
                               "Please only use underscores, digits and " +
                               "ASCII letters.");
                }
            }
        }
        Ok(())
    }

    pub fn from_json(js: &str) -> Result<RuleSpecContainer, String> {
        let rsc = match json::decode::<RuleSpecContainer>(js) {
            Ok(x) => x,
            Err(e) => {
                return Err(
                    (format!("Error parsing rules JSON: {}", e)).to_string());
            }
        };
        match RuleSpecContainer::validate_rules(&rsc) {
            Ok(_) => (),
            Err(e) => { return Err(e); }
        }
        Ok(rsc)
    }
}


struct VarVariant {
    single:     String,
    vector:     Vec<String>,
    map_single: HashMap<String, String>,
    map_vector: HashMap<String, Vec<String>>,
}

pub struct RulesEngine {
    variables: HashMap<String, VarVariant>
}

impl RulesEngine {
    pub fn new() -> RulesEngine {
        RulesEngine {
            variables: HashMap::new()
        }
    }

    pub fn apply_rule(&mut self, rule: &RuleSpec,
                      content: &mut structure::Content) -> Result<(), String> {
        // We have validated regexps before, so this must work.
        let re = regex::Regex::new(&rule.regex).unwrap();
        // Match and perform operations.
        for mut chunk in content.chunks.iter_mut() {
            // Process each match for storage
            for cap in re.captures_iter(&chunk) {
                for s in rule.storage.iter() {
                    let processed_key = cap.expand(s.key.as_str());
                    let valid_key = regex::Regex::new(
                        "^[_0-9a-zA-Z]+$").unwrap();
                    if !valid_key.is_match(&processed_key) {
                        return Err("Captured key '".to_string() +
                                   &processed_key +
                                   "' obtained from text when applying rule '" +
                                   &rule.name +
                                   "' is invalid. Please only use " +
                                   "underscores, digits, and ASCII letters.");
                    }
                    let processed_value = cap.expand(s.replace.as_str());
                    if !self.variables.contains_key(&s.variable) {
                        self.variables.insert(s.variable.clone(), VarVariant {
                            single: String::new(),
                            vector: Vec::new(),
                            map_single: HashMap::new(),
                            map_vector: HashMap::new()
                        });
                    }
                    match s.action {
                        StorageAction::StoreSingle => {
                            self.variables.get_mut(&s.variable).unwrap(
                                ).single = processed_value;
                        },
                        StorageAction::StoreVector => {
                            self.variables.get_mut(&s.variable).unwrap(
                                ).vector.push(processed_value);
                        },
                        StorageAction::StoreMapSingle => {
                            self.variables.get_mut(&s.variable).unwrap(
                                ).map_single.insert(processed_key,
                                                    processed_value);
                        },
                        StorageAction::StoreMapVector => {
                            let must_init = !self.variables.get(&s.variable)
                                            .unwrap().map_vector
                                            .contains_key(&processed_key);
                            if must_init {
                                self.variables.get_mut(&s.variable).unwrap()
                                    .map_vector.insert(
                                        processed_key.clone(), Vec::new());
                            }
                            self.variables.get_mut(&s.variable).unwrap(
                                ).map_vector.get_mut(&processed_key).unwrap(
                                    ).push(processed_value);
                        },
                    }
                }
            }
            // Finally replace all matches
            let new_chunk = re.replace_all(&chunk, rule.replace.as_str());
            // Surely there's an easier way of doing this...
            // Why isn't there a String.swap()?
            chunk.clear();
            chunk.push_str(&new_chunk);
        }
        Ok(())
    }

    pub fn apply_rules(&mut self, rules: &RuleSpecContainer,
                       content: &mut structure::Content) -> Result<(), String> {
        for rule in rules.rules.iter() {
            match self.apply_rule(rule, content) {
                Ok(_) => (),
                Err(e) => { return Err(e); }
            }
        }
        Ok(())
    }

    fn construct_map_content(&self, m: &HashMap<String, String>) -> String {
        m.iter().map(|(k, v)| {
            format!("{}: {}", k, v)
        }).fold(String::new(), |acc, x| {
            acc + "\n\n" + &x
        })
    }

    fn construct_mapv_content(&self, m: &HashMap<String, Vec<String>>) -> String {
        m.iter().map(|(k, v)| {
            format!("{}: {}", k, v.iter().fold(String::new(), |acc, x| {
                acc + " " + &x
            }))
        }).fold(String::new(), |acc, x| {
            acc + "\n\n" + &x
        })
    }

    pub fn substitute_vars(&self, content: &mut structure::Content) 
    -> Result<(), String>  {
        let re_keyed_var = regex::Regex::new(
            r"\{\{\s*([_0-9a-zA-Z]+)\.([_0-9a-zA-Z]+)\s*\}\}").unwrap();
        let re_plain_var = regex::Regex::new(
            r"\{\{\s*([_0-9a-zA-Z]+)\s*\}\}").unwrap();
        for mut chunk in content.chunks.iter_mut() {
            // Loop until there isn't anything to substitute anymore.
            while let Some(m) = re_keyed_var.captures(&chunk.clone()) {
                let var_name = m.at(1).unwrap();
                let var_key = m.at(2).unwrap();
                match self.variables.get(var_name) {
                    Some(var) => {
                        match var.map_single.get(var_key) {
                            Some(value) => {
                                let new_chunk = re_keyed_var.replace(
                                    &chunk, regex::NoExpand(value));
                                // Why isn't there a String.swap()?
                                chunk.clear();
                                chunk.push_str(&new_chunk);
                                continue;
                            },
                            None => ()
                        }
                        match var.map_vector.get(var_key) {
                            Some(vector) => {
                                let new_chunk = re_keyed_var.replace(
                                    &chunk, regex::NoExpand(
                                        &vector.join("\n\n")));
                                // Why isn't there a String.swap()?
                                chunk.clear();
                                chunk.push_str(&new_chunk);
                                continue;
                            },
                            None => ()
                        }
                        return Err("Variable '".to_string() +
                                   &var_name +
                                   "' does not contain key '" +
                                   &var_key + "' at the point of " +
                                   "variable substitution.");
                    },
                    None => {
                        return Err("Variable '".to_string() +
                                   &var_name +
                                   "' is not defined at the point of " +
                                   "variable substitution.");
                    }
                }
            }
            while let Some(m) = re_plain_var.captures(&chunk.clone()) {
                let var_name = m.at(1).unwrap();
                match self.variables.get(var_name) {
                    Some(var) => {
                        if !var.single.is_empty() {
                            let new_chunk = re_plain_var.replace(
                                &chunk, regex::NoExpand(&var.single));
                            // Why isn't there a String.swap()?
                            chunk.clear();
                            chunk.push_str(&new_chunk);
                            continue;
                        }
                        if !var.vector.is_empty() {
                            let new_chunk = re_plain_var.replace(
                                &chunk, regex::NoExpand(
                                    &var.vector.join("\n\n")));
                            // Why isn't there a String.swap()?
                            chunk.clear();
                            chunk.push_str(&new_chunk);
                            continue;
                        }
                        if !var.map_single.is_empty() {
                            let new_chunk = re_plain_var.replace(
                                &chunk, regex::NoExpand(
                                    &self.construct_map_content(
                                        &var.map_single)));
                            // Why isn't there a String.swap()?
                            chunk.clear();
                            chunk.push_str(&new_chunk);
                            continue;
                        }
                        if !var.map_vector.is_empty() {
                            let new_chunk = re_plain_var.replace(
                                &chunk, regex::NoExpand(
                                    &self.construct_mapv_content(
                                        &var.map_vector)));
                            // Why isn't there a String.swap()?
                            chunk.clear();
                            chunk.push_str(&new_chunk);
                            continue;
                        }
                        return Err("Variable '".to_string() +
                                   &var_name +
                                   "' does not contain content " +
                                   "at the point of " +
                                   "variable substitution.");
                    },
                    None => {
                        return Err("Variable '".to_string() +
                                   &var_name +
                                   "' is not defined at the point of " +
                                   "variable substitution.");
                    }
                }
            }
        }
        Ok(())
    }
}
