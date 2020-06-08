use pest::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct TwilightParser;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let f =
            "Place Influence (2 Ops):\nUSSR +1 in Venezuela [0][2]\nUSSR +1 in South Africa [3][2]";
        let parsed = TwilightParser::parse(Rule::file, &f)
            .expect("Bad parse")
            .next()
            .unwrap();
        dbg!(parsed);
    }
}
