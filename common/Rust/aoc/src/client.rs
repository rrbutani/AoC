use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Component, Path, PathBuf};

use clap::{App, Arg, ArgGroup};
use http::StatusCode;
use reqwest::{header, Client, Error as RequestError, RedirectPolicy, Url};
use scan_fmt::scan_fmt;
use select::{
    document::Document,
    predicate::Name,
};
use tap::TapOps;

#[derive(Debug, PartialEq)]
pub struct AocClient {
    day: u8,
    year: u16,
    token: String,
}

#[derive(Debug)]
pub enum ErrDirection {
    TooHigh,
    TooLow,
    Unknown,
}

impl ErrDirection {
    pub fn to_string(&self) -> &'static str {
        match self {
            ErrDirection::TooHigh => "The answer was given to be too high.",
            ErrDirection::TooLow => "The answer was given to be too low.",
            ErrDirection::Unknown => "No indication about the answer was given.",
        }
    }
}

#[derive(Debug)]
pub enum AocError {
    AuthError(RequestError),
    NotFound(RequestError),
    InvalidToken(String),
    WrongAnswer(Option<u8>, Option<u8>, ErrDirection),
    InvalidAnswer,
    LevelIssue(String),
    Timeout(Option<u8>, Option<u8>),
    RequestError(RequestError),
    UnknownError(String),
    UnexpectedResponse(String),
}

pub enum Part {
    One,
    Two,
}

impl Part {
    pub fn to_string(&self) -> &'static str {
        match self {
            Part::One => "one",
            Part::Two => "two",
        }
    }
}

pub type AocResult = Result<String, AocError>;

fn get_client(token: &String) -> Result<Client, AocError> {
    let tok = String::from("session=").tap(|s| s.push_str(token));

    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::COOKIE,
        header::HeaderValue::from_str(&tok).map_err(|e| AocError::InvalidToken(e.to_string()))?,
    );

    reqwest::Client::builder()
        .default_headers(headers)
        // .gzip(true)
        .redirect(RedirectPolicy::none())
        .build()
        .map_err(|e| AocError::RequestError(e))
}

pub(crate) fn base(y: u16, d: u8) -> String {
    format!("https://adventofcode.com/{}/day/{}", y, d)
}

fn get_input_url(year: u16, day: u8) -> String {
    format!("{}/input", base(year, day))
}

fn get_output_url(year: u16, day: u8) -> String {
    format!("{}/answer", base(year, day))
}

fn err_mapper(err: RequestError) -> AocError {
    if let Some(status) = err.status() {
        match status {
            StatusCode::BAD_REQUEST => AocError::AuthError(err),
            StatusCode::NOT_FOUND => AocError::NotFound(err),
            StatusCode::FOUND => AocError::InvalidAnswer,
            _ => AocError::RequestError(err),
        }
    } else {
        AocError::RequestError(err)
    }
}

impl AocClient {
    pub fn new(year: u16, day: u8, token: String) -> Result<Self, String> {
        if !(1..=25).contains(&day) {
            return Err(format!("Invalid day: {}", day));
        }

        Ok(AocClient { year, day, token })
    }

    pub fn get_input(&self) -> AocResult {
        get_input(self.year, self.day, &self.token)
    }

    pub fn submit_answer(&self, part: &Part, answer: &String) -> AocResult {
        submit_answer(self.year, self.day, &self.token, part, answer)
    }

    pub(crate) fn get_token(&self) -> &String {
        &self.token
    }
}

pub fn get_input(year: u16, day: u8, token: &String) -> AocResult {
    let client = get_client(token)?;
    let url =
        Url::parse(&get_input_url(year, day)).map_err(|e| AocError::UnknownError(e.to_string()))?;

    let mut resp = client
        .get(url)
        .send()
        .map_err(err_mapper)?
        .error_for_status()
        .map_err(err_mapper)?;

    let body = resp.text().map_err(|e| AocError::RequestError(e))?;

    match resp.status() {
        StatusCode::OK => Ok(body),
        _ => Err(AocError::UnknownError(body)),
    }
}

