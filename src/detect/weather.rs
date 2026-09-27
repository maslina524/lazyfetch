use core::str::{FromStr, SplitN};

use alloc::{
    string::String,
    borrow::ToOwned
};

use crate::{
    format, 
    formats::{Percent, Temperature}, 
    imp::{
        env, 
        fs::{self, Access, File}, 
        http::Request, 
        path::Path
    }, 
    superstr::ConcatStr, 
    modules::weather::Weather, 
    warning
};

const WTTR_URL: &str = "http://wttr.in/?lang=en&format=%c;%C;%x;%h;%t;%f;%w;%l;%m;%M;%p;%P;%u;%D;%S;%z;%s;%d;%T;%Z";

pub fn get() -> Weather {
    let cur_hour = env::timestamp_hours();
    let raw_string = if let Some((hours, data)) = read_cache() {
        if cur_hour == hours {
            data
        } else {
            request().map_or_else(String::new, |data| {
                set_cache(cur_hour, &data);
                data
            })
        }
    } else {
        request().map_or_else(String::new, |data| {
            set_cache(cur_hour, &data);
            data
        })
    };
    if raw_string.is_empty() {
        return Weather::default();
    }
    let raw: &'static str = String::leak(raw_string);
    let mut parts = raw.splitn(20, ';');
    let condition_emoji = parts.next().unwrap_or("");
    let condition = parts.next().unwrap_or("");
    let condition_symbol = parts.next().unwrap_or("");
    let humidity = parts.next().unwrap_or("");
    let temperature_actual = parts.next().unwrap_or("");
    let temperature_feels = parts.next().unwrap_or("");
    let wind = parts.next().unwrap_or("");
    let location = parts.next().unwrap_or("");
    let moon_emoji = parts.next().unwrap_or("");
    let moon_day = parts.next().unwrap_or("");
    let precipitation = parts.next().unwrap_or("");
    let pressure = parts.next().unwrap_or("");
    let uv_index = parts.next().unwrap_or("");
    let dawn = parts.next().unwrap_or("");
    let sunrise = parts.next().unwrap_or("");
    let zenith = parts.next().unwrap_or("");
    let sunset = parts.next().unwrap_or("");
    let dusk = parts.next().unwrap_or("");
    let time = parts.next().unwrap_or("");
    let timezone = parts.next().unwrap_or("");

    Weather {
        result: ConcatStr::new([temperature_actual, " — ", condition, " (", location, ")"]),
        condition_emoji,
        condition, 
        condition_symbol, 
        humidity: Percent::from_str(humidity).expect("Unreachable"), 
        temperature_actual: Temperature::from_str(temperature_actual).expect("Unreachable"), 
        temperature_feels: Temperature::from_str(temperature_feels).expect("Unreachable"), 
        wind, 
        location, 
        moon_emoji, 
        moon_day: moon_day
            .parse::<u8>()
            .expect("Strange response from wttr.is"), 
        precipitation, 
        pressure, 
        uv_index: uv_index
            .parse::<u8>()
            .expect("Strange response from wttr.is"), 
        dawn, 
        sunrise, 
        zenith, 
        sunset, 
        dusk, 
        time, 
        timezone
    }
}

fn request() -> Option<String> {
    let response = match Request::new(WTTR_URL).unwrap().get() {
        Ok(r) => r,
        Err(e) => {
            warning!("Failed to connect to server (weather): {}", e.code());
            return None
        }
    };
    if response.is_success() {
        Some(response.as_text().unwrap())
    } else {
        warning!("Wttr.in (weather) response code: {}", response.code());
        None
    }
}

fn read_cache() -> Option<(u64, String)> {
    fn next_item<'iter>(parts: &mut SplitN<'iter, char>) -> Option<&'iter str> {
        parts.next().map_or_else(|| {
            warning!("Strange response from ip-api.com");
            None
        }, Some)
    }

    let path = Path::cache().join("weather");
    let string = fs::read_to_string(path).ok()?;

    let mut parts = string.splitn(2, '\n');
    let hours = next_item(&mut parts)?.parse::<u64>().ok()?;
    let data = next_item(&mut parts)?.to_owned();

    Some((hours, data))
}

fn set_cache(hour: u64, data: &str) -> Option<()> {
    let path_dir = Path::cache();
    if let Err(e) = fs::create_dirs(&path_dir) {
        warning!("Failed to create {path_dir}: {e}");
    }
    let path = path_dir.join("weather");
    let file = File::create_always(path, Access::Write).ok()?;
    let formatted = format!("{hour}\n{data}");
    file.write(formatted).ok()?;
    Some(())
}