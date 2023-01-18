# JSON Position

JSON Position is json library for finding the path to JSON at an index in the original string.
Similar to extensions in most IDEs to get path to cursor position.

## Examples

```rust
use json_position::{ path, dot_path, Index };

let json = r#"[9, {"name": "b", "fields": [null, null, 87, 4], "path": "file.txt"}]"#;
let position = json.find("87").unwrap();

let vec_path = path(json, position).expect("Invalid JSON");
assert_eq!(vec_path, [Index::Array(1), Index::Object("fields".to_string()), Index::Array(2)]);

let dotted = dot_path(json, position).expect("Invalid JSON");
assert_eq!(dotted, "$.1.fields.2");
```

In this example we start with the raw JSON string `[9, {"name": "b", "fields": [null, null, 87, 4]}]`</br>
We are trying to find the path to the first `87` contained in the string, which starts at char 42.
This can be indexed to with the path `json[1]["fields"][2]`
The `path` function returns this path in a `Vec<Index>`. `Index` is an `enum` with two varients. One is `Array`, which is an index into an array, and `Object` which is a key in an object.
The `dot_path` function returns this path in a format used by most JsonPath libraries: `"$.1.fields.2"`