pub fn submit_answer(
    year: u16,
    day: u8,
    token: &String,
    part: &Part,
    answer: &String,
) -> AocResult {
    let client = get_client(token)?;
    let url = Url::parse(&get_output_url(year, day))
        .map_err(|e| AocError::UnknownError(e.to_string()))?;

    let mut params = HashMap::new();
    params.insert(
        "level",
        match part {
            Part::One => "1",
            Part::Two => "2",
        },
    );
    params.insert("answer", answer);

    let mut resp = client
        .post(url)
        .form(&params)
        .send()
        .map_err(err_mapper)?
        .error_for_status()
        .map_err(err_mapper)?;

    let body = resp.text().map_err(|e| AocError::RequestError(e))?;

    match resp.status() {
        StatusCode::OK => {
            let doc = Document::from(body.as_str());

            let err = || {
                let body = body.clone();
                AocError::UnexpectedResponse(body)
            };

            let message = doc
                .find(Name("main"))
                .take(1)
                .next()
                .ok_or_else(err)?
                .find(Name("p"))
                .take(1)
                .next()
                .ok_or_else(err)?
                .text();

            eprintln!("Message: {}", message);

            // Todo: Add "curiously it's the right answer for someone else"
            // "Curiously, it's the right answer for someone else; you're either logged in to the wrong account, unlucky, or cheating. In any case, you need to be using your puzzle input."

            if message.contains("That's not the right answer") {
                let (attempts, timeout) = if message
                    .contains("Because you have guessed incorrectly")
                {
                    let idx = message
                        .rfind("Because you have guessed incorrectly")
                        .unwrap();
                    let incorrect_msg = &message[idx..];

                    let end_idx = incorrect_msg.find('.').unwrap();
                    let incorrect_msg = &incorrect_msg[..end_idx];

                    scan_fmt!(incorrect_msg,
                        "Because you have guessed incorrectly {} times on this \
                        puzzle, please wait {} minutes before trying again",
                        u8, u8)
                } else {
                    (None, None)
                };

                let timeout = if None == timeout && message.contains("Please wait") {
                    let idx = message.rfind("Please wait").unwrap();
                    let idx_end = message.rfind("minute").unwrap();

                    let timeout_message = &message[idx + 12..idx_end];

                    scan_fmt!(timeout_message, " {} ", u8)
                } else {
                    timeout
                };

                let dir = if message.contains("too high") {
                    ErrDirection::TooHigh
                } else if message.contains("too low") {
                    ErrDirection::TooLow
                } else {
                    ErrDirection::Unknown
                };

                Err(AocError::WrongAnswer(attempts, timeout, dir))
            } else if message.contains("You gave an answer too recently") {
                let (mins, seconds) = if message.contains("left to wait") {
                    let end_idx = message.find("left to wait.").unwrap();
                    let start_idx = message.rfind("You have ").unwrap();

                    let timeout_message = &message[start_idx + 8..end_idx];

                    scan_fmt!(timeout_message, " {}m {}s ", u8, u8)
                } else {
                    (None, None)
                };

                Err(AocError::Timeout(mins, seconds))
            } else if message.contains("You don't seem to be solving the right level.") {
                Err(AocError::LevelIssue(message))
            } else if message.contains("That's the right answer!") {
                Ok(message)
            } else {
                Err(AocError::UnexpectedResponse(message))
            }
        }
        StatusCode::FOUND => Err(AocError::InvalidAnswer),
        _ => Err(AocError::UnknownError(body)),
    }
}

#[derive(Debug, PartialEq)]
enum InputSource {
    File(String),
    Stdin,
    Web,
}

#[derive(Debug, PartialEq)]
enum OutputSink {
    StdOut,
    Web(AocClient),
}

#[derive(Debug)]
struct Config {
    year: u16,
    day: u8,
    input: InputSource,
    output: OutputSink,
}

