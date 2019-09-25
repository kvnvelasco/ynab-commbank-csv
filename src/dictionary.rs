use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use regex::Regex;
use serde::de::Visitor;
use serde::export::fmt::Error;
use serde::export::Formatter;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::io::Write;

#[derive(Debug)]
pub struct SerdeRegex(pub Regex);

#[derive(Serialize, Deserialize, Debug)]
pub struct DictionaryEntry {
    pub name: String,
    pub regex: SerdeRegex,
    pub memos: Option<Vec<Memo>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Memo {
    regex: SerdeRegex,
    text: String,
}

impl Serialize for SerdeRegex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.as_str())
    }
}

impl<'de> Deserialize<'de> for SerdeRegex {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        struct RegexVisitor;
        impl<'de> Visitor<'de> for RegexVisitor {
            type Value = SerdeRegex;

            fn expecting(&self, formatter: &mut Formatter) -> Result<(), Error> {
                formatter.write_str("an regex string parseable by regex-rs")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
                let regex = Regex::from_str(v).expect("shit");
                Ok(SerdeRegex(regex))
            }
        }

        deserializer.deserialize_string(RegexVisitor)
    }
}

impl SerdeRegex {
    pub fn as_regex(&self) -> &Regex {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Dictionary(pub Vec<DictionaryEntry>);

impl Dictionary {
    pub fn new() -> Self {
        Dictionary(vec![])
    }

    pub fn new_entry(&mut self, regex_string: &str) -> &mut Self {
        let entry = DictionaryEntry {
            name: "".to_string(),
            regex: SerdeRegex(Regex::new(regex_string).unwrap()),
            memos: Option::Some(vec![]),
        };
        let mut next_vec = vec![entry];
        next_vec.append(&mut self.0);
        self.0 = next_vec;
        self
    }
}

pub struct FileBackedDictionary {
    dictionary: Dictionary,
    path: PathBuf,
}

impl FileBackedDictionary {
    fn dictionary_file(path: &Path) -> File {
        let mut dir = std::env::current_dir().expect("No current directory");
        dir.push(path);
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(&dir)
            .expect("No dictionary file")
    }

    pub fn new(path: &Path) -> Self {
        let dictionary: Dictionary =
            serde_yaml::from_reader(FileBackedDictionary::dictionary_file(&path))
                .unwrap_or(Dictionary::new());
        FileBackedDictionary {
            path: path.to_owned(),
            dictionary,
        }
    }

    pub fn as_dictionary(&self) -> &Dictionary {
        &self.dictionary
    }

    pub fn as_dictionary_entries(&self) -> &Vec<DictionaryEntry> {
        &self.as_dictionary().0
    }

    pub fn new_blank_entry(&mut self, regex_str: &str) -> &mut Self {
        let dictionary = &mut self.dictionary;
        dictionary.new_entry(regex_str);
        self
    }

    // Refresh the data from the target file
    pub fn reload(&mut self) -> &Self {
        let dictionary: Dictionary =
            serde_yaml::from_reader(FileBackedDictionary::dictionary_file(&self.path))
                .expect("Unable to parse reloaded file");
        self.dictionary = dictionary;
        self
    }

    pub fn save(&self) -> &Self {
        let dict_str = serde_yaml::to_string(&self.dictionary).expect("Unable to serialize");
        let mut dictionary_file = FileBackedDictionary::dictionary_file(&self.path);
        dictionary_file
            .set_len(0)
            .expect("Unable to write to dictionary file [0]");
        dictionary_file.write_fmt(format_args!("{}", &dict_str)).expect("Unable to write to dictionary file [1]");
        &self
    }
}
