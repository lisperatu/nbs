# Currency Quote Scraper for Ledger

This is a simple currency quote scraper written in Rust, primarily intended to fetch currency quotes and format them for the [Ledger](https://www.ledger-cli.org/) command-line accounting tool. The program fetches currency quotes from specified websites and prints the output in a format suitable for Ledger. The quote parameters, such as the URL, selectors, and currency codes, are stored in a configuration file called `.quoteparams` located in the user's home directory.

## Dependencies

  *  chrono: for handling date and time operations
  *  serde: for deserializing data from the configuration file
  *  serde_yaml: for parsing YAML configuration files
  *  home: for finding the user's home directory
  *  rayon: for parallel processing
  *  reqwest: for making HTTP requests
  *  scraper: for parsing and querying HTML documents

## Usage

1. Build the program using `cargo build --release`. The binary will be located in the `target/release` directory.

2. Move the binary to a location in your PATH, for example:

```bash
mv target/release/your_binary_name ~/.local/bin/
```

3. Create a YAML file named .quoteparams in your home directory with the following structure:

```yaml
- url: <URL of the webpage>
  select: <CSS selector for the currency quote element>
  from: <Base currency code>
  to: <Quote currency code>
```

4. Run the program by calling the binary directly or using an alias. The program will fetch the currency quotes in parallel and print the results in the following format:

```
P <timestamp> <base_currency> <quote> <quote_currency>
```


To directly insert the output into a Ledger file from Emacs, add the following function to your Emacs configuration:

```elisp
(defun insert-nbs-output ()
  "Call the external program 'nbs' and insert its output at the current cursor position in the current buffer."
  (interactive)
  (let ((output (shell-command-to-string "~/.local/bin/nbs")))
    (insert output)))
```

## License
License

This project is open-source and available under the MIT License.
