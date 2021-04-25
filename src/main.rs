extern crate clap;
extern crate futures;
extern crate nipper;
extern crate reqwest;
extern crate tokio;

use clap::{App, Arg};
use futures::future;
use nipper::Document;
use std::convert::*;
use std::error::Error;
use std::fmt;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;


/// Represents different type of errors that can happen.
#[derive(Debug, Clone)]
enum MyError {
    /// The error was caued by a failure to read or write bytes on an IO
    /// stream.
    Io,
    // The error was caused during an HTTP GET request.
    HttpReq,
    /// The error was caused because it was not specified as input a valid http/https url.
    URLFormat
}


struct ParseError<'uri> {
    uri: &'uri str,
}

#[derive(Debug, Default)]
struct CliOpts<'input> {
    page: &'input str,
    out_dir: &'input str,
    ftype: FileType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FileType {
    PDF,
    DOC,
    DOCX,
    XLSX,
    CSV,
    PPT,
    PPTX,
    ALL,
}

impl From<&str> for FileType {
    fn from(input: &str) -> Self {
        match input {
            "pdf" => FileType::PDF,
            "doc" => FileType::DOC,
            "docx" => FileType::DOCX,
            "xlsx" => FileType::XLSX,
            "ppt" => FileType::PPT,
            "pptx" => FileType::PPTX,
            "csv" => FileType::CSV,
            _ => FileType::ALL,
        }
    }
}

impl Into<&str> for FileType {
    fn into(self) -> &'static str {
        match self {
            FileType::PDF => ".pdf",
            FileType::DOC => ".doc",
            FileType::DOCX => ".docx",
            FileType::XLSX => ".xlsx",
            FileType::PPT => ".ppt",
            FileType::PPTX => ".pptx",
            FileType::CSV => ".csv",
            FileType::ALL => "",
        }
    }
}

impl Default for FileType {
    fn default() -> Self {
        FileType::ALL
    }
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
    if let Ok(mut file) = File::create(location.split('/').last().unwrap()).await {
        file.write_all(&body).await.unwrap();
    }
}

#[tokio::main]
async fn download_all(vector_path: Vec<String>) {
    let futures: Vec<_> = vector_path.iter().map(|path| download_one(path)).collect();
    future::join_all(futures).await;
}

fn parse_page(base_uri: &str, ftype: FileType) -> Option<Vec<String>> {
    check_url(base_uri).unwrap();
    let body = reqwest::blocking::get(base_uri).unwrap().text().unwrap();
    let document = Document::from(&body);
    let elements: Vec<String> = document
        .select("a")
        .iter()
        .map(|elem| elem.attr("href").unwrap().to_string())
        .filter(|elem_str| {
            if ftype == FileType::ALL {
                elem_str.contains(".")
            } else {
                let ftype_str: &str = ftype.into();
                elem_str.ends_with(ftype_str)
            }
        })
        .map(|elem| format!("{}{}", &base_uri, elem))
        .collect();
    if elements.len() > 0 {
        Some(elements)
    } else {
        None
    }
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
                .help("Http page url to be scraped. (Required)"),
        )
        .arg(
            Arg::with_name("out_dir")
                .takes_value(true)
                .short("o")
                .help("Relative path for the folder to place the output. (Optional)"),
        )
        .arg(
            Arg::with_name("ftype").short("f").takes_value(true).help(
                "File suffix for the type of files to be searched( e.g pdf,doc,csv). (Optional)",
            ),
        )
        .get_matches();

    let cli_opts = CliOpts {
        page: &matches.value_of("uri").unwrap().to_ascii_lowercase(),
        out_dir: matches.value_of("out_dir").unwrap_or("./"),
        ftype: matches.value_of("ftype").unwrap_or("").into(),
    };

    if let Some(paths) = parse_page(cli_opts.page, cli_opts.ftype) {
        download_all(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_page(){

    }
}
