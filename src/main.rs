extern crate futures;
extern crate nipper;
extern crate reqwest;
extern crate tokio;
extern crate clap;


use futures::future;
use nipper::Document;
use std::env::Args;
use std::error::Error;
use std::fmt;
use tokio::fs::File;
use std::convert::*;
use tokio::io::AsyncWriteExt;
use clap::{Arg, App};

struct ParseError<'uri> {
    uri: &'uri str,
}


#[derive(Debug, Clone, Copy)]
enum FileType {
    PDF,
    DOC,
    DOCX,
    XLSX,
    CSV,
    PPT,
    PPTX,
    ALL
}


impl From<&str> for FileType {
    fn from(input: &str) -> Self {
        match input {
            "pdf"  => FileType::PDF,
            "doc"  => FileType::DOC,
            "docx" => FileType::DOCX,
            "xlsx" => FileType::XLSX,
            "ppt"  => FileType::PPT,
            "pptx" => FileType::PPTX,
            "csv" => FileType::CSV,
            _ => FileType::ALL
        }
    }
}


impl Into<&str> for FileType {
    fn into(self) -> &'static str {
        match self {
            FileType::PDF => ".pdf",
            FileType::DOC => ".doc",
            FileType::DOCX  => ".docx",
            FileType::XLSX => ".xlsx",
            FileType::PPT => ".ppt",
            FileType::PPTX => ".pptx",
            FileType::CSV => ".csv",
            FileType::ALL => ""  
        }
    }
}

impl Default for FileType {
    fn default() -> Self {
        FileType::ALL
    }
}

#[derive(Debug, Default)]
struct CliOpts<'input> {
    page: &'input str,
    out_dir: &'input str,
    ftype: FileType
}

impl<'uri> Error for ParseError<'uri> {}

impl<'uri> fmt::Debug for ParseError<'uri> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ParserErrror : {} does not have a http/https scheme!",
            self.uri
        )
    }
}

impl<'uri> fmt::Display for ParseError<'uri> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ParserError: {} does not have a http/https scheme!",
            self.uri
        )
    }
}

async fn download_one(location: &str) {
    let body = reqwest::get(location).await.unwrap().bytes().await.unwrap();
    let mut file = File::create(location.split('/').last().unwrap())
        .await
        .unwrap();
    file.write_all(&body).await.unwrap();
}

fn parse_page(args: &mut Args, ftype: FileType) -> Vec<String> {
    let base_uri = args.nth(1).unwrap();
    check_url(&base_uri).unwrap();
    let body = reqwest::blocking::get(base_uri.clone())
        .unwrap()
        .text()
        .unwrap();
    let document = Document::from(&body);
    let elements: Vec<String> = document
        .select("a")
        .iter()
        .map(|elem| elem.attr("href").unwrap().to_string())
        .filter(|elem_str| {
            let ftype_str : &str = ftype.into();
            elem_str.ends_with(ftype_str)
        })
        .map(|elem| format!("{}{}", &base_uri, elem))
        .collect();
    println!("Elements[1]: {}, len: {}", elements[1], elements.len());
    elements
}

fn check_url(url_str: &str) -> Result<(), ParseError> {
    match url_str.split(':').next().unwrap() {
        "http" => Ok(()),
        "https" => Ok(()),
        _ => Err(ParseError { uri: &url_str }),
    }
}

fn main() {
    let matches = App::new("dhref")
                        .version("0.1.0")
                        .author("Kostas L. <konlampro94@gmail.com>")
                        .about("Download files embed in a page through\n relative and root-relative hyperlinks.")
                        .arg(
                            Arg::with_name("uri")
                            .required(true)
                            .takes_value(true)
                            .help("Http page url to be scraped. (Required)")
                        )
                        .arg(
                            Arg::with_name("out_dir")
                                .takes_value(true)
                                .help("Relative path for the folder to place the output. (Optional)"),
                        )
                        .arg(
                            Arg::with_name("ftype")
                                .takes_value(true)
                                .help("File prefix for the output files for each table. (Optional)"),
                        )
                        .get_matches();
    let cli_opts = CliOpts {
        page:  matches.value_of("uri").unwrap(),
        out_dir : matches.value_of("out_dir").unwrap_or("./"),
        ftype : matches.value_of("ftype").unwrap_or("").into()
    };

}



//TODO: CHECK FOR ABSOLUTE VS RELATIVE VS ROOT-RELATIVE URLS


// fn main() {
//     let mut args = std::env::args();
//     if args.len() <= 1 {
//         println!("Not enough arguments! Please specify a http/https uri!");
//         return;
//     } else if args.len() > 3 {
//         println!("Too many arguments!\n");
//         return;
//     }
//     let paths = parse_page(&mut args, ".pdf");
//     download_all(paths);
//     // let body = download("https://15445.courses.cs.cmu.edu/fall2019/slides/");
// }

#[tokio::main]
async fn download_all(vector_path: Vec<String>) {
    let futures: Vec<_> = vector_path.iter().map(|path| download_one(path)).collect();
    future::join_all(futures).await;
}
