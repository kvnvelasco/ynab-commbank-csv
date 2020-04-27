use std::io::{stdin, stdout, Read, Write};

use csv::{Reader, Writer};
use serde::Serialize;

use crate::cli::get_cli_opts;
use crate::dictionary::FileBackedDictionary;

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

    let mut input_buffer = String::new();
    loop {
        println!(
            "Paste any pending transactions from the commbank interface. Press ctrl-D when done"
        );
        println!(
            "-----------------------------PASTE BELOW (ctrl-d to finish)----------------------------"
        );

        {
            let stdin_init = stdin();
            let mut lock = stdin_init.lock();
            let size = lock
                .read_to_string(&mut input_buffer)
                .expect("unable to read bytes");
            println!(
                "---------------------------------READ {} BYTES------------------------------------",
                size
            );
        }
        let mut continue_string = String::new();
        print!("Continue (y) / Re-do (n): ");
        stdout().flush();
        stdin().read_line(&mut continue_string);
        if continue_string == "y\n" {
            break;
        }
    }

    if input_buffer != "" {
        let lines: Vec<&str> = input_buffer.trim().split("\n").collect();
        if lines.len() > 4 {
            for chunk in lines.chunks(4) {
                let date = {
                    let date_str = chunk[0].trim();
                    dbg!(&date_str);
                    // format spec: https://docs.rs/chrono/0.4.9/chrono/format/strftime/index.html
                    let date_object = chrono::NaiveDate::parse_from_str(date_str, "%d %b %Y")
                        .expect("Unable to parse date in pasted data");
                    date_object.format("%d/%m/%Y").to_string()
                };
                let payee = chunk[2].replace("PENDING -", "");
                let amount = chunk[3].trim();
                let line_item = test_line(
                    &date,
                    payee.trim(),
                    amount
                        .replace("$", "")
                        .parse()
                        .expect("Invalid amount in pasted data"),
                    &mut dictionary,
                );
                output_csv
                    .serialize(line_item)
                    .expect("Unable to serialize record");
            }
        }
    }

    for line in csv.records() {
        let inner = line.expect("An unexpected error occured");
        let date = inner.get(0).expect("No Date");
        let amount: f64 = (inner.get(1).expect("No amount"))
            .parse()
            .expect("Invalid amount set");
        let payee = inner.get(2).expect("No Payee");
        let record = test_line(date, payee, amount, &mut dictionary);
        output_csv
            .serialize(record)
            .expect("Unable to serialize record");
        // go over each dictionary item and run regex on it, if we don't find a match, pause wait for input then refresh
        // the regex store
    }
    output_csv.flush().expect("Unable to write to CSV");
}

fn test_line(
    date: &str,
    payee: &str,
    amount: f64,
    dictionary: &mut FileBackedDictionary,
) -> YnabRecord {
    'main: loop {
        {
            dbg!("Testing {}", &payee);
            for line in dictionary.as_dictionary_entries() {
                if line.regex.as_regex().as_str() == "" {
                    continue;
                }
                if line.regex.as_regex().is_match(&payee) {
                    // write to the outer scope and break
                    dbg!("Found match with", line.regex.as_regex());
                    return YnabRecord {
                        date: date.to_string(),
                        amount,
                        payee: line.name.clone(),
                        memo: String::new(),
                    };
                }
            }
        }
        println!("Could not find a match for {}", &payee);
        dictionary.new_blank_entry(&payee);
        dictionary.save();
        let mut str = String::new();
        loop {
            println!(
                "\
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
