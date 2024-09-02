use std::collections::HashMap;
use std::fmt;

use chrono::{DateTime, FixedOffset, MappedLocalTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::{ParseError, Tz};
use minijinja::value::Kwargs;
use minijinja::{Error, ErrorKind, Value};

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

pub(crate) fn date(value: Value, kwargs: Kwargs) -> Result<Value, Error> {
    // pub(crate) fn date(
    //     value: Value,
    //     format: Option<&str>,
    //     tz: Option<&str>,
    //     locale: Option<&str>
    // ) -> Result<Value, Error> {
    // TODO: how to determine local vs UTC
    // TODO: timezone / locale

    println!("are we entering custom filter????");
    println!("{}", format!("value [{:?}]", &value));

    // let format = format.unwrap_or_else(|| "%Y-%m-%d");
    let format = kwargs
        .get::<Option<&str>>("format")?
        .unwrap_or_else(|| "%Y-%m-%d");
    let tzname = kwargs.get::<Option<&str>>("tz")?;

    // let timezone = match tz {
    //     Some(val) => {
    //         match val.parse::<Tz>() {
    //             Ok(timezone) => Some(timezone),
    //             Err(_) => {
    //                 return Err(Error::new(
    //                     ErrorKind::InvalidOperation,
    //                     format!("Error parsing `{}` as a timezone", val),
    //                 ));
    //                 // return Err(Error::from(Err(format!("Error parsing `{}` as a timezone", val))))
    //             }
    //         }
    //     },
    //     None => None,
    // };

    let timezone: Option<Tz> = match kwargs.get::<Option<&str>>("tz")? {
        Some(val) => match val.parse::<Tz>() {
            Ok(timezone) => Some(timezone),
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    format!("Error parsing `{}` as a timezone", val),
                ));
                // return Err(Error::from(Err(format!("Error parsing `{}` as a timezone", val))))
            }
        },
        None => None,
    };
    //
    // let locale = match locale {
    //     Some(val) => chrono::Locale::try_from(val).map_err(|_| {
    //         Error::new(
    //             ErrorKind::InvalidOperation,
    //             format!("Error parsing `{}` as a locale", val),
    //         )
    //     })?,
    //     None => chrono::Locale::POSIX,
    // };

    let locale = match kwargs.get::<Option<&str>>("locale")? {
        Some(val) => chrono::Locale::try_from(val).map_err(|_| {
            Error::new(
                ErrorKind::InvalidOperation,
                format!("Error parsing `{}` as a locale", val),
            )
        })?,
        None => chrono::Locale::POSIX,
    };

    let datetime = if let Some(value) = value.as_i64() {
        let date = DateTime::from_timestamp(value, 0)
            .expect("out of bound seconds should not appear, as we set nanoseconds to zero");

        match timezone {
            // Some(timezone) => timezone
            //     .from_utc_datetime(&date)
            //     .format_localized(&format, locale),
            Some(timezone) => date
                .with_timezone(&timezone)
                .format_localized(&format, locale),
            None => date.format(&format),
        }

        // TODO: whats the differnce between above an this?
        // match Utc.timestamp_millis_opt(value) {
        //     MappedLocalTime::Single(dt) => dt,
        //     _ => panic!("Incorrect timestamp_millis"),
        // }
    } else if let Some(value) = value.as_str() {
        if value.contains('T') {
            match value.parse::<DateTime<FixedOffset>>() {
                Ok(val) => match timezone {
                    Some(timezone) => val
                        .with_timezone(&timezone)
                        .format_localized(&format, locale),
                    None => val.format_localized(&format, locale),
                },
                Err(_) => {
                    let native_dt = value.parse::<NaiveDateTime>().map_err(|_| {
                        Error::new(
                            ErrorKind::InvalidOperation,
                            format!(
                                "Error parsing `{:?}` as rfc3339 date or naive datetime",
                                value
                            ),
                        )
                    })?;
                    DateTime::<Utc>::from_naive_utc_and_offset(native_dt, Utc)
                        .format_localized(&format, locale)
                }
            }
        } else {
            match NaiveDate::parse_from_str(value, "%Y-%m-%d") {
                Ok(val) => DateTime::<Utc>::from_naive_utc_and_offset(
                    val.and_hms_opt(0, 0, 0)
                        .expect("out of bound should not appear, as we set the time to zero"),
                    Utc,
                )
                .format_localized(&format, locale),
                Err(_) => {
                    return Err(Error::new(
                        ErrorKind::InvalidOperation,
                        format!("Error parsing `{:?}` as YYYY-MM-DD date", value),
                    ));
                }
            }
        }
    } else {
        return Err(Error::new(
            ErrorKind::InvalidOperation,
            "'date' filter received incorrect type for 'value'. Expected i64, or string",
        ));
    };

    Ok(Value::from(datetime.to_string()))
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
