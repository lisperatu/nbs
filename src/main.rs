//! A simple currency quote scraper for Ledger.
//!
//! This program provides a currency quote scraper that fetches currency quotes
//! from specified websites and formats them for the Ledger command-line accounting tool.
//! It reads configuration from a `.quoteparams` YAML file located in the user's home directory.

#![doc = include_str!("../Readme.md")]

use chrono::prelude::*;
use home::home_dir;
use rayon::prelude::*;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use thiserror::Error;

use serde::*;
use serde_yaml;

/// A custom error type for handling currency-related errors.
#[derive(Error, Debug)]
pub enum CurrencyError {
    #[error("File error {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML deserialization error {0}")]
    SerdeYaml(#[from] serde_yaml::Error),

    #[error("HTTP request error {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Scrapper selector error {0}")]
    ScraperSelector(String),

    #[error("Scrapper error {0}")]
    Scraper(String),

    #[error("Home directory not found")]
    HomeDirNotFound,
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
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let params: Vec<QuoteParams> = serde_yaml::from_str(&content)?;

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
    let res = reqwest::blocking::get(param.url.as_str())?;
    let content = res.text()?;
    let page = scraper::Html::parse_document(&content);
    let selector = scraper::Selector::parse(&param.select)
        .map_err(|e| CurrencyError::ScraperSelector(format!("Selector parse error: {}", e)))?;

    let currency = page
        .select(&selector)
        .map(|v| v.inner_html())
        .next()
        .ok_or(CurrencyError::Scraper(format!("Failed to extract currency data {}", param.select)))?;
    let now = Utc::now().format("%Y/%m/%d %H:%M:%S").to_string();
    let out = format!("P {} {} {} {}", now, param.from, currency, param.to);
    Ok(out)
}

/// The main function of the currency quote scraper.
///
/// It reads the configuration file, retrieves the currency quotes in parallel,
/// and prints the results in a format suitable for Ledger.
fn main() -> Result<(), CurrencyError> {
    let mut path = home_dir().ok_or(CurrencyError::HomeDirNotFound)?;
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
