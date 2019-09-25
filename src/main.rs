use std::io::stdin;

use csv::{Reader, Writer};
use serde::Serialize;

use crate::cli::get_cli_opts;
use crate::dictionary::{FileBackedDictionary};

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
    let opts = get_cli_opts();
    let mut csv = Reader::from_path(&opts.input_path).expect("No Data File");
    let mut output_csv = Writer::from_path(&opts.output_path).expect("No Data File");
    let mut dictionary = FileBackedDictionary::new(&opts.dictionary_path);

    for line in csv.records() {
        let inner = line.expect("An unexpected error occured");
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
                        output_csv
                            .serialize(YnabRecord {
                                date: date.to_string(),
                                amount,
                                payee: line.name.clone(),
                                memo: String::new(),
                            })
                            .expect("Unable to write to csv output");
                        output_csv.flush().expect("Unable to write to csv output");
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
                    Check the created entry for errors \n\n\
                    Are you done checking for errors (y)?"
                );
                str.clear();
                stdin()
                    .read_line(&mut str)
                    .expect("Unable to read from stidin");
                dbg!(&str);
                if str == "y\n" {
                    break;
                }
            }
            dictionary.reload();
        }
    }
}
