use core::str::SplitN;

use alloc::string::String;

use crate::{
    format, imp::{
        env,
        http::Request,
        path::Path,
        fs::{self, File, Access}
    }, json::{Json, Map}, modules::publicip::PublicIP, warning
};

const PUBLICIP_URL: &str = "http://ip-api.com/json/";

pub fn get() -> PublicIP {
    let cur_hour = env::timestamp_hours();
    let json: Map = if let Some((hours, data)) = read_cache() {
        if cur_hour == hours {
            data
        } else {
            request().map_or_else(Map::default, |data| {
                set_cache(cur_hour, &data);
                data
            })
        }
    } else {
        request().map_or_else(Map::default, |data| {
            set_cache(cur_hour, &data);
            data
        })
    };

    let ip = json.get_string("query").cloned().unwrap_or_else(|| {
        warning!("Failed to get query (ip) from response");
        String::new()
    });
    let city = json.get_string("city").cloned().unwrap_or_else(|| {
        warning!("Failed to get city from response");
        String::new()
    });
    let country_code = json.get_string("countryCode").cloned().unwrap_or_else(|| {
        warning!("Failed to get countryCode from response");
        String::new()
    });

    PublicIP { ip, location: format!("{city}, {country_code}") }
}

fn request() -> Option<Map> {
    let response = match Request::new(PUBLICIP_URL).unwrap().get() {
        Ok(r) => r,
        Err(e) => {
            warning!("Failed to connect to server (publicip): {}", e.code());
            return None
        }
    };
    if response.is_success() {
        let text = response.as_text().unwrap();
        match Json::from_str(&text) {
            Ok(c) => Some(c),
            Err(e) => {
                warning!("Failed to parse response (publicip): {e}");
                None
            }
        }
    } else {
        warning!("Ip-api.com (publicip) response code: {}", response.code());
        None
    }
}

fn read_cache() -> Option<(u64, Map)> {
    fn next_item<'iter>(parts: &mut SplitN<'iter, char>) -> Option<&'iter str> {
        parts.next().map_or_else(|| {
            warning!("Strange response from ip-api.com");
            None
        }, Some)
    }

    let path = Path::cache().join("publicip");
    let string = fs::read_to_string(path).ok()?;

    let mut parts = string.splitn(2, '\n');
    let hours = next_item(&mut parts)?.parse::<u64>().ok()?;
    let raw = next_item(&mut parts)?;

    match Json::from_str(raw) {
        Ok(m) => Some((hours, m)),
        Err(e) => {
            warning!("Failed to parse cache (publicip): {e}");
            None
        }
    }
}

fn set_cache(hour: u64, map: &Map) -> Option<()> {
    let path_dir = Path::cache();
    if let Err(e) = fs::create_dirs(&path_dir) {
        warning!("Failed to create {path_dir}: {e}");
    }

    let path = path_dir.join("publicip");
    let file = File::create_always(path, Access::Write).ok()?;
    let formatted = format!("{hour}\n{map}");
    file.write(formatted.as_bytes()).ok()?;

    Some(())
}