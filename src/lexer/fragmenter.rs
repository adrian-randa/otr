use std::str::{Chars, FromStr};

use derive_more::IntoIterator;

use crate::error::{fragmenter_error::FragmentationError, Error};

struct CharCoordinateIterator<'a> {
    iter: Chars<'a>,
    line: usize,
    column: usize,
}

impl<'a> Iterator for CharCoordinateIterator<'a> {
    type Item = (char, usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let c = self.iter.next()?;
        self.column += 1;
        if c == '\n' {
            self.column = 1;
            self.line += 1;
        }

        Some((c, self.line, self.column))
    }
}

#[derive(Debug)]
pub struct Fragment {
    pub(crate) fragment: String,
    pub(crate) line_index: usize,
    pub(crate) column_index: usize,
}

#[derive(Debug, IntoIterator)]
pub struct FragmentStream(Vec<Fragment>);

impl FromStr for FragmentStream {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut stream = Vec::new();

        #[derive(Debug, PartialEq)]
        enum CharKind {
            Alphabetic,
            Numeric,
            Punctuation,
        }

        impl From<char> for CharKind {
            fn from(value: char) -> Self {
                if value.is_ascii_alphabetic() {
                    return Self::Alphabetic;
                }
                if value.is_numeric() {
                    return Self::Numeric;
                }
                if value.is_ascii_punctuation() {
                    return Self::Punctuation;
                }

                panic!("Unsupported char kind");
            }
        }

        let mut current = String::new();
        let mut current_pos = (0, 0);
        let mut current_kind = CharKind::Alphabetic;

        let s = s.to_string();

        let chars: Vec<(char, usize, usize)> = CharCoordinateIterator {
            iter: s.chars(),
            line: 1,
            column: 1,
        }
        .collect();

        let (_, line, column) = chars.last().unwrap();

        let mut i = 0;

        while i < chars.len() {
            let (c, line, column) = chars[i];

            i += 1;

            if c == '\'' {
                if !current.is_empty() {
                    stream.push(Fragment {
                        fragment: current,
                        line_index: current_pos.0,
                        column_index: current_pos.1,
                    });
                    current_pos = (line, column);
                    current = String::new();
                }

                current.push('\'');

                current.push(chars[i].0);

                current.push('\'');

                stream.push(Fragment {
                    fragment: current,
                    line_index: current_pos.0,
                    column_index: current_pos.1,
                });
                current_pos = (line, column);
                current = String::new();

                i += 2;
                continue;
            }

            if c == '\"' {
                if !current.is_empty() {
                    stream.push(Fragment {
                        fragment: current,
                        line_index: current_pos.0,
                        column_index: current_pos.1,
                    });
                    current_pos = (line, column);
                    current = String::new();
                }

                current.push('\"');

                while chars[i].0 != '\"' {
                    if chars[i].0 == '\\' {
                        match chars[i + 1].0 {
                            'n' => {
                                current.push('\n');
                            }
                            't' => {
                                current.push('\t');
                            }
                            '\"' => {
                                current.push('\"');
                            }
                            '\\' => {
                                current.push('\\');
                            }
                            _ => {
                                return Err(FragmentationError::InvalidControlCharacter {
                                    line_index: line,
                                    column_index: column,
                                }
                                .boxed())
                            }
                        }
                        i = i + 2;
                        continue;
                    }
                    if chars[i].0 == '\n' {
                        return Err(FragmentationError::LinebreakInStringLiteral {
                            line_index: line,
                            column_index: column,
                        }
                        .boxed());
                    }

                    current.push(chars[i].0);

                    i += 1;
                }

                current.push('\"');

                stream.push(Fragment {
                    fragment: current,
                    line_index: current_pos.0,
                    column_index: current_pos.1,
                });
                current_pos = (line, column);
                current = String::new();

                i += 1;
                continue;
            }

            if c.is_ascii_whitespace() {
                if current.is_empty() {
                    current_pos = (line, column);
                    continue;
                }
                stream.push(Fragment {
                    fragment: current,
                    line_index: current_pos.0,
                    column_index: current_pos.1,
                });
                current_pos = (line, column);
                current = String::new();
                continue;
            }

            if c == '#' {
                if !current.is_empty() {
                    stream.push(Fragment {
                        fragment: current,
                        line_index: current_pos.0,
                        column_index: current_pos.1,
                    });
                    current_pos = (line, column);
                    current = String::new();
                }

                while chars[i].0 != '\n' && i < chars.len() {
                    i += 1;
                }

                continue;
            }

            if c == ';' {
                stream.push(Fragment {
                    fragment: current,
                    line_index: current_pos.0,
                    column_index: current_pos.1,
                });
                stream.push(Fragment {
                    fragment: ";".into(),
                    line_index: current_pos.0,
                    column_index: column,
                });
                current_pos = (line, column);
                current = String::new();
                continue;
            }

            let next_char_kind: CharKind = c.into();

            if !current.is_empty() {
                use CharKind::*;
                match (current_kind, next_char_kind) {
                    (Alphabetic, Punctuation)
                    | (Punctuation, Alphabetic)
                    /*| (Numeric, Alphabetic) */ => {
                        stream.push(Fragment {
                            fragment: current,
                            line_index: current_pos.0,
                            column_index: current_pos.1,
                        });
                        current_pos = (line, column);
                        current = String::new();
                    }
                    (Numeric, Punctuation) => {
                        if c != '.' {
                            stream.push(Fragment {
                                fragment: current,
                                line_index: current_pos.0,
                                column_index: current_pos.1,
                            });
                            current_pos = (line, column);
                            current = String::new();
                        }
                    }

                    _ => {}
                }
            }

            current_kind = c.into();

            current.push(c);
        }

        if !current.is_empty() {
            stream.push(Fragment {
                fragment: current,
                line_index: current_pos.0,
                column_index: current_pos.1,
            });
        }

        Ok(Self(stream))
    }
}
