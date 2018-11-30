#![deny(missing_debug_implementations, missing_docs)]
#![feature(range_contains)]

extern crate clap;
extern crate reqwest;

use std::path::{Component, Path, PathBuf};
use std::io::prelude::*;
use std::fs::File;

use tap::TapOps;
use clap::{Arg, ArgGroup, App};

use crate::AdventOfCodeClient::{AocClient, AocError, Part};

const YEAR: u16 = 2017;

mod AdventOfCodeClient {

    #[derive(PartialEq)]
    pub struct AocClient {
        day: u8,
        year: u16,
        token: String,
    }

    #[derive(Debug)]
    pub enum AocError {
        UnableToSubmit,
        WrongAnswer,
        OtherError(String),
    }

    pub enum Part {
        One,
        Two,
    }

    pub type AocResult = Result<String, AocError>;

    impl AocClient {
        pub fn new(year: u16, day: u8, token: String) -> Result<Self, String> {
            if ! (1 .. 25).contains(&day) {
                return Err(format!("Invalid day: {}", day));
            }

            unimplemented!()
        }

        pub fn get_input(self) -> AocResult {
            get_input(self.year, self.day, self.token)
        }

        pub fn submit_answer(self, answer: String) -> AocResult {
            submit_answer(self.year, self.day, self.token, answer)
        }

        pub(crate) fn get_token(self) -> String {
            self.token
        }
    }

    pub fn get_input(year: u16, day: u8, token: String) -> AocResult {
        unimplemented!()
    }

    pub fn submit_answer(year: u16, day: u8, token: String, answer: String) -> AocResult {
        unimplemented!()
    }
}

#[derive(PartialEq)]
enum InputSource {
    File(String),
    Stdin,
    Web,
}

#[derive(PartialEq)]
enum OutputSink {
    StdOut,
    Web(AocClient)
}

struct Config {
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
    let mut file_path = PathBuf::new();

    // Though we will never create an input file that doesn't have the token
    // specified (we only create this file when we grab the input from the
    // web which we can only do if we have a token), the user may create such a
    // file which is why we account for this possibility.
    let file_name = format!("{}{}.input", day, tok.map_or("", |t| &format!("-{}", t)));

    file_path
        .tap(|f| f.push(Component::ParentDir))
        .tap(|f| f.push(Component::ParentDir))
        .tap(|f| f.push(file_name))
}

impl Config {
    pub fn get_config(day: u8) -> Self {
        Self::get_config_internal(day, None)
    }

    pub fn get_config_with_token(day: u8, tok: &str) -> Self {
        Self::get_config_internal(day, Some(tok))
    }

    fn get_config_internal(day: u8, tok: Option<&str>) -> Self {

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
                    .number_of_values(1))
            .arg(
                Arg::with_name("stdin")
                    .short("")
                    .long("stdin")
                    .number_of_values(0))
            .arg(
                Arg::with_name("creds")
                    .short("c")
                    .long("creds")
                    .value_names(&["FILE"])
                    .number_of_values(1))
            .arg(
                Arg::with_name("token")
                    .short("t")
                    .long("token")
                    .value_names(&["token"])
                    .number_of_values(1))
            .group(ArgGroup::with_name("credentials")
                .args(&["token", "creds"]))
            .group(ArgGroup::with_name("input-source")
                .args(&["input", "stdin"]))
            .get_matches();

        // Check if we've been given a token:
        // Check for args first so they'll 'shadow' a programmatically provided
        // token.
        let token = if let Some(cred_file) = matches.value_of("creds") {
            let mut file = File::open(cred_file).expect(&format!("Unable to open `{}`.", cred_file));
            let mut token = String::new();

            file.read_to_string(&mut token).expect(&format!("Unable to read `{}`.", cred_file));
            
            Some(token)
        } else if let Some(token) = matches.value_of("token") {
            Some(String::from(token))
        } else if let Some(token) = tok {
            Some(String::from(token))
        } else {
            None
        };

        // Now check if the token (if we have one) is valid:
        let output = if let Some(token) = token {
            OutputSink::Web(AocClient::new(YEAR, day, token).unwrap())
        } else {
            // If we don't have a valid token, fall back to printing out to stdout:
            eprint!("Warning: Printing results to stdout");
            OutputSink::StdOut
        };

