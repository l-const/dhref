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

/// Define custom Result
type Result<T> = std::result::Result<T, CrateError>;

/// Represents different type of errors that can happen.
#[derive(Debug, Clone)]
enum CrateError {
    /// The error was caued by a failure to read or write bytes on an IO stream.
    IoError,
    // The error was caused during an HTTP GET request.
    HttpReqError,
    /// The error was caused because it was not specified as input a valid http/https URL.
    URLFormatError,
}

impl fmt::Display for CrateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CrateError::IoError => write!(f, "Io Error!"),
            CrateError::HttpReqError => write!(f, "Http Request Error!"),
            CrateError::URLFormatError => write!(f, "URL Format Error!"),
        }
    }
}

impl Error for CrateError {}

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

async fn download_one(location: &str, out_dir: &str) -> Result<()> {
    if let Ok(body) = reqwest::get(location).await.unwrap().bytes().await {
        if let Ok(mut file) = File::create(format!(
            "{}{}",
            out_dir,
            location.split('/').last().expect("Error creating a file!")
        ))
        .await
        {
            return file.write_all(&body).await.map_err(|_| CrateError::IoError);
        }
    }
    Err(CrateError::HttpReqError)
}

#[tokio::main]
async fn download_all(vector_path: Vec<String>, out_dir: &str) -> Vec<Result<()>> {
    let futures: Vec<_> = vector_path
        .iter()
        .map(|path| download_one(path, out_dir))
        .collect();
    future::join_all(futures).await
}

fn parse_page(base_uri: &str, ftype: FileType) -> Result<Option<Vec<String>>> {
    if let Err(err) = check_url(base_uri) {
        return Err(err);
    }

    if let Ok(body) = reqwest::blocking::get(base_uri).unwrap().text() {
        let document = Document::from(&body);
        let elements: Vec<String> = document
            .select("a")
            .iter()
            .map(|elem| elem.attr("href").unwrap_or_default().to_string())
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
            return Ok(Some(elements));
        } else {
            return Ok(None);
        }
    }
    Err(CrateError::HttpReqError)
}

fn check_url(url_str: &str) -> Result<()> {
    if let Some(res) = url_str.split(':').next() {
        return match res {
            "http" => Ok(()),
            "https" => Ok(()),
            _ => Err(CrateError::URLFormatError),
        };
    }
    Err(CrateError::URLFormatError)
}

fn main() {
    let matches = App::new("dhref")
        .version("0.1.1")
        .author("Kostas L. <konlampro94@gmail.com>")
        .about("Download files embed in a page through\n relative and root-relative hyperlinks,\nfrom your terminal.")
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

    match parse_page(cli_opts.page, cli_opts.ftype) {
        Ok(Some(paths)) => {
            download_all(paths, cli_opts.out_dir);
        }
        Err(err) => eprintln!("Error: {}", err),
        Ok(None) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_url() {
        let mut url = "file://hello";
        assert!(check_url(url).is_err());
        url = "https://google.com";
        assert!(check_url(url).is_ok());
        url = "http://google.com";
        assert!(check_url(url).is_ok());
    }

    #[test]
    fn test_parse_page() {}
}
