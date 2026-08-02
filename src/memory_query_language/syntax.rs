use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use super::{MemoryQueryError, MemoryQueryValue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Token {
    Word(String),
    Quoted(String),
    Number(String),
    Symbol(char),
    Operator(String),
}

pub(super) fn tokenize(source: &str) -> Result<Vec<Token>, MemoryQueryError> {
    let mut tokens = Vec::new();
    let mut cursor = 0;
    while cursor < source.len() {
        let character = source[cursor..]
            .chars()
            .next()
            .expect("cursor is inside source");
        if character.is_whitespace() {
            cursor += character.len_utf8();
            continue;
        }
        if matches!(character, '\'' | '"' | '`') {
            let (value, consumed) = read_quoted(&source[cursor..], character)?;
            tokens.push(Token::Quoted(value));
            cursor += consumed;
            continue;
        }
        if character.is_ascii_digit()
            || (character == '-'
                && source[cursor + character.len_utf8()..]
                    .chars()
                    .next()
                    .is_some_and(|next| next.is_ascii_digit()))
        {
            let consumed = source[cursor..]
                .char_indices()
                .take_while(|(_, found)| {
                    found.is_ascii_digit() || matches!(found, '-' | '+' | '.' | 'e' | 'E')
                })
                .map(|(position, found)| position + found.len_utf8())
                .last()
                .unwrap_or_else(|| character.len_utf8());
            tokens.push(Token::Number(source[cursor..cursor + consumed].to_owned()));
            cursor += consumed;
            continue;
        }
        if is_word_start(character) {
            let consumed = source[cursor..]
                .char_indices()
                .take_while(|(_, found)| is_word_continue(*found))
                .map(|(position, found)| position + found.len_utf8())
                .last()
                .unwrap_or_else(|| character.len_utf8());
            tokens.push(Token::Word(source[cursor..cursor + consumed].to_owned()));
            cursor += consumed;
            continue;
        }
        if matches!(character, '<' | '>' | '!' | '=') {
            let mut operator = character.to_string();
            cursor += character.len_utf8();
            if source[cursor..].starts_with('=')
                || (character == '<' && source[cursor..].starts_with('>'))
            {
                operator.push(source[cursor..].chars().next().expect("checked prefix"));
                cursor += 1;
            }
            tokens.push(Token::Operator(operator));
            continue;
        }
        if matches!(
            character,
            '(' | ')' | '{' | '}' | '[' | ']' | ',' | ':' | ';' | '*' | '$'
        ) {
            tokens.push(Token::Symbol(character));
            cursor += character.len_utf8();
            continue;
        }
        return Err(MemoryQueryError::new(format!(
            "unexpected_character:{cursor}:{character}"
        )));
    }
    Ok(tokens)
}

fn read_quoted(input: &str, delimiter: char) -> Result<(String, usize), MemoryQueryError> {
    let mut value = String::new();
    let mut characters = input.char_indices();
    let _ = characters.next();
    while let Some((position, character)) = characters.next() {
        if character == delimiter {
            if input[position + character.len_utf8()..].starts_with(delimiter) {
                value.push(delimiter);
                let _ = characters.next();
                continue;
            }
            return Ok((value, position + character.len_utf8()));
        }
        if character == '\\' {
            let Some((_, escaped)) = characters.next() else {
                return Err(MemoryQueryError::new("quoted_value_unterminated"));
            };
            value.push(match escaped {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            });
        } else {
            value.push(character);
        }
    }
    Err(MemoryQueryError::new("quoted_value_unterminated"))
}

fn is_word_start(character: char) -> bool {
    character.is_alphabetic() || matches!(character, '_' | '@')
}

fn is_word_continue(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '_' | '-' | '.' | '@')
}

pub(super) struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    pub(super) fn new(source: &str) -> Result<Self, MemoryQueryError> {
        Ok(Self {
            tokens: tokenize(source)?,
            cursor: 0,
        })
    }

    pub(super) fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
    }

    pub(super) fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.cursor).cloned();
        self.cursor += usize::from(token.is_some());
        token
    }

    pub(super) fn word(&mut self) -> Result<String, MemoryQueryError> {
        match self.next() {
            Some(Token::Word(value) | Token::Quoted(value)) => Ok(value),
            found => Err(self.expected("identifier", found.as_ref())),
        }
    }

    pub(super) fn value(&mut self) -> Result<MemoryQueryValue, MemoryQueryError> {
        match self.next() {
            Some(Token::Number(value)) if value.contains(['.', 'e', 'E']) => value
                .parse::<f64>()
                .map(MemoryQueryValue::Float)
                .map_err(|_| MemoryQueryError::new(format!("invalid_number:{value}"))),
            Some(Token::Number(value)) => value
                .parse::<i64>()
                .map(MemoryQueryValue::Integer)
                .map_err(|_| MemoryQueryError::new(format!("invalid_integer:{value}"))),
            Some(Token::Word(value)) if value.eq_ignore_ascii_case("null") => {
                Ok(MemoryQueryValue::Null)
            }
            Some(Token::Word(value)) if value.eq_ignore_ascii_case("true") => {
                Ok(MemoryQueryValue::Boolean(true))
            }
            Some(Token::Word(value)) if value.eq_ignore_ascii_case("false") => {
                Ok(MemoryQueryValue::Boolean(false))
            }
            Some(Token::Quoted(value) | Token::Word(value)) => Ok(MemoryQueryValue::Text(value)),
            found => Err(self.expected("value", found.as_ref())),
        }
    }

    pub(super) fn eat_word(&mut self, expected: &str) -> bool {
        let found = matches!(
            self.peek(),
            Some(Token::Word(value)) if value.eq_ignore_ascii_case(expected)
        );
        self.cursor += usize::from(found);
        found
    }

    pub(super) fn expect_word(&mut self, expected: &str) -> Result<(), MemoryQueryError> {
        if self.eat_word(expected) {
            Ok(())
        } else {
            Err(self.expected(expected, self.peek()))
        }
    }

    pub(super) fn eat_symbol(&mut self, expected: char) -> bool {
        let found = self.peek() == Some(&Token::Symbol(expected));
        self.cursor += usize::from(found);
        found
    }

    pub(super) fn expect_symbol(&mut self, expected: char) -> Result<(), MemoryQueryError> {
        if self.eat_symbol(expected) {
            Ok(())
        } else {
            Err(self.expected(&format!("`{expected}`"), self.peek()))
        }
    }

    pub(super) fn eat_operator(&mut self) -> Option<String> {
        match self.peek() {
            Some(Token::Operator(value)) => {
                let value = value.clone();
                self.cursor += 1;
                Some(value)
            }
            _ => None,
        }
    }

    pub(super) fn is_word(&self, expected: &str) -> bool {
        matches!(self.peek(), Some(Token::Word(value)) if value.eq_ignore_ascii_case(expected))
    }

    pub(super) fn finish(&mut self) -> Result<(), MemoryQueryError> {
        let _ = self.eat_symbol(';');
        if self.peek().is_none() {
            Ok(())
        } else {
            Err(self.expected("end of query", self.peek()))
        }
    }

    fn expected(&self, expected: &str, found: Option<&Token>) -> MemoryQueryError {
        MemoryQueryError::new(format!(
            "expected:{}:{expected}:{}",
            self.cursor,
            found.map_or_else(
                || String::from("end_of_query"),
                |token| format!("{token:?}")
            )
        ))
    }
}
