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
        let first = tokens.get(0).ok_or("empty query")?.to_uppercase();
        if first == "SELECT" {
            let from_pos = tokens.iter()
                .position(|t| t.to_uppercase() == "FROM")
                .ok_or("missing FROM clause")?;
            if from_pos < 2 {
                return Err("SELECT must include at least one expression before FROM");
            }
            let table = tokens.get(from_pos + 1).ok_or("missing table name after FROM")?;
            if table.is_empty() {
                return Err("empty table name");
            }
            Ok(())
        } else if first == "ALTER" {
            if tokens.get(1).map(|t| t.to_uppercase()) != Some("TABLE".to_string()) {
                return Err("expected TABLE after ALTER");
            }
            let table = tokens.get(2).ok_or("missing table name after ALTER TABLE")?;
            if table.is_empty() {
                return Err("empty table name");
            }
            let action = tokens.get(3).ok_or("missing ADD or DROP after table name")?.to_uppercase();
            if action != "ADD" && action != "DROP" {
                return Err("expected ADD or DROP after ALTER TABLE <name>");
            }
            Ok(())
        } else {
            Err("only SELECT and ALTER TABLE statements are supported in sql! macro")
        }
    }
} 