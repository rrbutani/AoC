use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::{collections::HashMap, ffi::OsString};
use std::{
    env::ArgsOs,
    fs::{self, File},
};

use cargo_metadata::camino::Utf8PathBuf;
use cargo_metadata::MetadataCommand;
use clap::{App, Arg, ArgGroup};
use http::StatusCode;
use indoc::indoc;
use once_cell::sync::OnceCell;
use reqwest::{blocking::Client, header, Error as RequestError, Url};
use scan_fmt::scan_fmt_some;
use select::{
    document::Document,
    predicate::{Class, Name},
};
use tap::tap::Tap;

static LOG_OUTPUT: OnceCell<bool> = OnceCell::new();

macro_rules! dprintln {
    ($($tt:tt)*) => {
        if let Some(true) = LOG_OUTPUT.get() {
            eprintln!($($tt)*);
        }
    };
}

#[derive(Debug, PartialEq, Clone)]
pub struct AocClient {
    day: u8,
    year: u16,
    token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    IncorrectResubmission { correct: String, got: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
pub enum CorrectSubmission {
    New { message: String },
    Resubmitted { answer: String },
}

pub type AocResult<T> = Result<T, AocError>;

fn get_client(token: &str) -> Result<Client, AocError> {
    let tok = String::from("session=").tap_mut(|s| s.push_str(token));

    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::COOKIE,
        header::HeaderValue::from_str(&tok).map_err(|e| AocError::InvalidToken(e.to_string()))?,
    );

    Client::builder()
        .default_headers(headers)
        // .gzip(true)
        .build()
        .map_err(AocError::RequestError)
}

fn base(y: u16, d: u8) -> String {
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

    pub fn get_input(&self) -> AocResult<String> {
        get_input(self.year, self.day, &self.token)
    }

    pub fn submit_answer(&self, part: Part, answer: &str) -> AocResult<CorrectSubmission> {
        submit_answer(self.year, self.day, &self.token, part, answer)
    }

    // Tries to get the answer; the inner option will be empty if the given part
    // has not yet been solved.
    pub fn get_answer(&self, part: Part) -> AocResult<Option<String>> {
        get_answer(self.year, self.day, &self.token, part)
    }

    pub(crate) fn get_token(&self) -> &String {
        &self.token
    }
}

pub fn get_input(year: u16, day: u8, token: &str) -> AocResult<String> {
    let client = get_client(token)?;
    let url =
        Url::parse(&get_input_url(year, day)).map_err(|e| AocError::UnknownError(e.to_string()))?;

    let resp = client
        .get(url)
        .send()
        .map_err(err_mapper)?
        .error_for_status()
        .map_err(err_mapper)?;

    let status = resp.status();
    let body = resp.text().map_err(AocError::RequestError)?;

    match status {
        StatusCode::OK => Ok(body),
        _ => Err(AocError::UnknownError(body)),
    }
}

fn parse_response_message(message: String) -> AocResult<CorrectSubmission> {
    dprintln!("Message: {}", message);

    // Todo: Add "curiously it's the right answer for someone else"
    // "Curiously, it's the right answer for someone else; you're either logged in to the wrong account, unlucky, or cheating. In any case, you need to be using your puzzle input."

    if message.contains("That's not the right answer") {
        let (attempts, timeout) = if message.contains("Because you have guessed incorrectly") {
            let idx = message
                .rfind("Because you have guessed incorrectly")
                .unwrap();
            let incorrect_msg = &message[idx..];

            let end_idx = incorrect_msg.find('.').unwrap();
            let incorrect_msg = &incorrect_msg[..end_idx];

            scan_fmt_some!(
                incorrect_msg,
                "Because you have guessed incorrectly {} times on this \
                puzzle, please wait {} minutes before trying again",
                u8,
                u8
            )
        } else {
            (None, None)
        };

        let timeout = if None == timeout && message.contains("Please wait") {
            let idx = message.rfind("Please wait").unwrap();
            let idx_end = message.rfind("minute").unwrap();

            let timeout_message = &message[idx + 12..idx_end];

            scan_fmt_some!(timeout_message, " {} ", u8)
                .or_else(|| Some(1).filter(|_| timeout_message.starts_with("one")))
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

            let timeout_message = &message[start_idx + 8..end_idx].split_whitespace();

            let m = scan_fmt_some!(timeout_message.clone().next().unwrap(), "{}m", u8);
            let s = scan_fmt_some!(timeout_message.clone().last().unwrap(), "{}s", u8);

            (m, s)
        } else {
            (None, None)
        };

        Err(AocError::Timeout(mins, seconds))
    } else if message.contains("You don't seem to be solving the right level.") {
        Err(AocError::LevelIssue(message))
    } else if message.contains("That's the right answer!") {
        Ok(CorrectSubmission::New { message })
    } else {
        Err(AocError::UnexpectedResponse(message))
    }
}

pub fn submit_answer(
    year: u16,
    day: u8,
    token: &str,
    part: Part,
    answer: &str,
) -> AocResult<CorrectSubmission> {
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

    let resp = client
        .post(url)
        .form(&params)
        .send()
        .map_err(err_mapper)?
        .error_for_status()
        .map_err(err_mapper)?;

    let status = resp.status();
    let body = resp.text().map_err(AocError::RequestError)?;

    match status {
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

            let res = parse_response_message(message);

            // If we got one of these errors, it's possible that we've already
            // solved this level.
            //
            // Note: we don't check for this upfront so that the cases where
            // we're actually submitting an answer for the first time have a
            // (extremely marginally) lower latency.
            if let Err(AocError::LevelIssue(_)) = res {
                // If this is actually the case, we compare the "correct" answer
                // to the answer we just tried to upload:
                if let Ok(Some(correct_answer)) = get_answer(year, day, token, part) {
                    if answer == correct_answer {
                        Ok(CorrectSubmission::Resubmitted {
                            answer: correct_answer,
                        })
                    } else {
                        Err(AocError::IncorrectResubmission {
                            correct: correct_answer,
                            got: answer.to_string(),
                        })
                    }
                } else {
                    res
                }
            } else {
                res
            }
        }
        StatusCode::FOUND => Err(AocError::InvalidAnswer),
        _ => Err(AocError::UnknownError(body)),
    }
}