macro_rules! cargo_env {
    ($cargo_env_var:ident) => {
        env!(concat!("CARGO_", stringify!($cargo_env_var)))
    };
}

fn get_cached_file_path(day: u8, tok: Option<&str>) -> PathBuf {
    let file_path = PathBuf::new();

    // Though we will never create an input file that doesn't have the token
    // specified (we only create this file when we grab the input from the
    // web which we can only do if we have a token), the user may create such a
    // file which is why we account for this possibility.
    let file_name = format!(
        "{}{}.input",
        day,
        tok.map_or(String::from(""), |t| format!("-{}", t))
    );

    file_path
        .tap(|f| f.push(Component::ParentDir))
        .tap(|f| f.push(file_name))
}

impl Config {
    pub fn get_config(year: u16, day: u8) -> Self {
        Self::get_config_internal(year, day, None)
    }

    pub fn get_config_with_token(year: u16, day: u8, tok: &str) -> Self {
        Self::get_config_internal(year, day, Some(tok))
    }

    fn get_config_internal(year: u16, day: u8, tok: Option<&str>) -> Self {
        // Args:
        //  - input: [ stdin | input file ]
        //  - auth: [ credentials file | token ]
        //
        // The general strategy here is use defaults implicitly, but if an
        // option is specified, use it or fail.
        //
        // For example, for the input we'll try to use a file/stdin if you
        // say to. If you don't we'll try to grab the input from the web if
        // a valid token is provided. However, if you specify a file that
        // doesn't exist, we'll blow up, even if you _did_ specify a valid
        // token.
        //
        // In other words, if you specify something explicitly, that's what
        // we'll use.
        //
        // The same goes for the output: if you pass in a token as an cli arg,
        // we'll use that or fail. But if you didn't we'll check for a
        // credentials file/a token passed in programmatically.
        //
        // We rely on clap not allowing you to pass in multiple input/output
        // methods so that there isn't ambiguity. The one exception to this is
        // the token argument in this function and `get_config_with_token`: a
        // token passed in at runtime will take precedence over a token passed
        // into the function programmatically (since the latter is potentially
        // fixed). This allows for a nice override mechanism that could be
        // useful if the token changes (it'll save you a recompile).
        //
        // One other bit of ambiguity that I should document: if inputs are
        // grabbed from the web, they're written to a local file (with the
        // token appended to the file name). The next time the program is run,
        // it looks for the file matching the current day+token and if it
        // exists, it'll opt to use that instead of grabbing the input file
        // anew. If an input file or stdin are specified, however, those will
        // take precedence. So, to recap, the order is:
        //  - stdin/specified file
        //  - cached file
        //  - grabbed anew, if possible
        let matches = App::new("Advent of Code Helper")
            .version(cargo_env!(PKG_VERSION))
            .author(cargo_env!(PKG_AUTHORS))
            .about(cargo_env!(PKG_DESCRIPTION))
            .arg(
                Arg::with_name("input")
                    .short("i")
                    .long("input")
                    .value_names(&["FILE"])
                    .number_of_values(1),
            )
            .arg(Arg::with_name("stdin").short("").long("stdin"))
            .arg(
                Arg::with_name("creds")
                    .short("c")
                    .long("creds")
                    .value_names(&["FILE"])
                    .number_of_values(1),
            )
            .arg(
                Arg::with_name("token")
                    .short("t")
                    .long("token")
                    .value_names(&["token"])
                    .number_of_values(1),
            )
            .group(ArgGroup::with_name("credentials").args(&["token", "creds"]))
            .group(ArgGroup::with_name("input-source").args(&["input", "stdin"]))
            .get_matches();

        fn read_token_from_file(cred_file_path: impl AsRef<Path>) -> Option<String> {
            let mut file = File::open(cred_file_path).ok()?;
            let mut token = String::new();

            file.read_to_string(&mut token).expect(&format!(
                "Unable to read `{}`.",
                cred_file_path.as_ref().display()
            ));

            Some(
                token
                    .lines()
                    .next()
                    .expect("A token file that's *not* empty.")
                    .to_owned(),
            )
        }

        // Check if we've been given a token:
        // Check for args first so they'll 'shadow' a programmatically provided
        // token.
        let token = if let Some(cred_file) = matches.value_of("creds") {
            Some(
                read_token_from_file(cred_file)
                    .expect(&format!("Unable to open `{}`.", cred_file)),
            )
        } else if let Some(token) = matches.value_of("token") {
            Some(String::from(token))
        } else if let Some(token) = tok {
            Some(String::from(token))
        } else {
            // Try to use "../common/creds" as a last resort:
            read_token_from_file("../common/creds")
        };

        // Now check if the token (if we have one) is valid:
        let output = if let Some(ref token) = token {
            OutputSink::Web(AocClient::new(year, day, token.to_string()).unwrap())
        } else {
            // If we don't have a valid token, fall back to printing out to stdout:
            eprintln!("Warning: Printing results to stdout.");
            OutputSink::StdOut
        };

        // Next, figure out how we're going to take input:
        let input = if let Some(input) = matches.value_of("input") {
            // If we're going to use an input file, check that it exists:
            let path = Path::new(input);

            if path.exists() {
                InputSource::File(input.to_string())
            } else {
                panic!(
                    "`{}` doesn't exist! Please specify a valid input file.",
                    input
                );
            }
        } else if matches.is_present("stdin") {
            InputSource::Stdin
        } else {
            // Failing any explicit input option, we'll try to take input from
            // the website. Before we do that though, we should make sure that
            // there isn't already a copy of the input data we're looking for:

            let tok = if let Some(ref token) = token {
                Some(token.as_str())
            } else {
                None
            };
            let f = get_cached_file_path(day, tok);

            // If there is, we'll use it:
            if f.exists() {
                eprintln!("Note: Using cached input file.");
                InputSource::File(f.to_str().unwrap().to_string())
            } else {
                // If there isn't we'll just grab the input data. We can only do
                // this if we're configured to submit outputs too (if we're not and
                // still running at this point it means we don't have tokens), so
                // let's check that:
                if let OutputSink::Web(_) = output {
                    InputSource::Web
                } else {
                    // If we have no way to take input, we must error!
                    panic!("No way to take input specified and no token provided!");
                }
            }
        };

        Config {
            year,
            day,
            input,
            output,
        }
    }

