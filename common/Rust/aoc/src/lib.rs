#[doc(hidden)]
pub use aoc_macros::*;

#[macro_export]
macro_rules! sub {
    ($($tt:tt)*) => {{
        let Some((year, day)) = (|| -> Option<_> {
            let p = ::std::path::Path::new(file!());
            let mut p = p.iter();
            let year = p.next()?.to_str()?.parse().ok()?;
            let day = p.next()?.to_str()?.parse().ok()?;

            Some((year, day))
        })() else {
            panic!("unable to infer year/day from file name ({})", file!());
        };

        ::aoc::AdventOfCode::sub(year, day, $($tt)*).unwrap()
    }};
}

pub struct Hang;

impl Display for Hang {
    #[allow(clippy::empty_loop, clippy::print_in_format_impl)]
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        eprintln!("stalled");
        loop {}
    }
}

pub use AdventOfCode as A;

#[macro_use]
mod macros;
pub mod client;
pub mod friends;
pub mod iterator_collect_ext;
pub mod iterator_dbg_ext;
pub mod iterator_freq_ext;
pub mod iterator_map_ext;
pub mod line_try_map;
pub mod object_store;
pub mod tuple_idx;

pub use client::AdventOfCode;
pub use friends::Itertools;
pub use friends::*;
pub use iterator_collect_ext::IterCollectExt as _;
pub use iterator_dbg_ext::IterDbgExt as _;
pub use iterator_freq_ext::IterFreqExt as _;
pub use iterator_map_ext::IterMapExt as _;
pub use line_try_map::LineTryMap as _;
pub use tuple_idx::num_structs::*;
pub use tuple_idx::TupleGetExt as _;
