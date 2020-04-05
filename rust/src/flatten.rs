//! I got annoyed with the fact that I couldn't easily write a function
//! from Iterator<Result<String>> to Iterator<Result<String>> (becuase
//! the parsing of the first line of input can't be handled in the loop
//! body) so I wrote this adaptor. The entire point of its existence is
//! to convert the output of drive_robots() from
//! `Result<Iterator<Result<String>>>`) to `Iterator<Result<String>>`.

use anyhow::{Error, Result};

pub(crate) enum FlattenedIteratorOfResult<T>
where
    T: Iterator<Item = Result<String>> + Sized,
{
    Err(Option<Error>),
    Ok(T),
}

impl<T> Iterator for FlattenedIteratorOfResult<T>
where
    T: Iterator<Item = Result<String>> + Sized,
{
    type Item = Result<String>;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            FlattenedIteratorOfResult::Ok(iter) => iter.next(),
            FlattenedIteratorOfResult::Err(err) => Some(Err(err.take()?)),
        }
    }
}

pub(crate) trait ResultOfIteratorOfResult<T>
where
    T: Iterator<Item = Result<String>>,
{
    fn flatten_to_iterator(self) -> FlattenedIteratorOfResult<T>;
}

impl<T> ResultOfIteratorOfResult<T> for Result<T>
where
    T: Iterator<Item = Result<String>>,
{
    fn flatten_to_iterator(self) -> FlattenedIteratorOfResult<T> {
        match self {
            Ok(iter) => FlattenedIteratorOfResult::Ok(iter),
            Err(err) => FlattenedIteratorOfResult::Err(Some(err)),
        }
    }
}
