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
            "Place Influence (2 Ops):\nUSSR +1 in Venezuela [0][2]\nUSSR +1 in South Africa [3][2]\n";
        let f2 = "Realignment (2 Ops):
Target: Angola
USSR rolls 6
US rolls 4 (+2) = 6
Target: Angola
USSR rolls 1
US rolls 5 (+2) = 7
USSR -4 in Angola [4][0]\n";

        let f3 = "Event: AWACS Sale To Saudis*
US +2 in Saudi Arabia [2][3]
AWACS Sale To Saudis* is now in play.
";

        let f4 = "Coup (2 Ops):
Target: Nigeria
SUCCESS: 3 [ + 2 - 2x1 = 3 ]
US -1 in Nigeria [0][0]
USSR +2 in Nigeria [0][2]
USSR Military Ops to 2
DEFCON degrades to 2
";
        for string in [f, f2, f3, f4].iter() {
            let parsed = TwilightParser::parse(Rule::file, &string)
                .expect("Bad parse")
                .next()
                .unwrap();
            dbg!(parsed);
        }
    }
}
