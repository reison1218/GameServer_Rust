use std::str::FromStr;

//use isahc::prelude::*;

use crate::{net::query_user_id_from_redis, CONF_MAP};

pub fn auth_account(ticket: &str) -> anyhow::Result<u32> {
    let steam_id = auth_user_ticket(ticket)?;
    let user_id = query_user_id_from_redis(steam_id.to_string().as_str())?;
    Ok(user_id)
}

pub fn auth_user_ticket(ticket: &str) -> anyhow::Result<u64> {
    let res = CONF_MAP.conf.get(&"steam".to_owned());
    if let None = res {
        anyhow::bail!("steam auth fail!steam key is not find!")
    }

    let json_value = res.unwrap();

    let res = json_value.as_array().unwrap().get(0).unwrap();
    let json_value = res.as_object().unwrap();

    let web_api_key = json_value.get("web_api_key").unwrap();
    let web_api_key = web_api_key.as_str().unwrap();

    let app_id = json_value.get("player_test_app_id").unwrap();
    let app_id = app_id.as_u64().unwrap();

    let url = format!("https://partner.steam-api.com/ISteamUserAuth/AuthenticateUserTicket/v1/?key={:?}&appid={}&ticket={:?}",web_api_key,app_id,ticket);
    let url = url.replace(r#"""#, "");

    // let res = isahc::get(url);
    // if let Err(e) = res {
    //     anyhow::bail!("{:?}", e)
    // }
    // let mut res = res.unwrap();
    let res = String::new();
    // let res = res.text().unwrap();

    let json = serde_json::Value::from_str(res.as_str());

    if let Err(e) = json {
        anyhow::bail!("{:?}", e)
    }

    let res = json.unwrap();

    let res = res.as_object();
    if let None = res {
        anyhow::bail!("auth fail!response is None!ticket:{:?}", ticket)
    }
    let res = res.unwrap();

    let res = res.get("response");
    if let None = res {
        anyhow::bail!("auth fail!response is None!ticket:{:?}", ticket)
    }
    let res = res.unwrap();

    let res = res.as_object();
    if let None = res {
        anyhow::bail!("auth fail!response is None!ticket:{:?}", ticket)
    }

    let res = res.unwrap();

    let json_res = res.get("params");

    if let None = json_res {
        anyhow::bail!("auth fail!params is None!ticket:{:?}", ticket)
    }

    let json_res = json_res.unwrap();

    let json_res = json_res.as_object();
    if let None = json_res {
        anyhow::bail!("auth fail!params is None!ticket:{:?}", ticket)
    }
    let json_res = json_res.unwrap();

    let result = json_res.get("result");

    if let None = result {
        anyhow::bail!("auth fail!result is None!ticket:{:?}", ticket)
    }

    let result = result.unwrap();
    let result = result.as_str().unwrap();

    if !result.eq("OK") {
        anyhow::bail!("auth fail!result is not OK!ticket:{:?}", ticket)
    }

    let steam_id = json_res.get("steamid");
    if let None = steam_id {
        anyhow::bail!("auth fail!steam_id is None!ticket:{:?}", ticket)
    }
    let steam_id = steam_id.unwrap();

    let steam_id = steam_id.as_str();
    if let None = steam_id {
        anyhow::bail!("auth fail!steam_id is None!ticket:{:?}", ticket)
    }
    let steam_id = steam_id.unwrap();

    let steam_id = u64::from_str(steam_id);
    if let Err(e) = steam_id {
        anyhow::bail!("{:?}", e)
    }
    let steam_id = steam_id.unwrap();

    //检测是否拥有app
    let res = check_app_owner_ship(steam_id, web_api_key, app_id);
    if let Err(e) = res {
        anyhow::bail!("{:?}", e);
    }
    Ok(steam_id)
}

pub fn check_app_owner_ship(steam_id: u64, web_api_key: &str, app_id: u64) -> anyhow::Result<()> {
    let url = format!("https://partner.steam-api.com/ISteamUser/CheckAppOwnership/v2/?key={:?}&appid={}&steamid={:?}",web_api_key,app_id,steam_id);
    let url = url.replace(r#"""#, "");

    // let res = isahc::get(url);
    // if let Err(e) = res {
    //     anyhow::bail!("{:?}", e)
    // }
    // let mut res = res.unwrap();

    // let res = res.text().unwrap();
    let res = String::new();

    let json = serde_json::Value::from_str(res.as_str());

    if let Err(e) = json {
        anyhow::bail!("{:?}", e)
    }

    let json = json.unwrap();

    let json = json.as_object();
    if let None = json {
        anyhow::bail!("auth fail!appownership is None!steam_id:{:?}", steam_id)
    }

    let json = json.unwrap();

    let appownership = json.get("appownership");
    if let None = appownership {
        anyhow::bail!("auth fail!appownership is None!steam_id:{:?}", steam_id)
    }

    let appownership = appownership.unwrap();

    let map = appownership.as_object();
    if let None = map {
        anyhow::bail!("auth fail!appownership is None!steam_id:{:?}", steam_id)
    }

    let map = map.unwrap();

    let ownsapp = map.get("ownsapp");
    if let None = ownsapp {
        anyhow::bail!("auth fail!ownsapp is None!steam_id:{:?}", steam_id)
    }

    let ownsapp = ownsapp.unwrap();

    let ownsapp = ownsapp.as_bool();

    if let None = ownsapp {
        anyhow::bail!("auth fail!ownsapp is None!steam_id:{:?}", steam_id)
    }

    let ownsapp = ownsapp.unwrap();
    if !ownsapp {
        anyhow::bail!("auth fail!ownsapp is false!steam_id:{:?}", steam_id)
    }
    Ok(())
}