pub fn get_answer(year: u16, day: u8, token: &str, part: Part) -> AocResult<Option<String>> {
    let client = get_client(token)?;
    let url = Url::parse(&base(year, day)).map_err(|e| AocError::UnknownError(e.to_string()))?;

    let resp = client
        .post(url)
        .send()
        .map_err(err_mapper)?
        .error_for_status()
        .map_err(err_mapper)?;

    let body = resp.text().map_err(AocError::RequestError)?;
    let doc = Document::from(body.as_str());

    let err = || {
        let body = body.clone();
        AocError::UnexpectedResponse(body)
    };

    // Three cases:
    //  - both parts are unsolved:
    //      * contents:
    //          + article: day-desc
    //          + p: "to begin, ..."
    //      * looking for:
    //          + p1: fails on not finding a p with the right contents; None
    //          + p2: fails on not finding a 2nd day-desc; level error
    //  - first part is unsolved
    //      * contents:
    //          + article: day-desc
    //          + p: "your puzzle answer was..."
    //          + p: day-success
    //          + article: day-desc
    //          + form (post)
    //      * looking for:
    //          + p1: succeeds
    //          + p2: fails because the thing after the 2nd day-desc is not a p; None
    //  - both parts are solved
    //      * contents:
    //          + article: day-desc
    //          + p: "your puzzle answer was..."
    //          + article: day-desc
    //          + p: "your puzzle answer was..."
    //      * looking for:
    //          + p1: succeeds
    //          + p2: succeeds

    let answer_node = doc
        .find(Name("main"))
        .take(1)
        .next()
        .ok_or_else(err)?
        .find(Class("day-desc"))
        .take(match part {
            Part::One => 1,
            Part::Two => 2,
        })
        .last()
        // Two hops after day-desc instead of 1; not sure why..
        .ok_or_else(|| AocError::LevelIssue(body.clone()))?
        .next()
        .ok_or_else(|| AocError::LevelIssue(body.clone()))?
        .next()
        // We expect a node after the day-desc article:
        .ok_or_else(err)?;

    // After the day-desc for the part there should be a p containing the string
    // "Your puzzle answer was".
    //
    // If there isn't a p or if the p found doesn't have the string we'll return
    // None (i.e. we'll assume that we have access to the level but it hasn't
    // been solved yet).
    let ans = if answer_node.is(Name("p")) && answer_node.text().contains("Your puzzle answer was")
    {
        answer_node
            .children()
            .find(|c| c.is(Name("code")))
            .map(|c| c.text())
    } else {
        None
    };

    Ok(ans)
}

