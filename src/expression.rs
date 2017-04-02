use std::fmt;
use std::marker::PhantomData;

use serde::ser::{Serialize, Serializer};
use serde::de::{Deserialize, Deserializer, Visitor, Error};

use combine::char::{char, letter, spaces};
use combine::{many1, parser, try, Parser};
use combine::combinator::FnParser;
use combine::primitives::{State, Stream, ParseResult};

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Id(String),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>)
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Expr::*;

        match *self {
            Id(ref s) => write!(f, "{}", s),
            Add(ref l, ref r) => write!(f, "{} + {}", l, r),
            Sub(ref l, ref r) => write!(f, "{} - {}", l, r)
        }
    }
}

impl Serialize for Expr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl Deserialize for Expr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize(ExprVisitor)
    }
}

struct ExprVisitor;

impl Visitor for ExprVisitor {
    type Value = Expr;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("any math expression referencing accounts")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where E: Error
    {
        match Expression::expr().parse(State::new(v)) {
            Ok((expr, _)) => Ok(expr),
            Err(e) => Err(Error::custom(e)),
        }
    }
}

#[derive(Default)]
pub struct Expression<I>(PhantomData<fn(I) -> I>);

type ExprParser<O, I> = FnParser<I, fn(I) -> ParseResult<O, I>>;

fn fn_parser<O, I>(f: fn(I) -> ParseResult<O, I>) -> ExprParser<O, I>
  where I: Stream<Item = char>
{
  parser(f)
}

impl<I> Expression<I>
  where I: Stream<Item = char>
{

    fn id() -> ExprParser<Expr, I> {
        fn_parser(Expression::<I>::id_)
    }

    fn id_(input: I) -> ParseResult<Expr, I>
        where I: Stream<Item=char>
    {
        many1(letter().or(char(':')))
            .skip(spaces())
            .map(Expr::Id)
            .parse_stream(input)
    }

    fn add() -> ExprParser<Expr, I> {
        fn_parser(Expression::<I>::add_)
    }

    fn add_(input: I) -> ParseResult<Expr, I>
        where I: Stream<Item=char>
    {
        let follow = char('+')
            .skip(spaces())
            .with(Expression::expr());
        let mut add = Expression::id()
            .and(follow)
            .map(|(l, r)| Expr::Add(Box::new(l), Box::new(r)));
        add.parse_stream(input)
    }

    fn sub() -> ExprParser<Expr, I> {
        fn_parser(Expression::<I>::sub_)
    }

    fn sub_(input: I) -> ParseResult<Expr, I>
        where I: Stream<Item=char>
    {
        let follow = char('-')
            .skip(spaces())
            .with(Expression::expr());
        let mut sub = Expression::id()
            .and(follow)
            .map(|(l, r)| Expr::Sub(Box::new(l), Box::new(r)));
        sub.parse_stream(input)
    }

    fn expr() -> ExprParser<Expr, I> {
        fn_parser(Expression::<I>::expr_)
    }

    fn expr_(input: I) -> ParseResult<Expr, I>
        where I: Stream<Item=char>
    {
        // TODO prevent 'assets -' invalid expressions
        try(Expression::add())
            .or(try(Expression::sub()))
            .or(Expression::id())
            .parse_stream(input)
    }
}
