use aoc::{client::Config, AdventOfCode};
use pyo3::prelude::*;

use std::env::args_os;

/// A Python wrapper for [`AdventOfCode`].
#[pyclass]
#[pyo3(text_signature = "(year, day, /)")]
struct Aoc {
    inner: AdventOfCode,
}

#[pymethods]
impl Aoc {
    /// Constructs a new [`Aoc`] instance.
    ///
    /// Let's the `aoc` crate search for the credentials.
    #[new]
    fn new(year: u16, day: u8) -> Self {
        // We skip the first arg since that's the Python interpreter!
        let config = Config::get_config_with_custom_args(year, day, None, args_os().skip(1));

        Self {
            inner: AdventOfCode::new_from_config(config),
        }
    }

    /// Grabs the input for the problem.
    #[pyo3(text_signature = "($self)")]
    fn get_input(&mut self) -> String {
        self.inner.get_input()
    }

    /// Submits part 1. Prints the results on stderr.
    ///
    /// Returns `True` or `False` to indicate success or failure.
    #[pyo3(text_signature = "($self, answer)")]
    fn submit_p1(&mut self, answer: &PyAny) -> bool {
        self.inner.submit_p1(answer).is_ok()
    }

    /// Submits part 2. Prints the results on stderr.
    ///
    /// Returns `True` or `False` to indicate success or failure.
    #[pyo3(text_signature = "($self, answer)")]
    fn submit_p2(&mut self, answer: &PyAny) -> bool {
        self.inner.submit_p2(answer).is_ok()
    }
}

#[pymodule]
fn aoc_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Aoc>()
}