#[derive(Debug, PartialEq)]
enum InputSource {
    File(String),
    Stdin,
    Web(AocClient),
}

#[derive(Debug, PartialEq)]
enum OutputSink {
    StdOut,
    Web(AocClient),
}

#[derive(Debug)]
pub struct Config {
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

fn get_cached_file_path(year: u16, day: u8, tok: Option<&str>) -> PathBuf {
    // We use the target directory if we're run in a Cargo project; otherwise we
    // just use the current directory.
    // let mut path = PathBuf::from(env!("CARGO_TARGET_DIR"));
    let mut path = if let Ok(target_dir) = std::env::var("AOC_CACHE_DIR") {
        Utf8PathBuf::from(target_dir)
    } else {
        MetadataCommand::new()
            .exec()
            .map(|m| m.target_directory)
            .unwrap_or_else(|_| Utf8PathBuf::from("./"))
    };

    // No matter the base path, we use the `.aoc` dir:
    path.push(".aoc");

    // If we have a token, we use a subdirectory with the token's name.
    //
    // Though we will never create input files when we don't have a token
    // specified (we only create this file when we grab the input from the
    // web which we can only do if we have a token), the user may create such a
    // file which is why we account for this possibility.
    if let Some(tok) = tok {
        path.push(tok);
    }

    // Next, the year:
    path.push(format!("{}", year));

    // Finally, the file name:
    path.tap_mut(|p| p.push(format!("{}.input", day))).into()
}

impl Config {
    pub fn get_config(year: u16, day: u8) -> Self {
        Self::get_config_internal::<OsString, ArgsOs>(year, day, None, None)
    }

    pub fn get_config_with_token(year: u16, day: u8, tok: &str) -> Self {
        Self::get_config_internal::<OsString, ArgsOs>(year, day, Some(tok), None)
    }

    pub fn get_config_with_custom_args<T: Into<OsString> + Clone, I: IntoIterator<Item = T>>(
        year: u16,
        day: u8,
        tok: Option<&str>,
        args: I,
    ) -> Self {
        Self::get_config_internal(year, day, tok, Some(args))
    }

