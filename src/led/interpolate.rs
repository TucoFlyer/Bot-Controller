use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, Number, Map, to_value, from_value};
use std::iter::FromIterator;

pub fn serde_interpolate<T: Serialize + DeserializeOwned>(from: &T, to: &T, amount: f64) -> T {
    let from = to_value(from).unwrap();
    let to = to_value(to).unwrap();
    let result = value_interpolate(&from, &to, amount);
    from_value(result).unwrap()
}

fn value_interpolate(from: &Value, to: &Value, amount: f64) -> Value {
    match to {
        &Value::Number(ref to) => Value::Number(match from {
            &Value::Number(ref from) => number_interpolate(from, to, amount),
            _ => to.clone(),
        }),
        &Value::Array(ref to) => Value::Array(match from {
            &Value::Array(ref from) => array_interpolate(from, to, amount),
            _ => to.clone(),
        }),
        &Value::Object(ref to) => Value::Object(match from {
            &Value::Object(ref from) => object_interpolate(from, to, amount),
            _ => to.clone(),
        }),
        _ => to.clone(),
    }
}

fn number_interpolate(from: &Number, to: &Number, amount: f64) -> Number {
    match to.as_f64() {
        None => to.clone(),
        Some(to) => Number::from_f64(match from.as_f64() {
            None => to,
            Some(from) => from + (to - from) * amount
        }).unwrap(),
    }
}

fn array_interpolate(from: &Vec<Value>, to: &Vec<Value>, amount: f64) -> Vec<Value> {
    to.iter().enumerate().map(|(index, to)| {
        match from.get(index) {
            None => to.clone(),
            Some(ref from) => value_interpolate(from, to, amount)
        }
    }).collect()
}

fn object_interpolate(from: &Map<String, Value>, to: &Map<String, Value>, amount: f64) -> Map<String, Value> {
    Map::from_iter(to.iter().map(|(key, to)| {
        let value = match from.get(key) {
            None => to.clone(),
            Some(ref from) => value_interpolate(from, to, amount),
        };
        (key.clone(), value)
    }))
}
