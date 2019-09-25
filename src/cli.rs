use std::path::PathBuf;

pub struct CliOptions {
    pub dictionary_path: PathBuf,
    pub output_path: PathBuf,
    pub input_path: PathBuf,
}

pub fn get_cli_opts() -> CliOptions {

    let app = clap::App::new("ynab-csv")
        .arg(
            clap::Arg::with_name("dictionary")
                .short("d")
                .default_value("dictionary.yml"),
        )
        .arg(
            clap::Arg::with_name("output")
                .short("o")
                .default_value("out.csv"),
        )
        .arg(clap::Arg::with_name("INPUT"));

    let matches = app.get_matches();
    let dictionary = {
        let file_name = matches.value_of("dictionary");
        let mut buf = PathBuf::from(std::env::current_dir().expect("No current directory"));
        buf.push(file_name.expect("No dictionary file provided"));
        buf
    };
    let output_path = {
        let file_name = matches.value_of("output");
        let mut buf = PathBuf::from(std::env::current_dir().expect("No current directory"));
        buf.push(file_name.expect("No output file file provided"));
        buf
    };
    let input_path = {
        let file_name = matches.value_of("INPUT");
        let mut buf = PathBuf::from(std::env::current_dir().expect("No current directory"));
        buf.push(file_name.expect("No input file file provided"));
        buf
    };

    CliOptions {
        dictionary_path: dictionary,
        output_path,
        input_path,
    }
}
