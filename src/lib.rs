extern crate oxidized_json_checker;

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

/// Returns the path of the position in a human readable format usable by most JsonPath crates
///
/// # Examples
/// 
/// ```
/// use json_position::{dot_path, Index};
///
/// let json = "[null, 9, {\"a\": \"b\"}]".to_owned();
/// assert_eq!(path(json, json.find("b")), vec![Index::Array(2), Index::Object("a".to_owned())])
pub fn path(text: &str, offset: usize) -> Result<Vec<Index>, oxidized_json_checker::Error> {
    oxidized_json_checker::validate_str(&text)?;

    let mut pos = 0;
    let mut path: Vec<Index> = Vec::new();
    let mut in_key = false;

    let mut current: Vec<Current> = vec![Current::None];
    
    let chars: Vec<char> = text.chars().collect();

    while pos < offset {
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
                            pos += i;
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
                            Current::Object => {in_key = true;},
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

/// Returns the path of the position in a human readable format usable by most JsonPath crates
///
/// # Examples
/// 
/// ```
/// use json_position::{dot_path};
///
/// let json = "[null, 9, {\"a\": \"b\"}]".to_owned();
/// let path = dot_path(&json, json.find("b").unwrap()).expect("Invalid JSON");
/// assert_eq!(path, "$.2.a");
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
        let json = "[null, 9, {\"a\": \"b\"}]".to_owned();

        let dotted = dot_path(&json, json.find("b").unwrap()).expect("Invalid JSON");
        assert_eq!(dotted, "$.2.a");

        let vec_path = path(&json, json.find("b").unwrap()).expect("Invalid JSON");
        assert_eq!(vec_path, vec![Index::Array(2), Index::Object("a".to_owned())]);
    }
}
