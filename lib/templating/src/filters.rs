use std::collections::HashMap;

use minijinja::{Error, Value};

pub(crate) fn groupby(value: Value, attr: &str) -> Value {
    let mut map = HashMap::new();
    let v_i = value.try_iter().unwrap();
    for v in v_i {
        let p = get_path(&v, attr).unwrap();
        let values = map.entry(p).or_insert(Vec::new());
        values.push(v.clone());
    }

    let mut rv = Vec::with_capacity(map.len());
    for (k, v) in map {
        rv.push(Value::from(vec![k, Value::from(v)]));
    }

    Value::from(rv)
}

// trim_start_matches(pat="v")
// date(format="%Y-%m-%d")
// upper_first -- same as https://docs.rs/minijinja/latest/minijinja/filters/fn.capitalize.html / title case

//arrays
// nth
// sort
// unique
// map
// concat

//string
// trim_end
// trim_end_matches
// truncate
// wordcount
// replace
// linebreaksbr
// indent
// striptags
// spaceless
// split

pub(crate) fn get_path(val: &Value, path: &str) -> Result<Value, Error> {
    let mut rv = val.clone();
    for part in path.split('.') {
        if let Ok(num) = part.parse::<usize>() {
            rv = rv.get_item_by_index(num)?;
        } else {
            rv = rv.get_attr(part)?;
        }
    }
    Ok(rv)
}
