use pest_derive::Parser;

use pest::Parser;

#[derive(Parser)]
#[grammar = "./aml3/grammar.pest"]
pub struct Aml3Parser;

pub fn parse_file<'a>(
    input: &'a str,
) -> Result<pest::iterators::Pairs<'a, Rule>, pest::error::Error<Rule>> {
    Aml3Parser::parse(Rule::program, input)
}
