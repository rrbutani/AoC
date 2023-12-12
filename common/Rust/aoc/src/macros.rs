// our venerable `triangle` macro:
/// `triangle!((a b c d) | foo (J))`:
/// ```
/// foo!(J)
/// foo!(J a)
/// foo!(J a b)
/// foo!(J a b c)
/// foo!(J a b c d)
/// ```
#[macro_export]
macro_rules! triangle {
    (( $($i:tt)* ) | $next:tt $( ($($fwd:tt)+) )?) => {
        $crate::triangle!(@ () # ($($i)*) | $next ($($($fwd)+)?));
    };

    (@ ($($consumed:tt)*) # ($t:tt $($rest:tt)*) | $next:tt ($($f:tt)*)) => {
        $next!($($f)* $($consumed)*);
        $crate::triangle!(@ ($($consumed)* $t) # ($($rest)*) | $next ($($f)*));
    };

    (@ ($($consumed:tt)*) # () | $next:tt ($($f:tt)*)) => {
        $next!($($f)* $($consumed)*);
    };
}

#[macro_export]
macro_rules! count {
    ($a:tt $($t:tt)*) => {
        (1 + $crate::count!($($t)*))
    };
    () => { 0 }
}

#[macro_export]
macro_rules! cdr {
    ($a:tt $($b:tt)+) => { $($b)+ }
}

pub(crate) use crate::{cdr, count, triangle};