    fn get_config_internal<T, I>(
        year: u16,
        day: u8,
        tok: Option<&str>,
        custom_args: Option<I>,
    ) -> Self
    where
        T: Into<OsString> + Clone,
        I: IntoIterator<Item = T>,
    {
        // Args:
        //  - input: [ stdin | input file | web* ]
        //  - output: [ stdout | web* ]
        //  - auth: [ credentials file* | token ]
        //
        // (defaults marked with *s)
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
        let app = App::new("Advent of Code Helper")
            .version(cargo_env!(PKG_VERSION))
            .author(cargo_env!(PKG_AUTHORS))
            .about(cargo_env!(PKG_DESCRIPTION))
            .arg(
                Arg::with_name("file-input")
                    .short("i")
                    .long("file-input")
                    .help("Read the input from the given file")
                    .long_help(indoc! {"
                Exactly 1 of the three input options will be used.

                If no input option is explicitly specified, we'll try to grab
                the input from adventofcode.com (i.e. the web input option).
                    "})
                    .display_order(0)
                    .value_names(&["FILE"])
                    .number_of_values(1),
            )
            .arg(
                Arg::with_name("stdin-input")
                    .short("")
                    .long("stdin")
                    .help("Read the input from stdin")
                    .display_order(1)
                    .takes_value(false),
            )
            .arg(
                Arg::with_name("web-input")
                    .short("")
                    .long("web-input")
                    .help("Grab the input from adventofcode.com")
                    .display_order(2),
            )
            .arg(
                Arg::with_name("output")
                    .short("o")
                    .long("output")
                    .possible_value("web")
                    .possible_value("stdout")
                    .case_insensitive(true)
                    .default_value("web")
                    .display_order(4),
            )
            .arg(
                Arg::with_name("creds")
                    .short("c")
                    .long("creds")
                    .help("Use the given credentials file")
                    .long_help(indoc! {"
                If the input or output options require us to communicate with
                with adventofcode.com (i.e. --web-input and/or --output=web),
                we'll need credentials in one form or another.

                If no credentials file is given and no token is given we'll try
                to use `common/creds` in the current directory or a few parent
                directories before giving up.
                    "})
                    .value_names(&["FILE"])
                    .number_of_values(1)
                    .display_order(6),
            )
            .arg(
                Arg::with_name("token")
                    .short("t")
                    .long("token")
                    .value_names(&["token"])
                    .number_of_values(1)
                    .display_order(7),
            )
            .arg(
                Arg::with_name("verbose")
                    .short("v")
                    .long("verbose")
                    .takes_value(false),
            )
            .group(ArgGroup::with_name("input-source").args(&[
                "file-input",
                "stdin-input",
                "web-input",
            ]))
            .group(ArgGroup::with_name("credentials").args(&["token", "creds"]));

        let matches = if let Some(args) = custom_args {
            app.get_matches_from(args)
        } else {
            app.get_matches()
        };

        if matches.is_present("verbose") {
            LOG_OUTPUT.set(true).unwrap()
        }

        fn read_token_from_file(cred_file_path: impl AsRef<Path>) -> Option<String> {
            let mut file = File::open(cred_file_path.as_ref()).ok()?;
            let mut token = String::new();

            file.read_to_string(&mut token).unwrap_or_else(|_| {
                panic!("Unable to read `{}`.", cred_file_path.as_ref().display())
            });

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
        let mut token_was_explicitly_specified = true;
        let token = if let Some(cred_file) = matches.value_of("creds") {
            Some(
                read_token_from_file(&cred_file)
                    .unwrap_or_else(|| panic!("Unable to open `{}`.", cred_file)),
            )
        } else if let Some(token) = matches.value_of("token") {
            Some(String::from(token))
        } else if let Some(token) = tok {
            Some(String::from(token))
        } else {
            token_was_explicitly_specified = false;

            // Try to use "common/creds" as a last resort:
            read_token_from_file("common/creds")
                .or_else(|| read_token_from_file("../common/creds"))
                .or_else(|| read_token_from_file("../../common/creds"))
                .or_else(|| read_token_from_file("../../../common/creds"))
        };

        let client = token
            .as_ref()
            .map(|tok| AocClient::new(year, day, tok.clone()).unwrap());

        // Now, the output sink:
        let output = match &*matches.value_of("output").unwrap().to_lowercase() {
            "web" => OutputSink::Web(
                client
                    .clone()
                    .expect("We need a token to submit outputs online."),
            ),
            "stdout" => OutputSink::StdOut,
            _ => unreachable!(),
        };

        // Next, figure out how we're going to take input:
        let input = if let Some(input) = matches.value_of("file-input") {
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
        } else if matches.is_present("stdin-input") {
            InputSource::Stdin
        } else {
            // If the web input option wasn't explicitly specified, mention
            // we're falling back to this default:
            if !matches.is_present("web-input") {
                dprintln!("Note: defaulting to grabbing inputs from the web.");
            }

            // If we're gonna try to use input from the web, we should first
            // make sure that there isn't already a copy of the input data
            // we're looking for:

            // NOTE: we should probably actually just panic here since we really
            // do expect a token if we're told to use input from the web..
            let tok = if let Some(ref token) = token {
                Some(token.as_str())
            } else {
                None
            };

            // let before = Instant::now();
            let f = get_cached_file_path(year, day, tok);
            // let after = Instant::now();
            // eprintln!("cache file path lookup: {:?}", after - before);

            // If there is, we'll use it:
            if f.exists() {
                dprintln!("Note: Using cached input file.");
                InputSource::File(f.to_str().unwrap().to_string())
            } else {
                InputSource::Web(client.expect("We need a token to get inputs from the web."))
            }
        };

        // Finally, emit a warning if we were given a token but didn't use it.
        if token_was_explicitly_specified
            && !matches!(
                (&input, &output),
                (InputSource::Web(_), _) | (_, OutputSink::Web(_))
            )
        {
            dprintln!("Warning: The given token is not being used for anything.")
        }

        Config {
            year,
            day,
            input,
            output,
        }
    }
}

#[derive(Debug)]
pub struct AdventOfCode {
    config: Config,
    input: Option<String>,
    last_event: Option<Instant>,
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
            last_event: None,
        }
    }

    pub fn new_with_token(year: u16, day: u8, token: &str) -> Self {
        Self {
            config: Config::get_config_with_token(year, day, token),
            input: None,
            last_event: None,
        }
    }

    pub fn new_from_config(config: Config) -> Self {
        Self {
            config,
            input: None,
            last_event: None,
        }
    }

    pub fn get_input(&mut self) -> String {
        let ret = self.get_input_inner();
        if self.last_event.is_none() {
            self.last_event = Some(Instant::now());
        }

        ret
    }

    fn get_input_inner(&mut self) -> String {
        if let Some(input) = &self.input {
            input.clone()
        } else {
            use self::InputSource::*;
            match &self.config.input {
                File(f) => {
                    let mut file =
                        fs::File::open(f).unwrap_or_else(|_| panic!("Unable to open `{}`.", f));
                    let mut input = String::new();

                    file.read_to_string(&mut input)
                        .unwrap_or_else(|_| panic!("Unable to read `{}`.", f));

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
                Web(aoc) => {
                    let f = get_cached_file_path(
                        self.config.year,
                        self.config.day,
                        Some(&aoc.get_token()),
                    );
                    let out = if f.exists() {
                        fs::read_to_string(f).unwrap() // error handling?
                    } else {
                        let input = aoc.get_input().unwrap();

                        // If we successfully got input, let's take this opportunity
                        // to cache the input to be nice to the Advent of Code
                        // servers:
                        fs::create_dir_all(f.parent().unwrap()).unwrap();
                        fs::write(&f, &input).unwrap_or_else(|_| {
                            panic!("Couldn't write to file `{}`.", f.display())
                        });

                        input
                    };

                    self.input = Some(out.clone());
                    out
                }
            }
        }
    }

    fn submit<T: ToString>(&mut self, part: Part, answer: T) -> Result<CorrectSubmission, Error> {
        let answer = answer.to_string();
        if let Some(last) = self.last_event {
            eprintln!(
                "{GREY}Part {}: `{answer}` [{:?}]{RESET}",
                part.to_string(),
                last.elapsed(),
                GREY = "\u{001b}[0;37m",
                RESET = "\u{001b}[0m",
            );
        }

        use self::OutputSink::*;
        let ret = match &self.config.output {
            StdOut => {
                eprintln!("{answer}");
                Err(Error::CannotSubmitAutomatically)
            }
            Web(aoc) => aoc
                .submit_answer(part, &answer)
                .map_err(Error::AutoSubmitError),
        };

        self.last_event = Some(Instant::now());
        ret
    }

    fn submit_with_feedback<T: ToString>(
        &mut self,
        part: Part,
        answer: T,
    ) -> Result<CorrectSubmission, Error> {
        let res = self.submit(part, answer);

        use AocError::*;
        use CorrectSubmission::*;
        use Error::*;
        match res {
            Ok(New { ref message }) => {
                eprintln!("⭐ Success for part {}! Got: {}", part.to_string(), message);
            }
            Ok(Resubmitted { ref answer }) => {
                eprintln!("🌠 Still correct! Part {} was already solved but {} \
                    is indeed the correct answer!", part.to_string(), answer)
            }
            Err(ref err) => match err {
                CannotSubmitAutomatically => {
                    eprintln!("🌐 Not configured to submit automatically.");
                    eprintln!(
                        "Please go to '{}' to submit!",
                        base(self.config.year, self.config.day)
                    );
                }
                AutoSubmitError(err) => match err {
                    AuthError(err) => eprintln!(
                        "⛔ Authentication failed; check your token? Got: {}",
                        err.to_string()
                    ),
                    NotFound(err) => eprintln!(
                        "😴 Got a 404; maybe it's too early? We're trying to submit Part \
                        {} of Day {}, {}. Got: {}",
                        part.to_string(),
                        self.config.day,
                        self.config.year,
                        err.to_string()
                    ),
                    InvalidToken(message) => eprintln!("Invalid token. Got: {}", message),
                    WrongAnswer(attempts, timeout, dir) => eprintln!(
                        "❌ Wrong answer! {} {:?} attempts so far and now a {:?} minute \
                        timeout.",
                        dir.to_string(),
                        attempts,
                        timeout
                    ),
                    LevelIssue(message) => {
                        eprintln!("❓ Wrong level? Got: {}", message)
                    }
                    InvalidAnswer => eprintln!(
                        "😕 Something went wrong and the server didn't reply to us. Make \
                        sure the POST request is still right."
                    ),
                    Timeout(mins, seconds) => eprintln!(
                        "🛑 Slow down! Hit a timeout: {:?} minutes and {:?} seconds",
                        mins, seconds
                    ),
                    IncorrectResubmission { correct, got } => eprintln!(
                        "❎ Wrong answer! This part was already solved so we know that the right answer is `{}` (instead of `{}`).",
                        correct,
                        got
                    ),
                    RequestError(err) => eprintln!("☠️ Request Error: {}", err.to_string()),
                    UnknownError(message) => eprintln!("😖 Unknown Error: {}", message),
                    UnexpectedResponse(message) => eprintln!(
                        "🤨 Received an unexpected response back from the server: {}",
                        message
                    ),
                },
            },
        };

        res
    }

    pub fn submit_p1<T: ToString>(&mut self, answer: T) -> Result<CorrectSubmission, Error> {
        self.submit_with_feedback(Part::One, answer)
    }

    pub fn submit_p2<T: ToString>(&mut self, answer: T) -> Result<CorrectSubmission, Error> {
        self.submit_with_feedback(Part::Two, answer)
    }

    pub fn sub<P1: ToString, P2: ToString>(
        year: u16,
        day: u8,
        func: impl FnOnce(&str) -> (P1, P2),
    ) -> Result<(), Error> {
        let mut aoc = Self::new(year, day);
        let inp = aoc.get_input();

        let (d1, d2) = func(inp.as_str());
        aoc.submit_p1(d1)?;
        aoc.submit_p2(d2)?;
        Ok(())
    }
}

#[cfg(test)]
mod response_message_tests {
    use super::{parse_response_message, AocError, ErrDirection};

    macro_rules! message_test {
        ($nom:ident: $msg:literal, $expected:pat) => {
            #[test]
            fn $nom() {
                match parse_response_message($msg.to_string()) {
                    $expected => { /* ok! */ }
                    err => panic!("Expected `{:#?}`, got `{:#?}`.", stringify!($expected), err),
                }
            }
        };
    }

    message_test! {
        first_wrong_answer:
        "
        That's not the right answer.  If you're stuck, make sure you're using \
        the full input data; there are also some general tips on the \
        <a href=\"/2015/about\">about page</a>, or you can ask for hints on \
        the <a href=\"https://www.reddit.com/r/adventofcode/\" \
        target=\"_blank\"> subreddit</a>.  Please wait one minute before \
        trying again. (You guessed <span style=\"white-space:nowrap;\"> \
        <code>89</code>.)</span> <a href=\"/2015/day/7\">[Return to Day 7]</a>
        ",
        Err(AocError::WrongAnswer(None, Some(1), ErrDirection::Unknown))
    }

    // Same as first. Third (omitted) is also the same as the first.
    message_test! {
        second_wrong_answer:
        "
        That's not the right answer.  If you're stuck, make sure you're using \
        the full input data; there are also some general tips on the \
        <a href=\"/2015/about\">about page</a>, or you can ask for hints on \
        the <a href=\"https://www.reddit.com/r/adventofcode/\" \
        target=\"_blank\"> subreddit</a>.  Please wait one minute before \
        trying again. (You guessed <span style=\"white-space:nowrap;\"> \
        <code>89</code>.)</span> <a href=\"/2015/day/7\">[Return to Day 7]</a>
        ",
        Err(AocError::WrongAnswer(None, Some(1), ErrDirection::Unknown))
    }

    message_test! {
        fourth_wrong_answer:
        "
        That's not the right answer.  If you're stuck, make sure you're using \
        the full input data; there are also some general tips on the \
        <a href=\"/2015/about\">about page</a>, or you can ask for hints on
        the <a href=\"https://www.reddit.com/r/adventofcode/\" \
        target=\"_blank\"> subreddit</a>.  Because you have guessed \
        incorrectly 4 times on this puzzle, please wait 5 minutes before \
        trying again. (You guessed <span style=\"white-space:nowrap;\"><code>d \
        </code>.)</span> <a href=\"/2015/day/6\">[Return to Day 6]
        ",
        Err(AocError::WrongAnswer(Some(4), Some(5), ErrDirection::Unknown))
    }

    message_test! {
        seventh_wrong_answer:
        "
        That's not the right answer.  If you're stuck, make sure you're using \
        the full input data; there are also some general tips on the \
        <a href=\"/2015/about\">about page</a>, or you can ask for hints on
        the <a href=\"https://www.reddit.com/r/adventofcode/\" \
        target=\"_blank\"> subreddit</a>.  Because you have guessed \
        incorrectly 7 times on this puzzle, please wait 10 minutes before \
        trying again. (You guessed <span style=\"white-space:nowrap;\"><code>d \
        </code>.)</span> <a href=\"/2015/day/6\">[Return to Day 6]
        ",
        Err(AocError::WrongAnswer(Some(7), Some(10), ErrDirection::Unknown))
    }

    message_test! {
        timeout_seconds:
        "
        You gave an answer too recently; you have to wait after submitting an \
        answer before trying again.  You have 44s left to wait. \
        <a href=\"/2015/day/7\">[Return to Day 7]</a>
        ",
        Err(AocError::Timeout(None, Some(44)))
    }

    message_test! {
        timeout_1_second:
        "
        You gave an answer too recently; you have to wait after submitting an \
        answer before trying again.  You have 1s left to wait. \
        <a href=\"/2015/day/7\">[Return to Day 7]</a>
        ",
        Err(AocError::Timeout(None, Some(1)))
    }

    message_test! {
        timeout_minutes:
        "
        You gave an answer too recently; you have to wait after submitting an \
        answer before trying again.  You have 5m 0s left to wait. \
        <a href=\"/2015/day/7\">[Return to Day 7]</a>
        ",
        Err(AocError::Timeout(Some(5), Some(0)))
    }

    message_test! {
        timeout_minutes_and_seconds:
        "
        You gave an answer too recently; you have to wait after submitting an \
        answer before trying again.  You have 4m 59s left to wait. \
        <a href=\"/2015/day/7\">[Return to Day 7]</a>
        ",
        Err(AocError::Timeout(Some(4), Some(59)))
    }
}

// TODO: example input mode?
//   - `--ex`
//   - attempts to scrape example code block/prints result on stdout instead of
//     submitting/erroring

// TODO: verbose option exposed to programs?
//   - perhaps as a lazy_static? idk
