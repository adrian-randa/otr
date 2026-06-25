use std::str::Chars;

use derive_more::IntoIterator;

use crate::lexer::error::FragmentationError;
use otr_core::error::Result;

#[allow(unused)]
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
}

#[derive(Debug, IntoIterator)]
pub struct FragmentStream(Vec<Fragment>);

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

pub struct Fragmenter {
    fragments: Vec<Fragment>,
    current: String,
}

impl Fragmenter {
    fn finalize_fragment(&mut self, line_index: usize) {
        if !self.current.is_empty() {
            let mut fragment = Fragment { fragment: String::new(), line_index };
            
            std::mem::swap(&mut fragment.fragment, &mut self.current);
    
            self.fragments.push(fragment);
        }
    } 
    
    pub fn fragment(s: impl AsRef<str>) -> Result<FragmentStream> {
        let mut this = Self {
            fragments: Vec::new(),
            current: String::new()
        };

        'line: for (line_index, line) in s.as_ref().lines().enumerate() {
            let chars: Vec<char> = line.chars().collect();
            
            let mut i = 0;
            while i < chars.len() {
                match chars[i] {
                    '\'' => {
                        this.finalize_fragment(line_index);

                        if chars.get(i + 2).is_none_or(|c| *c != '\'') {
                            return Err(FragmentationError::InvalidCharLiteral { line_index, column_index: i }.boxed());
                        }
                        this.finalize_fragment(line_index);
                        this.current = ['\'', chars[i + 1], '\''].iter().collect();
                        this.finalize_fragment(line_index);
                        
                        i += 2;
                    }
                    '\"' => {
                        this.finalize_fragment(line_index);

                        this.current.push('\"');

                        i += 1;

                        while chars[i] != '\"' {
                            if chars[i] == '\\' {
                                match chars.get(i + 1) {
                                    Some('n') => {
                                        this.current.push('\n');
                                    }
                                    Some('t') => {
                                        this.current.push('\t');
                                    }
                                    Some('\"') => {
                                        this.current.push('\"');
                                    }
                                    Some('\\') => {
                                        this.current.push('\\');
                                    }
                                    Some(_) => {
                                        return Err(FragmentationError::InvalidControlCharacter {
                                            line_index,
                                            column_index: i,
                                        }
                                        .boxed())
                                    }
                                    None => {
                                        return Err(
                                            FragmentationError::LinebreakInStringLiteral { line_index, column_index: i }.boxed()
                                        );
                                    }
                                }
                                i += 2;
                                continue;
                            }

                            this.current.push(chars[i]);

                            i += 1;

                            if i >= chars.len() {
                                return Err(
                                    FragmentationError::LinebreakInStringLiteral { line_index, column_index: i }.boxed()
                                );
                            }
                        }

                        this.current.push('\"');

                        this.finalize_fragment(line_index);
                    }
                    '#' => {
                        this.finalize_fragment(line_index);
                        continue 'line;
                    }
                    ';' => {
                        this.finalize_fragment(line_index);
                        this.current = ";".into();
                        this.finalize_fragment(line_index);
                    }
                    _ => {
                        if chars[i].is_ascii_whitespace() {
                            this.finalize_fragment(line_index);
                        } else {
                            if !this.current.is_empty() && i > 0 {
                                use CharKind::*;
                                match (CharKind::from(chars[i - 1]), CharKind::from(chars[i])) {
                                    (Alphabetic, Punctuation)
                                    | (Punctuation, Alphabetic)
                                    /*| (Numeric, Alphabetic) */ => {
                                        this.finalize_fragment(line_index);
                                    }
                                    (Numeric, Punctuation) => {
                                        if chars[i] != '.' {
                                            this.finalize_fragment(line_index);
                                        }
                                    }
            
                                    _ => {}
                                }
                            }
                            this.current.push(chars[i]);
                        }
                    }
                }


                i += 1;
            }

            this.finalize_fragment(line_index);
        }

        Ok(FragmentStream(this.fragments))
    }
}