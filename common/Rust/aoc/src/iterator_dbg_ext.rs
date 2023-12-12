use std::fmt::Debug;

#[derive(Debug, Clone, Copy, Hash)]
pub struct IteratorDbg<It, const DENSE: bool>(It);
impl<It: Iterator, const DENSE: bool> Iterator for IteratorDbg<It, DENSE>
where
    for<'a> &'a It::Item: Debug,
{
    type Item = It::Item;

    // TODO: track caller?
    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.0.next();

        if let Some(ref x) = ret {
            if DENSE {
                eprintln!("{:?}", x);
            } else {
                eprintln!("{:#?}", x);
            }
        }

        ret
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

// TODO: TrustedLen when stable
impl<It: ExactSizeIterator, const D: bool> ExactSizeIterator for IteratorDbg<It, D>
where
    for<'a> &'a It::Item: Debug,
{
    fn len(&self) -> usize {
        self.0.len()
    }

    // fn is_empty(&self) -> bool {
    //     self.is_empty()
    // }
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct IteratorNewl<It>(It);
impl<It: Iterator> Iterator for IteratorNewl<It> {
    type Item = It::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let ret @ Some(_) = self.0.next() {
            eprintln!();
            ret
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

// TODO: TrustedLen when stable
impl<It: ExactSizeIterator> ExactSizeIterator for IteratorNewl<It> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

pub trait IterDbgExt: Iterator
where
    for<'a> &'a Self::Item: Debug,
    Self: Sized,
{
    fn dbg(self) -> IteratorDbg<Self, false> {
        IteratorDbg(self)
    }

    fn ddbg(self) -> IteratorDbg<Self, true> {
        IteratorDbg(self)
    }

    fn newl(self) -> IteratorNewl<Self> {
        IteratorNewl(self)
    }
}

impl<I: Iterator> IterDbgExt for I
where
    for<'a> &'a I::Item: Debug,
    Self: Sized,
{
}
