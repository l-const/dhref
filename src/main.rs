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

/// Define custorm Result
type Result<T> = std::result::Result<T, CrateError>;

/// Represents different type of errors that can happen.
#[derive(Debug, Clone)]
enum CrateError {
    /// The error was caued by a failure to read or write bytes on an IO
    /// stream.
    IoError,
    // The error was caused during an HTTP GET request.
    HttpReqError,
    /// The error was caused because it was not specified as input a valid http/https url.
    URLFormatError,
    /// The error was used during parsing HTML tokens, searching for some HTML Element.
    ParseHtmlError,
}

impl fmt::Display for CrateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CrateError::IoError => write!(f, "Io Error!"),
            CrateError::HttpReqError => write!(f, "Http Request Error!"),
            CrateError::URLFormatError => write!(f, "URL Format Error!"),
            CrateError::ParseHtmlError => write!(f, "Parse HTML Tree Error!"),
        }
    }
}

impl Error for CrateError {}

#[derive(PartialEq)]
struct ParseURLError<'uri> {
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

impl<'uri> Error for ParseURLError<'uri> {}

impl<'uri> fmt::Debug for ParseURLError<'uri> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ParseURLErrror : {} does not have a http/https scheme!",
            self.uri
        )
    }
}

impl<'uri> fmt::Display for ParseURLError<'uri> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ParseURLError: {} does not have a http/https scheme!",
            self.uri
        )
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
async fn download_all(vector_path: Vec<String>, out_dir: &str) {
    let futures: Vec<_> = vector_path
        .iter()
        .map(|path| download_one(path, out_dir))
        .collect();
    future::join_all(futures).await;
}

fn parse_page(base_uri: &str, ftype: FileType) -> Result<Option<Vec<String>>> {
    check_url(base_uri).expect("Error in url format: ");
    //TODO: IO:ERROR handle
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
        Ok(Some(elements))
    } else {
        Ok(None)
    }
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

    if let Ok(Some(paths)) = parse_page(cli_opts.page, cli_opts.ftype) {
        download_all(paths, cli_opts.out_dir);
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
}
