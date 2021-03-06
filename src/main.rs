use clap::{App, Arg};
use futures::future;
use nipper::Document;
use std::convert::*;
use std::error::Error;
use std::fmt;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub use self::CrateError::*;
pub use self::FileType::*;

/// Define custom Result
type Result<T> = std::result::Result<T, CrateError>;

/// Represents different type of errors that can happen.
#[derive(Debug, Clone, PartialEq)]
pub enum CrateError {
    /// The error was caused by a failure to read or write bytes on an IO stream.
    IoError(String),
    /// The error was caused during an HTTP GET request.
    HttpReqError(String),
    /// The error was caused because it was not specified as input a valid http/https URL.
    URLFormatError,
}

impl fmt::Display for CrateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IoError(tokio_error) => write!(f, "Io Error: {}!", tokio_error),
            HttpReqError(http_error) => {
                write!(f, "Http Request Error: {}!", http_error)
            }
            URLFormatError => write!(f, "URL Format Error!"),
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
pub enum FileType {
    Pdf,
    Doc,
    Docx,
    Xlsx,
    Csv,
    Ppt,
    Pptx,
    All,
}

impl From<&str> for FileType {
    fn from(input: &str) -> Self {
        match input {
            "pdf" => Pdf,
            "doc" => Doc,
            "docx" => Docx,
            "xlsx" => Xlsx,
            "ppt" => Ppt,
            "pptx" => Pptx,
            "csv" => Csv,
            _ => All,
        }
    }
}

impl From<tokio::io::Error> for CrateError {
    fn from(input: tokio::io::Error) -> Self {
        IoError(input.to_string())
    }
}

impl From<reqwest::Error> for CrateError {
    fn from(input: reqwest::Error) -> Self {
        HttpReqError(input.to_string())
    }
}

impl Into<&str> for FileType {
    fn into(self) -> &'static str {
        match self {
            Pdf => ".pdf",
            Doc => ".doc",
            Docx => ".docx",
            Xlsx => ".xlsx",
            Ppt => ".ppt",
            Pptx => ".pptx",
            Csv => ".csv",
            All => "",
        }
    }
}

impl Default for FileType {
    fn default() -> Self {
        All
    }
}

async fn download_one(location: &str, out_dir: &str) -> Result<()> {
    let resp = reqwest::get(location).await?;
    let body = resp.bytes().await?;
    let mut file = File::create(format!(
        "{}{}",
        out_dir,
        location.split('/').last().unwrap()
    ))
    .await?;
    file.write_all(&body).await?;
    Ok(())
}

#[tokio::main]
async fn download_all(vector_path: Vec<String>, out_dir: &str) {
    let futures: Vec<_> = vector_path
        .iter()
        .map(|path| download_one(path, out_dir))
        .collect();
    let vec_results = future::join_all(futures).await;
    vec_results
        .into_iter()
        .filter(|res| res.is_err())
        .map(|err| err.unwrap_err())
        .for_each(|e| eprintln!("Error {:?}", e));
}

fn parse_page(base_uri: &str, ftype: FileType) -> Result<Option<Vec<String>>> {
    check_url(base_uri)?;
    let resp = reqwest::blocking::get(base_uri)?;
    let body = resp.text()?;
    let document = Document::from(&body);
    let elements: Vec<String> = document
        .select("a")
        .iter()
        .map(|elem| elem.attr("href").unwrap_or_default().to_string())
        .filter(|elem_str| {
            if ftype == FileType::All {
                elem_str.contains(".")
            } else {
                let ftype_str: &str = ftype.into();
                elem_str.ends_with(ftype_str)
            }
        })
        .map(|elem| format!("{}{}", &base_uri, elem))
        .collect();
    if elements.len() > 0 {
        Ok(Some(elements))
    } else {
        Ok(None)
    }
}

fn check_url(url_str: &str) -> Result<()> {
    if let Some(res) = url_str.split(':').next() {
        match res {
            "http" => Ok(()),
            "https" => Ok(()),
            _ => Err(URLFormatError),
        }
    } else {
        Err(URLFormatError)
    }
}

fn main() {
    let matches = App::new("dhref")
        .version("0.2.0")
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
    const PAGE: &str = "https://15445.courses.cs.cmu.edu/fall2019/schedule.html";

    #[test]
    fn test_check_url() {
        let mut url = "file://hello";
        assert_eq!(check_url(url), Err(URLFormatError));
        url = "https://google.com";
        assert_eq!(check_url(url), Ok(()));
        url = "http://google.com";
        assert_eq!(check_url(url), Ok(()));
    }

    #[test]
    fn test_parse_page() {
        let mut result = parse_page(PAGE, Pdf);
        assert!(result.is_ok());
        result = parse_page(PAGE, All);
        assert!(result.is_ok());
        let page = "http://";
        result = parse_page(page, All);
        assert!(result.is_err());
    }
}
