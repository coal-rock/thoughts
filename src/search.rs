use chrono::{self, NaiveDate};

use crate::tui::Entry;

#[derive(Debug, PartialEq, Clone)]
pub enum SearchToken {
    Boolean(bool),
    String(String),
    Date(NaiveDate),

    Favorite,
    Before,
    During,
    After,
    Contains,
    InTitle,
    StartsWith,
    ID,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub content: SearchToken,
}

#[derive(Debug)]
pub struct TokenStream {
    pub tokens: Vec<Token>,
    index: usize,
}

impl TokenStream {
    fn new(tokens: Vec<Token>) -> TokenStream {
        TokenStream { tokens, index: 0 }
    }

    fn peek(&self) -> Option<Token> {
        let tokens = self.tokens.clone();
        let token = tokens.get(self.index);

        if token.is_some() {
            Some(token.unwrap().clone())
        } else {
            None
        }
    }

    fn consume(&mut self) -> Option<Token> {
        let token = self.peek();
        self.index += 1;

        token
    }
}

fn parse_tokens(tokens: &mut TokenStream) -> Option<Vec<(Token, Token)>> {
    let mut search = Vec::new();

    while tokens.peek().is_some() {
        match tokens.peek().unwrap().content {
            SearchToken::Favorite => {
                let curr = tokens.consume().unwrap();
                let next = tokens.consume();

                if next.is_none() {
                    return Some(search);
                }

                let next = next.unwrap();

                match next.content {
                    SearchToken::Boolean(_) => search.push((curr, next)),
                    _ => {}
                }
            }
            SearchToken::Before | SearchToken::During | SearchToken::After => {
                let curr = tokens.consume().unwrap();
                let next = tokens.consume();

                if next.is_none() {
                    return Some(search);
                }

                let next = next.unwrap();

                match next.content {
                    SearchToken::Date(_) => search.push((curr, next)),
                    _ => {}
                }
            }

            SearchToken::Contains => {
                let curr = tokens.consume().unwrap();
                let next = tokens.consume();

                if next.is_none() {
                    return Some(search);
                }

                let next = next.unwrap();

                match next.content {
                    SearchToken::String(_) => search.push((curr, next)),
                    _ => {}
                }
            }

            SearchToken::String(_) | SearchToken::InTitle => {
                let in_title = Token {
                    content: SearchToken::InTitle,
                };

                search.push((in_title, tokens.consume().unwrap()));
            }

            SearchToken::StartsWith => {
                let starts_with = tokens.consume().unwrap();

                if let Some(token) = tokens.consume() {
                    search.push((starts_with, token));
                }
            }

            // Not sure if I like this behaivior.
            SearchToken::Boolean(bool) => {
                tokens.consume();

                search.push((
                    Token {
                        content: SearchToken::Contains,
                    },
                    Token {
                        content: SearchToken::String(bool.to_string()),
                    },
                ));
            }
            SearchToken::Date(_) => _ = tokens.consume(),
            SearchToken::ID => {
                let id = tokens.consume().unwrap();

                if let Some(token) = tokens.consume() {
                    search.push((id, token));
                }
            }
        }
    }

    Some(search)
}

// fix this enum hackery
pub fn tokenize_search(query: String) -> TokenStream {
    let mut tokens: Vec<Token> = Vec::new();

    for chunk in query.split_ascii_whitespace() {
        let token: SearchToken;

        match chunk {
            "favorite:" => token = SearchToken::Favorite,
            "before:" => token = SearchToken::Before,
            "during:" => token = SearchToken::During,
            "after:" => token = SearchToken::After,
            "contains:" => token = SearchToken::Contains,
            "startswith:" => token = SearchToken::StartsWith,
            "id:" => token = SearchToken::ID,
            "true" => token = SearchToken::Boolean(true),
            "false" => token = SearchToken::Boolean(false),
            _ => {
                let mut date = None;

                'sep: for sep in vec!["-", "/", "."] {
                    let mut parts = Vec::new();

                    for part in chunk.split(sep) {
                        if !(part.parse::<i32>().is_ok()) {
                            continue 'sep;
                        }
                        parts.push(part.parse::<i32>().unwrap());
                    }

                    if parts.len() != 3 {
                        continue 'sep;
                    }

                    date = NaiveDate::from_ymd_opt(parts[2], parts[0] as u32, parts[1] as u32);
                }
                if date.is_some() {
                    token = SearchToken::Date(date.unwrap());
                } else {
                    token = SearchToken::String(chunk.to_string());
                }
            }
        }

        tokens.push(Token { content: token });
    }

    TokenStream::new(tokens)
}

// This should be handled by sqlite, but this is *FINE*
// forgive me for this awful function
// (i don't understand dynamic programming with enums)
pub fn search(entries: Vec<Entry>, query: String) -> Vec<Entry> {
    let mut entries = entries.clone();

    let mut tokens = tokenize_search(query);
    let search = parse_tokens(&mut tokens);

    for param in search.unwrap_or(vec![]) {
        entries = entries
            .iter()
            .map(|e| e.clone())
            .filter(|e| match param.0.content {
                SearchToken::Favorite => {
                    if let SearchToken::Boolean(bool) = param.1.content.clone() {
                        e.favorite == bool
                    } else {
                        false
                    }
                }
                SearchToken::Contains => {
                    if let SearchToken::String(str) = param.1.content.clone() {
                        e.content.to_lowercase().contains(&str.to_lowercase())
                    } else {
                        false
                    }
                }
                SearchToken::InTitle => {
                    if let SearchToken::String(str) = param.1.content.clone() {
                        e.title.to_lowercase().contains(&str.to_lowercase())
                    } else {
                        false
                    }
                }
                SearchToken::StartsWith => {
                    if let SearchToken::String(str) = param.1.content.clone() {
                        e.title.to_lowercase().starts_with(&str.to_lowercase())
                    } else {
                        false
                    }
                }
                SearchToken::Before => {
                    if let SearchToken::Date(date) = param.1.content.clone() {
                        e.dt.date() < date
                    } else {
                        false
                    }
                }
                SearchToken::During => {
                    if let SearchToken::Date(date) = param.1.content.clone() {
                        e.dt.date() == date
                    } else {
                        false
                    }
                }
                SearchToken::After => {
                    if let SearchToken::Date(date) = param.1.content.clone() {
                        e.dt.date() > date
                    } else {
                        false
                    }
                }
                SearchToken::ID => {
                    if let SearchToken::String(id) = param.1.content.clone() {
                        let id = id.parse();
                        if id.is_ok() {
                            e.id == id.unwrap()
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                _ => false,
            })
            .collect();
    }

    entries.sort_by_key(|e| e.dt);
    entries
}
