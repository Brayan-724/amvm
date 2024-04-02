use std::char;
use std::fmt::Display;
use std::ops::RangeFrom;
use std::str::{CharIndices, Chars};

pub use nom::{
    bytes::complete::*, character::complete::*, combinator::*, error::*, AsBytes, AsChar, Compare,
    Err, FindSubstring, IResult, InputIter, InputTake, InputTakeAtPosition, Needed, Offset, Slice,
};

use nom::{FindToken, InputLength};

pub(crate) const CMD_VERBOSE: bool = true;

pub type ParserResult<'a, O, I = Parser<'a>, Err = VerboseError<I>> = IResult<I, O, Err>;

#[derive(Clone, Copy, Debug)]
pub struct Parser<'a> {
    input: &'a str,
    pub value: &'a str,

    line: usize,
    line_byte_start: usize,
}

impl Parser<'_> {
    pub fn new<'a>(input: &'a str) -> Parser<'a> {
        Parser {
            input,
            value: input,

            line: 0,
            line_byte_start: 0,
        }
    }

    pub fn over_nom_err_with_context(
        ctx: &'static str,
    ) -> impl Fn(Err<VerboseError<Self>>) -> Err<VerboseError<Self>> {
        move |err| {
            err.map(|err| VerboseError {
                errors: err
                    .errors
                    .iter()
                    .map(|err| match err.1 {
                        VerboseErrorKind::Nom(_) => (err.0.clone(), VerboseErrorKind::Context(ctx)),
                        _ => err.clone(),
                    })
                    .collect::<Vec<(Parser, VerboseErrorKind)>>(),
            })
        }
    }

    pub fn map_nom_err_with_context(
        ctx: impl Display,
    ) -> impl Fn(Err<VerboseError<Self>>) -> Err<Vec<String>> {
        move |err| {
            err.map(|e| {
                e.errors
                    .iter()
                    .map(|e| match e.1 {
                        VerboseErrorKind::Nom(_) => format!(
                            "{ctx} at byte {idx} {pos:?}",
                            idx = e.0.pointer_position(),
                            pos = e.0.cursor_position(),
                        ),
                        nom::error::VerboseErrorKind::Char(c) => {
                            format!(
                                "Expected '{c}' at byte {idx} {pos:?}",
                                idx = e.0.pointer_position(),
                                pos = e.0.cursor_position(),
                            )
                        }
                        _ => todo!("{:#?}", e.1),
                    })
                    .collect::<Vec<String>>()
            })
        }
    }

    pub fn map_nom_err(err: Err<VerboseError<Self>>) -> Err<Vec<String>> {
        err.map(|e| {
            e.errors
                .iter()
                .map(|e| match e.1 {
                    nom::error::VerboseErrorKind::Char(c) => {
                        format!(
                            "Expected '{c}' at byte {idx} {pos:?}",
                            idx = e.0.pointer_position(),
                            pos = e.0.cursor_position(),
                        )
                    }
                    VerboseErrorKind::Context(ctx) => format!(
                        "{ctx} at byte {idx} {pos:?}",
                        idx = e.0.pointer_position(),
                        pos = e.0.cursor_position(),
                    ),

                    VerboseErrorKind::Nom(n) => format!(
                        "{n:?} at byte {idx} {pos:?}",
                        idx = e.0.pointer_position(),
                        pos = e.0.cursor_position(),
                    ),
                })
                .collect::<Vec<String>>()
        })
    }
}

impl<'a> Parser<'a> {
    pub fn peek(&self, count: usize) -> Option<char> {
        self.value.chars().nth(count)
    }

    pub fn error(&self, kind: VerboseErrorKind, is_failure: bool) -> Err<VerboseError<Self>> {
        let err = VerboseError {
            errors: vec![(self.clone(), kind)],
        };

        if is_failure {
            Err::Failure(err)
        } else {
            Err::Error(err)
        }
    }

    pub fn nom_err_with_context(
        &self,
        ctx: &'static str,
    ) -> impl Fn(Err<VerboseError<Self>>) -> Err<VerboseError<Self>> {
        let this = self.clone();
        move |err| {
            err.map(|err| {
                let mut errors = err.errors;
                errors.push((this, VerboseErrorKind::Context(ctx)));
                VerboseError { errors }
            })
        }
    }

    #[inline(always)]
    fn update_value(&self, value: &'a str) -> Self {
        Parser {
            value,

            input: self.input,
            line: self.line,
            line_byte_start: self.line_byte_start,
        }
    }

    pub fn new_line(&self) -> Self {
        Self {
            line: self.line + 1,

            input: self.input,
            value: self.value,
            line_byte_start: self.line_byte_start,
        }
    }

    // #[inline(always)]
    // pub fn line(&self) -> usize {
    //     self.line
    // }

    #[inline(always)]
    pub fn column(&self) -> usize {
        self.pointer_position() - self.line_byte_start
    }

    /// Cursor in line and column.
    ///
    /// See also:
    /// [Parser::pointer_position]
    #[inline(always)]
    pub fn cursor_position(&self) -> (usize, usize) {
        (self.line, self.column())
    }

    /// Cursor in bytes.
    /// [`Offset::offset(self.input, self.value)`](nom::Offset::offset)
    ///
    /// See also:
    /// [Parser::cursor_position]
    /// [nom::Offset::offset]
    #[inline(always)]
    pub fn pointer_position(&self) -> usize {
        let fst = self.input.as_ptr();
        let snd = self.value.as_ptr();

        snd as usize - fst as usize
    }
}