    pub(crate) fn assert_config(self, inp: InputSource, out: OutputSink) -> bool {
        self.input == inp && self.output == out
    }
}

#[derive(Debug)]
pub struct AdventOfCode {
    config: Config,
    input: Option<String>,
}

#[derive(Debug)]
pub enum Error {
    CannotSubmitAutomatically,
    AutoSubmitError(AocError),
}

impl AdventOfCode {
    pub fn new(year: u16, day: u8) -> Self {
        Self {
            config: Config::get_config(year, day),
            input: None,
        }
    }

    pub fn new_with_token(year: u16, day: u8, token: &str) -> Self {
        Self {
            config: Config::get_config_with_token(year, day, token),
            input: None,
        }
    }

    pub fn new_with_year_and_token(day: u8, year: u16, token: &str) -> Self {
        Self {
            config: Config::get_config_with_token(year, day, token),
            input: None,
        }
    }

    pub fn get_input(&mut self) -> String {
        if let Some(input) = &self.input {
            input.clone()
        } else {
            use self::InputSource::*;
            match &self.config.input {
                File(f) => {
                    let mut file =
                        std::fs::File::open(f).expect(&format!("Unable to open `{}`.", f));
                    let mut input = String::new();

                    file.read_to_string(&mut input)
                        .expect(&format!("Unable to read `{}`.", f));

                    let out = input.clone();
                    self.input = Some(input);
                    out
                }
                Stdin => {
                    let mut input = String::new();
                    let stdin = std::io::stdin();
                    let mut handle = stdin.lock();

                    handle
                        .read_to_string(&mut input)
                        .expect("Unable to read from stdin");

                    let out = input.clone();
                    self.input = Some(input);
                    out
                }
                Web => {
                    let aoc = match &self.config.output {
                        OutputSink::Web(aoc) => aoc,
                        _ => panic!(
                            "It shouldn't be possible to make an AdventOfCode struct \
                            that's configured to grab input from the web but *not* \
                            configured to submit outputs to the web (since web inputs \
                            can only be enabled if we have a valid AocClient already. \
                            Please let someone know this happened."
                        ),
                    };
                    let input = aoc.get_input().unwrap();

                    // If we successfully got input, let's take this opportunity
                    // to cache the input to be nice to the Advent of Code
                    // servers:
                    let f = get_cached_file_path(self.config.day, Some(&aoc.get_token()));
                    let fname = f.clone();
                    let fname = fname.to_str().unwrap_or("<some file>");
                    std::fs::write(f, &input)
                        .expect(&format!("Couldn't write to file `{}`.", fname));

                    let out = input.clone();
                    self.input = Some(input);
                    out
                }
            }
        }
    }

