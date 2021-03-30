// extern crate futures;
extern crate futures;
extern crate nipper;
extern crate reqwest;
extern crate tokio;

use futures::future;
use nipper::Document;
use std::env::Args;
use std::error::Error;
use std::fmt;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

struct ParseError<'uri> {
    uri: &'uri str,
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

fn parse_page(args: &mut Args, pattern: &str) -> Vec<String> {
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
        .filter(|elem_str| elem_str.contains(pattern))
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
    let mut args = std::env::args();
    if args.len() <= 1 {
        println!("Not enough arguments! Please specify a http/https uri!");
        return;
    } else if args.len() > 3 {
        println!("Too many arguments!\n");
        return;
    }
    let paths = parse_page(&mut args, ".pdf");
    download_all(paths);
    // let body = download("https://15445.courses.cs.cmu.edu/fall2019/slides/");
}

#[tokio::main]
async fn download_all(vector_path: Vec<String>) {
    let futures: Vec<_> = vector_path.iter().map(|path| download_one(path)).collect();
    future::join_all(futures).await;
}
