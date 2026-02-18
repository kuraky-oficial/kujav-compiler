use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/kujav.pest"]
pub struct KujavParser;