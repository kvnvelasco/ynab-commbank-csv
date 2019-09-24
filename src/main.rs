use crate::dictionary::{Dictionary, FileBackedDictionary};
use csv::{Reader, Writer};
use regex::Regex;
use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::{stdin, Write};
mod cli;
mod dictionary;
#[derive(Serialize)]
struct YnabRecord {
    date: String,
    payee: String,
    memo: String,
    amount: f64,
}

fn main() {
    // open the csv
    let mut csv = {
        let mut dir = std::env::current_dir().expect("No current dir - wtf?");
        dir.push("data.csv");
        Reader::from_path(&dir).expect("No Data File")
    };

    let mut output_csv = {
        let mut dir = std::env::current_dir().expect("No current dir - wtf?");
        dir.push("out.csv");
        Writer::from_path(&dir).expect("No Data File")
    };

    let mut dictionary = {
        let mut dir = std::env::current_dir().expect("No current dir - wtf?");
        dir.push("dictionary.yml");
        FileBackedDictionary::new(dir)
    };

    for line in csv.records() {
        let inner = line.expect("Eh?");
        let date = inner.get(0).expect("No Date");
        let amount: f64 = (inner.get(1).expect("No amount"))
            .parse()
            .expect("Invalid amount set");
        let payee = inner.get(2).expect("No Payee");

        // go over each dictionary item and run regex on it, if we don't find a match, pause wait for input then refresh
        // the regex store

        'main: loop {
            {
                dbg!("Testing {}", &payee);
                for line in dictionary.as_dictionary_entries() {
                    if line.regex.as_regex().as_str() == "" {
                        continue;
                    }
                    dbg!("against {}", line.regex.as_regex());
                    if line.regex.as_regex().is_match(&payee) {
                        // write to the outer scope and break
                        dbg!("Found match with", line.regex.as_regex());
                        output_csv.serialize(YnabRecord {
                            date: date.to_string(),
                            amount,
                            payee: line.name.clone(),
                            memo: String::new(),
                        });
                        output_csv.flush();
                        break 'main;
                    }
                }
            }
            println!("Could not find a match for {}", &payee);
            dictionary.new_blank_entry(&payee);
            dictionary.save();
            let mut str = String::new();
            loop {
                println!("\
                    Your dictionary file has been updated with a new entry at the TOP of the file. \n\
                    Check the created entry for errors \n\
                    Press ENTER when you are done"
                );
                str.clear();
                stdin().read_line(&mut str);
                dbg!(&str);
                if str == "y\n" {
                    break;
                }
            }
            dictionary.reload();
        }
    }
}