impl<'a> Compare<&str> for Parser<'a> {
    #[inline]
    fn compare(&self, t: &str) -> nom::CompareResult {
        self.value.compare(t)
    }

    #[inline]
    fn compare_no_case(&self, t: &str) -> nom::CompareResult {
        self.value.compare_no_case(t)
    }
}

impl<'a> InputIter for Parser<'a> {
    type Item = char;
    type Iter = CharIndices<'a>;
    type IterElem = Chars<'a>;

    #[inline]
    fn iter_elements(&self) -> Self::IterElem {
        self.value.chars()
    }

    #[inline]
    fn iter_indices(&self) -> Self::Iter {
        self.value.char_indices()
    }

    #[inline]
    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.value.chars().position(predicate)
    }

    #[inline]
    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        let mut cnt = 0;
        for (index, _) in self.value.char_indices() {
            if cnt == count {
                return Ok(index);
            }
            cnt += 1;
        }
        if cnt == count {
            return Ok(self.value.len());
        }
        Err(nom::Needed::Unknown)
    }
}

impl<'a> InputLength for Parser<'a> {
    fn input_len(&self) -> usize {
        self.value.len()
    }
}

impl<'a> InputTake for Parser<'a> {
    #[inline]
    fn take(&self, count: usize) -> Self {
        self.update_value(&self.value[0..count])
    }

    #[inline]
    fn take_split(&self, count: usize) -> (Self, Self) {
        let (a, b) = self.value.split_at(count);

        (self.update_value(a), self.update_value(b))
    }
}

impl<'a> InputTakeAtPosition for Parser<'a> {
    type Item = char;

    fn split_at_position<P, E: ParseError<Self>>(&self, predicate: P) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.value.find(predicate) {
            // find() returns a byte index that is already in the slice at a char boundary
            Some(i) => unsafe {
                Ok((
                    self.update_value(self.value.get_unchecked(i..)),
                    self.update_value(self.value.get_unchecked(..i)),
                ))
            },
            None => Err(Err::Incomplete(Needed::new(1))),
        }
    }

    fn split_at_position1<P, E: ParseError<Self>>(
        &self,
        predicate: P,
        e: ErrorKind,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.value.find(predicate) {
            Some(0) => Err(Err::Error(E::from_error_kind(self.clone(), e))),
            // find() returns a byte index that is already in the slice at a char boundary
            Some(i) => unsafe {
                Ok((
                    self.update_value(self.value.get_unchecked(i..)),
                    self.update_value(self.value.get_unchecked(..i)),
                ))
            },
            None => Err(Err::Incomplete(Needed::new(1))),
        }
    }

    fn split_at_position_complete<P, E: ParseError<Self>>(
        &self,
        predicate: P,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.value.find(predicate) {
            // find() returns a byte index that is already in the slice at a char boundary
            Some(i) => unsafe {
                Ok((
                    self.update_value(self.value.get_unchecked(i..)),
                    self.update_value(self.value.get_unchecked(..i)),
                ))
            },
            // the end of slice is a char boundary
            None => unsafe {
                Ok((
                    self.update_value(self.value.get_unchecked(self.value.len()..)),
                    self.update_value(self.value.get_unchecked(..self.value.len())),
                ))
            },
        }
    }

    fn split_at_position1_complete<P, E: ParseError<Self>>(
        &self,
        predicate: P,
        e: ErrorKind,
    ) -> IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.value.find(predicate) {
            Some(0) => Err(Err::Error(E::from_error_kind(self.clone(), e))),
            // find() returns a byte index that is already in the slice at a char boundary
            Some(i) => unsafe {
                Ok((
                    self.update_value(self.value.get_unchecked(i..)),
                    self.update_value(self.value.get_unchecked(..i)),
                ))
            },
            None => {
                if self.value.is_empty() {
                    Err(Err::Error(E::from_error_kind(self.clone(), e)))
                } else {
                    // the end of slice is a char boundary
                    unsafe {
                        Ok((
                            self.update_value(self.value.get_unchecked(self.value.len()..)),
                            self.update_value(self.value.get_unchecked(..self.value.len())),
                        ))
                    }
                }
            }
        }
    }
}

impl<'a> FindToken<char> for Parser<'a> {
    fn find_token(&self, token: char) -> bool {
        FindToken::find_token(&self.value, token)
    }
}

impl<'a> FindSubstring<&str> for Parser<'a> {
    #[inline]
    fn find_substring(&self, substr: &str) -> Option<usize> {
        self.value.find_substring(substr)
    }
}

impl<'a> Offset for Parser<'a> {
    #[inline]
    fn offset(&self, second: &Self) -> usize {
        Offset::offset(self.value, &second.value)
    }
}

impl<'a> Slice<RangeFrom<usize>> for Parser<'a> {
    #[inline]
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        self.update_value(self.value.slice(range))
    }
}

pub fn take_space<I, Err>(parser: I) -> IResult<I, char, Err>
where
    I: InputIter<Item = char> + Slice<RangeFrom<usize>>,
    Err: ParseError<I>,
{
    one_of(" \t\r\n")(parser)
}

pub fn take_until_space<I, Err>(parser: I) -> IResult<I, I, Err>
where
    I: InputTakeAtPosition<Item = char>,
    Err: ParseError<I>,
{
    is_not(" \t\r\n")(parser)
}
