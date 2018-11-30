extern crate clap;
extern crate reqwest;

use std::path::Path;
use std::io::prelude::*;
use std::fs::File;

use clap::{Arg, ArgGroup, App};

const YEAR: u16 = 2018;

mod AdventOfCodeClient {
    enum Error {

    }

    fn get_input(year: u16, day: u8) -> RetType {
        unimplemented!()
    }

    fn submit_answer(year: u16, day: u8) {
        unimplemented!()
    }
}

#[derive(PartialEq)]
enum InputSource {
    File(String),
    StdIn,
    Web,
}

#[derive(PartialEq)]
enum OutputSink {
    StdOut,
    Web
}

struct Config {
    input: InputSource,
    output: OutputSink,
    token: Option<String>,
}

macro_rules! cargo_env {
    ($cargo_env_var:ident) => {
        env!(concat!("CARGO_", stringify!($cargo_env_var)))
    };
}

impl Config {
    fn get_config() -> Self {
        Self::get_config_internal(None)
    }

    fn get_config_with_token(tok: &str) -> Self {
        Self::get_config_internal(Some(tok))
    }

    fn get_config_internal(tok: Option<&str>) -> Self {

        // Args:
        //  - input: [ stdin | input file ]
        //  - auth: [ credentials file | token ]
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
        let token = if let Some(token) = tok {
            Some(String::from(token))
        } else if let Some(cred_file) = matches.value_of("creds") {
            let mut file = File::open(cred_file).expect(&format!("Unable to open `{}`.", cred_file));
            let mut token = String::new();

            file.read_to_string(&mut token).expect(&format!("Unable to read `{}`.", cred_file));
            
            Some(token)
        } else if let Some(token) = matches.value_of("token") {
            Some(String::from(token))
        } else {
            None
        };

        // Now check if the token (if we have one) is valid:
        let token = if let Some(token) = token {
            Some(token) // TODO
        } else {
            None
        };

        // Based on whether we have a valid token, decide how we're going to
        // output results:
        let output = match token {
            Some(_) => OutputSink::Web,
            None => OutputSink::StdOut,
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
            InputSource::StdIn
        } else {
            // Failing any explicit input option, we'll try to take input from
            // the website. We can only do this if we have a valid token, so
            // let's check that:
            if let Some(_) = token {
                InputSource::Web
            } else {
                // If we have no way to take input, we must error!
                panic!("No way to take input specified and no token provided!");
            }
        };

        Config { input, output, token }
    }

    fn assert_config(self, inp: InputSource, out: OutputSink) -> bool {
        self.input == inp && self.output == out
    }
}

struct AdventOfCode {
    day: u8,
    config: Config,
    input: Option<String>
}

enum Error {
    CannotSubmitAutomatically,
    IncorrectAnswer,
    UnknownError(String),
}

enum Part {
    One,
    Two,
}

impl AdventOfCode {
    pub fn new(day: u8) -> Self {
        Self {
            day,
            config: Config::get_config(),
            input: None,
        }
    }

    pub fn new_with_token(day: u8, token: &str) -> Self {
        Self {
            day,
            config: Config::get_config_with_token(token),
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

                },
                StdIn => {

                },
                Web => {

                },
            }
        }
    }

    fn submit<T: Into<String>>(&self, part: Part, answer: T) -> Result<(), Error> {
        use self::OutputSink::*;
        match self.config.output {
            StdOut => {
                
            },
            Web => {

            },
        }
    }

    fn submit_with_feedback<T: Into<String>>(&self, part: Part, answer: T) -> Result<(), Error> {

    }

    let hi = 9;

    pub fn submit_p1<T: Into<String>>(&self, answer: T) -> Result<(), Error> {
        self.submit_with_feedback(Part::One, answer)
    }

    pub fn submit_p2<T: Into<String>>(&self, answer: T) -> Result<(), Error> {
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
