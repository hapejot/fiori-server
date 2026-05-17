use peg::{error::ParseError, str::LineCol};

#[derive(Debug)]
pub enum ODataPathError {
    InvalidSegment(String),
    UnexpectedEndOfInput,
    // Add more error variants as needed
}

#[derive(Debug)]
pub enum ODataPathSegment {
    EntitySet(String),
    KeyPredicate(String, Vec<(String, String)>), // e.g., ID='P001'
    NavigationProperty(String),
    Count,
}

pub fn parse_odata_resource_path(path: &str) -> Result<Vec<ODataPathSegment>, ParseError<LineCol>> {
    odata_uri::parse_str(path)
}

peg::parser! {
/// Parses OData v4 URI paths.
    grammar odata_uri() for str {
                /// Entry point for parsing a URI path string.
        pub rule parse_str() -> Vec<ODataPathSegment>
            = p:path() { p.unwrap() }

            /// Parses an identifier.
        rule identifier() -> String
            = s:$(['a'..='z'|'A'..='Z'|'_']['a'..='z'|'A'..='Z'|'_'|'0'..='9']+) { s.to_string() }

        rule path() -> Result<Vec<ODataPathSegment>, ODataPathError>
            = segments:segment() ** "/" {
                let mut r = vec![];
                for s in segments {
                    r.push(s?);
                }
                Ok(r)
            }


        rule key_predicate_list() -> Result<Vec<(String, String)>, ODataPathError>
            = first:key_predicate() rest:(key_predicate_rest())* {
                let mut keys = vec![first?];
                for key in rest {
                    keys.push(key?);
                }
                Ok(keys)
            }

        rule key_predicate() -> Result<(String, String), ODataPathError>
            = name:identifier() "=" value:literal() { Ok((name, value?)) }

        rule key_predicate_rest() -> Result<(String, String), ODataPathError>
            = "," k:key_predicate() { k }


        rule literal() -> Result<String, ODataPathError>
            = s:string_value() { s }
            / "true" { Ok("true".to_string()) }
            / "false" { Ok("false".to_string()) }

                    /// Parses a string value enclosed in single quotes.
        rule string_value() -> Result<String, ODataPathError>
            = "'" s:quote_escaped_string_content()* "'" { Ok(s.into_iter().collect::<Result<String, _>>()?) }

        rule quote_escaped_string_content() -> Result<char, ODataPathError>
            = r"\" e:escape_character() { e }
            / c:[^'\''] { Ok(c) }

        rule escape_character() -> Result<char, ODataPathError>
            = "'" { Ok('\'') }
            / "n" { Ok('\n') }
            / "r" { Ok('\r') }
            / "t" { Ok('\t') }
            / r"\" { Ok('\\') }
            // / "u" sequence:$(hex()*<1,8>) {
                // u32::from_str_radix(sequence, 16).ok().and_then(char::from_u32).ok_or(ParseError::ParsingUnicodeCodePoint)
            // }

        rule segment() -> Result<ODataPathSegment, ODataPathError>
            = key_predicate:identifier() "(" keys:key_predicate_list() ")" { Ok(ODataPathSegment::KeyPredicate(key_predicate, keys?)) }
            / entity_set:identifier() { Ok(ODataPathSegment::EntitySet(entity_set)) }
            / "$count" { Ok(ODataPathSegment::Count) }
    }
}
