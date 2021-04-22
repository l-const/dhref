# dhref 

![Crates.io](https://img.shields.io/crates/v/dhref)

**Download files embed to a page through relative/root-relative URLs.**

### Description
The program scrapes the url page specified for the hyperlink with   
relative/root-relative URLs and asynchronously downloads files filtered
by the filetype specified in the input.

### FileTypes
* PDF
* XLSX
* DOCX
* DOC
* CSV
* PPT
* PPTX
* ALL OF ABOVE

### Examples

```bash
    dhref <url>  ftype pdf/PDF
    dhref <url>  ftype csv/CSV
    dhref <url>  -o ./ -f pdf
    dhref <url>  -o ./out -f csv
```

### Install

```bash
    cargo install dhref
```

### Help

```bash
    dhref --help

    dhref 0.1.0
    Kostas L. <konlampro94@gmail.com>
    Download files embed in a page through
        relative and root-relative hyperlinks.

    USAGE:
        dhref [OPTIONS] <uri>

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    OPTIONS:
        -f <ftype>          File suffix for the type of files to be searched( e.g pdf,doc,csv). (Optional)
        -o <out_dir>        Relative path for the folder to place the output. (Optional)

    ARGS:
        <uri>    Http page url to be scraped. (Required)
```