        // Next, figure out how we're going to take input:
        let input = if let Some(input) = matches.value_of("input") {
            // If we're going to use an input file, check that it exists:
            let path = Path::new(input);

            if path.exists() {
                InputSource::File(input.to_string())
            } else {
                panic!("`{}` doesn't exist! Please specify a valid input file.", input);
            }
        } else if matches.is_present("stdin") {
            InputSource::Stdin
        } else {
            // Failing any explicit input option, we'll try to take input from
            // the website. Before we do that though, we should make sure that
            // there isn't already a copy of the input data we're looking for:
            let f = get_cached_file_path(day, tok);

            // If there is, we'll use it:
            if f.exists() {
                InputSource::File(f.to_str().unwrap().to_string())
            } else {

                // If there isn't we'll just grab the input data. We can only do
                // this if we're configured to submit outputs too (if we're not and
                // still running at this point it means we don't have tokens), so
                // let's check that:
                if let OutputSink::Web(web) = output {
                    InputSource::Web
                } else {
                    // If we have no way to take input, we must error!
                    panic!("No way to take input specified and no token provided!");
                }
            }
        };

        Config { day, input, output }
    }

    fn assert_config(self, inp: InputSource, out: OutputSink) -> bool {
        self.input == inp && self.output == out
    }
}

struct AdventOfCode {
    config: Config,
    input: Option<String>
}

enum Error {
    CannotSubmitAutomatically,
    AutoSubmitError(AocError),
}

type Result = std::result::Result<Option<String>, Error>;

impl AdventOfCode {

    pub fn new(day: u8) -> Self {
        Self {
            config: Config::get_config(day),
            input: None,
        }
    }

    pub fn new_with_token(day: u8, token: &str) -> Self {
        Self {
            config: Config::get_config_with_token(day, token),
            input: None
        }
    }

    pub fn get_input(&mut self) -> &String {
        if let Some(input) = self.input {
            &input
        } else {
            use self::InputSource::*;
            match self.config.input {
                File(f) => {
                    let mut file = std::fs::File::open(f).expect(&format!("Unable to open `{}`.", f));
                    let input = String::new();

                    file.read_to_string(&mut input).expect(&format!("Unable to read `{}`.", f));

                    self.input = Some(input);
                    &input
                },
                Stdin => {
                    let input = String::new();
                    let mut handle = std::io::stdin().lock();

                    handle.read_to_string(&mut input).expect("Unable to read from stdin");

                    self.input = Some(input);
                    &input
                },
                Web => {
                    let aoc = match self.config.output {
                        OutputSink::Web(aoc) => aoc,
                        _ => panic!("It shouldn't be possible to make an AdventOfCode struct\
                            that's configured to grab input from the web but *not* configured\
                            to submit outputs to the web (since web inputs can only be enabled\
                            if we have a valid AocClient already. Please let someone know this\
                            happened")
                    };
                    let input = aoc.get_input().unwrap();

                    // If we successfully got input, let's take this opportunity
                    // to cache the input to be nice to the Advent of Code
                    // servers:
                    let f = get_cached_file_path(self.config.day, Some(&aoc.get_token()));
                    std::fs::write(f, input).expect(&format!("Couldn't write to file `{:?}`.", f));

                    self.input = Some(input);
                    &input
                },
            }
        }
    }

    fn submit<T: Into<String>>(&self, part: Part, answer: T) -> Result {
        use self::OutputSink::*;
        match self.config.output {
            StdOut => {
                println!("{}", answer.into());
                Err(Error::CannotSubmitAutomatically)
            },
            Web(aoc) => {

            },
        }
    }

    fn submit_with_feedback<T: Into<String>>(&self, part: Part, answer: T) -> Result {
        let res = self.submit(part, answer);

        use self::Error::*;
        match res {
            Ok(message) => {

            },
            Err(err) => match err {
                CannotSubmitAutomatically => {
                    eprintln!("Not configured to submit automatically.");
                    eprintln!("Please go to `https://adventofcode.com/{}/day/{}` to submit!", YEAR, self.config.day);
                },
                AutoSubmitError(err) => match err {

                }
            }
        };

        res
    }

    pub fn submit_p1<T: Into<String>>(&self, answer: T) -> Result {
        self.submit_with_feedback(Part::One, answer)
    }

    pub fn submit_p2<T: Into<String>>(&self, answer: T) -> Result {
        self.submit_with_feedback(Part::Two, answer)
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