    fn submit<T: ToString>(&mut self, part: &Part, answer: T) -> AocResult {
        use self::OutputSink::*;
        match &self.config.output {
            StdOut => {
                println!("{}", answer.to_string());
                Err(Error::CannotSubmitAutomatically)
            }
            Web(aoc) => aoc
                .submit_answer(part, &answer.to_string())
                .map_err(|e| Error::AutoSubmitError(e)),
        }
    }

    fn submit_with_feedback<T: ToString>(&mut self, part: &Part, answer: T) -> AocResult {
        let res = self.submit(part, answer);

        use self::AocError::*;
        use self::Error::*;
        match res {
            Ok(ref message) => {
                println!("Success for part {}! Got: {}", part.to_string(), message);
            }
            Err(ref err) => match err {
                CannotSubmitAutomatically => {
                    eprintln!("Not configured to submit automatically.");
                    eprintln!(
                        "Please go to '{}' to submit!",
                        base(self.config.year, self.config.day)
                    );
                }
                AutoSubmitError(err) => match err {
                    AuthError(err) => eprintln!(
                        "Authentication failed; check your token? Got: {}",
                        err.to_string()
                    ),
                    NotFound(err) => eprintln!(
                        "Got a 404; maybe it's too early? We're trying to submit Part \
                        {} of Day {}, {}. Got: {}",
                        part.to_string(),
                        self.config.day,
                        self.config.year,
                        err.to_string()
                    ),
                    InvalidToken(message) => eprintln!("Invalid token. Got: {}", message),
                    WrongAnswer(attempts, timeout, dir) => eprintln!(
                        "Wrong answer! {} {:?} attempts so far and now a {:?} minute \
                        timeout.",
                        dir.to_string(),
                        attempts,
                        timeout
                    ),
                    LevelIssue(message) => {
                        eprintln!("Wrong level? Or already finished? Got: {}", message)
                    }
                    InvalidAnswer => eprintln!(
                        "Something went wrong and the server didn't reply to us. Make \
                        sure the POST request is still right."
                    ),
                    Timeout(mins, seconds) => eprintln!(
                        "Slow down! Hit a timeout: {:?} minutes and {:?} seconds",
                        mins, seconds
                    ),
                    RequestError(err) => eprintln!("Request Error: {}", err.to_string()),
                    UnknownError(message) => eprintln!("Unknown Error: {}", message),
                    UnexpectedResponse(message) => eprintln!(
                        "Received an unexpected response back from the server: {}",
                        message
                    ),
                },
            },
        };

        res
    }

    pub fn submit_p1<T: ToString>(&mut self, answer: T) -> AocResult {
        self.submit_with_feedback(&Part::One, answer)
    }

    pub fn submit_p2<T: ToString>(&mut self, answer: T) -> AocResult {
        self.submit_with_feedback(&Part::Two, answer)
    }
}
