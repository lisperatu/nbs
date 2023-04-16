//! A simple currency quote scraper for Ledger.
//!
//! This program provides a currency quote scraper that fetches currency quotes
//! from specified websites and formats them for the Ledger command-line accounting tool.
//! It reads configuration from a `.quoteparams` YAML file located in the user's home directory.

#![doc = include_str!("../Readme.md")]



use chrono::prelude::*;
use home::home_dir;
use rayon::prelude::*;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use serde::*;
use serde_yaml;
use std::fmt;

/// A custom error type for handling currency-related errors.
#[derive(Debug)]
struct CurrencyError(String);

impl fmt::Display for CurrencyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Error for CurrencyError {}

impl From<String> for CurrencyError {
    fn from(str: String) -> Self {
        Self(str)
    }
}

/// A struct representing a set of quote parameters.
#[derive(Deserialize, Debug)]
struct QuoteParams {
    url: String,
    select: String,
    from: String,
    to: String,
}

/// Reads the quote parameters from a specified path.
///
/// # Arguments
///
/// * `path` - A reference to a `PathBuf` representing the path to the configuration file.
///
/// # Returns
///
/// * A `Result` containing a `Vec` of `QuoteParams` if successful, or a `CurrencyError` if an error occurs.
fn read_quote_params(path: &PathBuf) -> Result<Vec<QuoteParams>, CurrencyError> {
    let mut file = File::open(path).map_err(|_| format!("Failed to open file {:?}", path))?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|_| format!("Failed to read config file {:?}", path))?;

    let params: Vec<QuoteParams> = serde_yaml::from_str(&content)
        .map_err(|_| format!("Failed to parse config file {:?}", path))?;

    Ok(params)
}

/// Retrieves a currency quote using the provided `QuoteParams`.
///
/// # Arguments
///
/// * `param` - A `QuoteParams` instance containing the required parameters for retrieving the quote.
///
/// # Returns
///
/// * A `Result` containing a `String` with the formatted quote if successful, or a `CurrencyError` if an error occurs.
fn get_currency(param: QuoteParams) -> Result<String, CurrencyError> {
    let res = reqwest::blocking::get(param.url.as_str())
        .map_err(|_| format!("Cannot read url {}", param.url))?;
    let content = res
        .text()
        .map_err(|_| format!("Cannot parse html from url {}", param.url))?;
    let page = scraper::Html::parse_document(&content);

    // I couldn't use simple result? syntax here, I couldn't follow why
    let selector = if let Ok(selector) = scraper::Selector::parse(&param.select) {
        selector
    } else {
        return Err(CurrencyError(format!(
            "Cannot parse html selector {}",
            param.select
        )));
    };

    let currency = page
        .select(&selector)
        .map(|v| v.inner_html())
        .next()
        .unwrap_or("".to_string());
    let now = Utc::now().format("%Y/%m/%d %H:%M:%S").to_string();
    let out = format!("P {} {} {} {}", now, param.from, currency, param.to);
    Ok(out)
}

/// The main function of the currency quote scraper.
///
/// It reads the configuration file, retrieves the currency quotes in parallel,
/// and prints the results in a format suitable for Ledger.
fn main() -> Result<(), Box<dyn Error>> {
    let mut path = home_dir().ok_or("Cannot find home dir")?;
    path.push(".quoteparams");
    let params = read_quote_params(&path)?;

    let results: Vec<String> = params
        .into_par_iter()
        .map(|param| get_currency(param))
        .collect::<Result<Vec<String>, _>>()?;

    for res in results {
        println!("{}", res);
    }

    Ok(())
}
