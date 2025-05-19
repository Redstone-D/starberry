use crate::keywords::{Keyword, ALL_KEYWORDS, ALL_KEYWORDS_INDEX};

/// Map a string to a Keyword enum (case-insensitive), or `NoKeyword` if not found.
pub fn to_keyword(s: &str) -> Keyword {
    let up = s.to_uppercase();
    ALL_KEYWORDS
        .iter()
        .position(|&kw| kw.eq_ignore_ascii_case(&up))
        .map(|i| ALL_KEYWORDS_INDEX[i])
        .unwrap_or(Keyword::NoKeyword)
}

/// Tokenize SQL into simple string tokens (keywords, identifiers, commas, semicolons).
pub fn tokenize_sql(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    for c in input.chars() {
        if c.is_alphanumeric() {
            cur.push(c);
        } else {
            if !cur.is_empty() {
                tokens.push(cur.clone());
                cur.clear();
            }
            if ",;()".contains(c) {
                tokens.push(c.to_string());
            }
        }
    }
    if !cur.is_empty() {
        tokens.push(cur);
    }
    tokens
}

/// Dialect trait: parse a token stream, returning Ok or Err(message).
pub trait SqlDialect {
    fn parse_tokens(&self, tokens: &[String]) -> Result<(), &'static str>;
}

/// ANSI SQL subset: supports SELECT ... FROM ... and ALTER TABLE ... ADD|DROP ...
pub struct AnsiSqlDialect;

impl SqlDialect for AnsiSqlDialect {
    fn parse_tokens(&self, tokens: &[String]) -> Result<(), &'static str> {
        // Classify first token as keyword
        let first_kw = to_keyword(tokens.get(0).ok_or("empty query")?);
        if first_kw == Keyword::SELECT {
            // Expect FROM
            let from_pos = tokens
                .iter()
                .position(|t| to_keyword(t) == Keyword::FROM)
                .ok_or("missing FROM clause")?;
            if from_pos < 2 {
                return Err("SELECT must include at least one expression before FROM");
            }
            let table = tokens.get(from_pos + 1).ok_or("missing table name after FROM")?;
            if table.is_empty() {
                return Err("empty table name");
            }
            Ok(())
        } else if first_kw == Keyword::ALTER {
            // Expect TABLE keyword
            if to_keyword(tokens.get(1).unwrap()) != Keyword::TABLE {
                return Err("expected TABLE after ALTER");
            }
            let table = tokens.get(2).ok_or("missing table name after ALTER TABLE")?;
            if table.is_empty() {
                return Err("empty table name");
            }
            // Expect ADD or DROP
            let action_kw = to_keyword(tokens.get(3).unwrap());
            if action_kw != Keyword::ADD && action_kw != Keyword::DROP {
                return Err("expected ADD or DROP after ALTER TABLE <name>");
            }
            Ok(())
        } else {
            Err("only SELECT and ALTER TABLE statements are supported in sql! macro")
        }
    }
} 