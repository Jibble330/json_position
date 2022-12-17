//! Json Position is json library for finding the path to json at an index in the original string.
//! Similar to extensions in most IDEs to get path to cursor position.
//! 
//! # Examples
//! 
//! ```
//! use json_position::dot_path;
//! 
//! let json = "[9, {\"name\": \"b\", \"fields\": [null, null, 87, 4], \"path\": \"file.txt\"}]";
//! let position = json.find("87").unwrap();
//! 
//! let dotted = dot_path(json, position).expect("Invalid JSON");
//! assert_eq!(dotted, "$.1.fields.2");
//! ```

extern crate oxidized_json_checker;

/// Index or key into an array or object
#[derive(Debug, PartialEq, Eq)]
pub enum Index {
    Array(usize),
    Object(String)
}

impl Index {
    fn increment(&mut self) {
        if let Index::Array(ref mut i) = self {
            *i += 1;
        }
    }
}

impl std::fmt::Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Index::Array(i) => i.to_string(),
            Index::Object(key) => key.to_owned()
        })
    }
}

#[derive(PartialEq)]
enum Current {
    Array,
    Object,
    None
}

fn end_quote(chars: &Vec<char>, start: usize) -> usize {
    let mut i = start;
    while i < chars.len() {
        if chars[i] == '\\' {
            i += 2;
        }

        if chars[i] == '"' {
            break;
        }
        i += 1;
    }
    i
}

fn substring(str: &Vec<char>, start: usize, end: usize) -> String {
    str.iter().skip(start).take(end-start).collect()
}

/// Constructs the path to an index in a raw json string.
///
/// # Examples
/// 
/// ```
/// use json_position::{path, Index};
/// 
/// let json = "[null, 9, {\"a\": \"b\"}]";
/// 
/// let vec_path = path(json, json.find("b").unwrap()).expect("Invalid JSON");
/// assert_eq!(vec_path, vec![Index::Array(2), Index::Object(String::from("a"))]);
/// ```
/// 
/// # Errors
/// 
/// Returns [`oxidized_json_checker::Error`] if the input json is invalid.
/// 
/// [`oxidized_json_checker::Error`]: https://docs.rs/oxidized-json-checker/0.3.2/oxidized_json_checker/enum.Error.html
pub fn path(text: &str, offset: usize) -> Result<Vec<Index>, oxidized_json_checker::Error> {
    oxidized_json_checker::validate_str(&text)?;

    let mut pos = 0;
    let mut path: Vec<Index> = Vec::new();
    let mut in_key = false;

    let mut current: Vec<Current> = vec![Current::None];
    
    let chars: Vec<char> = text.chars().collect();

    while pos < offset && pos < chars.len() {
        let start_pos = pos;
        match chars[pos] {
            '"' => {
                let i = end_quote(&chars, pos+1);
                let key = substring(&chars, pos+1, i);

                match current.last() {
                    Some(last) => {
                        if *last == Current::Object && in_key {
                            path.push(Index::Object(key));
                            in_key = false;
                            pos = i;
                        }
                    }
                    None => {}
                }
                
            }
            '{' => {
                current.push(Current::Object);
                in_key = true;
            }
            '[' => {
                path.push(Index::Array(0));
                current.push(Current::Array);
            }
            '}' => {
                path.pop();
                current.pop();
            }
            ']' => {
                path.pop();
                current.pop();
            }
            ',' => {
                match current.last() {
                    Some(last) => {
                        match last {
                            Current::Object => {
                                path.pop();
                                in_key = true;
                            },
                            Current::Array => {
                                let last = path.len()-1;
                                path[last].increment();
                            },
                            Current::None => {}
                        }
                    }
                    None => {}
                }
            }
            _ => ()
        }
        if pos == start_pos {
            pos += 1;
        }
    }

    Ok(path)
}

/// Constructs the path of an index in a raw json string. 
/// Returns path in a human readable format usable by most JsonPath crates.
///
/// # Examples
/// 
/// ```
/// use json_position::{dot_path};
///
/// let json = "[null, 9, {\"a\": \"b\"}]";
/// 
/// let path = dot_path(json, json.find("b").unwrap()).expect("Invalid JSON");
/// assert_eq!(path, "$.2.a");
/// ```
/// 
/// # Errors
/// 
/// Returns [`oxidized_json_checker::Error`] if the input json is invalid.
/// 
/// [`oxidized_json_checker::Error`]: https://docs.rs/oxidized-json-checker/0.3.2/oxidized_json_checker/enum.Error.html
pub fn dot_path(text: &str, offset: usize) -> Result<String, oxidized_json_checker::Error> {
    let p = path(text, offset)?;
    Ok(dots(&p))
}

fn dots(p: &Vec<Index>) -> String {
    let mut dotted = "$".to_owned();

    for i in p {
        dotted += ".";
        dotted += &match i {
            Index::Array(i) => i.to_string(),
            Index::Object(key) => key.to_owned()
        }
    }
    
    dotted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let json = "[9, {\"field1\": \"b\", \"field2\": [null, null, 87, 4], \"field3\": \"file.txt\"}]";
        
        // Tests regular path
        let vec_path = path(json, json.find("87").unwrap()).expect("Invalid JSON");
        assert_eq!(vec_path, vec![Index::Array(1), Index::Object(String::from("field2")), Index::Array(2)]);

        // Tests dotted path
        let dotted = dot_path(json, json.find("87").unwrap()).expect("Invalid JSON");
        assert_eq!(dotted, "$.1.field2.2");

        // Tests out of bounds 
        assert_eq!(path(json, 1000).unwrap(), vec![]);
    }
}
