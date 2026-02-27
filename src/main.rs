use netroute::RouteFamily::Ipv4;
use serde::Deserialize;
use std::io;

#[derive(Debug, Deserialize, Clone)]
struct BasicInfo {
    brand: String,
    model: String,
    sn: String,
    mac: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Resp<T> {
    code: i32,
    message: String,
    data: T,
}

#[derive(Debug, Deserialize)]
struct RespItem<T> {
    id: i32,
    result: Resp<T>,
}

#[derive(Debug)]
#[allow(dead_code)]
enum ApiError {
    Network(String),
    Status(i32, String),
    Parse(String),
}

fn call_api<T, E>(ip: &str, path: &str) -> Result<T, E>
where
    T: for<'de> Deserialize<'de> + Clone,
    E: From<ApiError>,
{
    let url = format!("http://{}{}", ip, path);
    eprintln!("\nGET {}", url);
    let response = minreq::get(&url)
        .with_timeout(1)
        .send()
        .map_err(|e| ApiError::Network(e.to_string()))?;
    if response.status_code != 200 {
        return Err(ApiError::Status(
            response.status_code,
            response.reason_phrase,
        )
        .into());
    }
    let contents = response
        .as_str()
        .map_err(|e| ApiError::Parse(e.to_string()))?;
    eprintln!("{}", contents);
    let resp = match serde_json::from_str::<Vec<RespItem<T>>>(contents) {
        Ok(resp_vec) => resp_vec
            .get(0)
            .ok_or(ApiError::Parse("Empty JSON array".to_owned()))?
            .result
            .clone(),
        Err(_) => serde_json::from_str::<Resp<T>>(contents)
            .map_err(|e| ApiError::Parse(e.to_string()))?,
    };
    if resp.code != 0 {
        return Err(ApiError::Status(resp.code, resp.message).into());
    }
    Ok(resp.data)
}

fn get_basic_info(host: &str) -> Result<BasicInfo, ApiError> {
    call_api(host, "/api/wizard/getBasicInfo")
}

fn enable_telnet(
    host: &str,
    info: &BasicInfo,
    password: &str,
) -> Result<serde_json::Value, ApiError> {
    let md5_input = format!(
        "sn={}--ethaddr={}--usrpwd={}\n--H3C.MAGIC.ZH",
        info.sn.to_lowercase(),
        info.mac.to_lowercase(),
        password,
    );
    let md5_bytes = md5::compute(md5_input.as_bytes());
    let md5_hex = format!("{:x}", md5_bytes);
    call_api(host, &format!("/api/debug?status=enable{}", md5_hex))
}

fn list_possible_gateways() -> Vec<String> {
    let mut hosts = Vec::new();

    // Try to detect gateway from default routes
    if let Ok(routes) = netroute::list_routes() {
        for route in routes {
            if route.family != Ipv4 || route.destination.prefix_len > 0 {
                continue;
            }
            let Some(gw) = route.gateway else {
                continue;
            };
            let gateway = gw.to_string();
            if !hosts.contains(&gateway) {
                hosts.push(gateway);
            }
        }
    }

    for fallback in ["192.168.124.1", "moshujia.cn"] {
        if !hosts.iter().any(|s| s == fallback) {
            hosts.push(fallback.to_owned());
        }
    }

    hosts
}

fn get_available_gateway(hosts: Vec<String>) -> Option<(String, BasicInfo)> {
    for host in &hosts {
        match get_basic_info(host) {
            Ok(info) => {
                return Some((host.clone(), info));
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }
    None
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    try_gateways(if args.len() > 1 {
        args[1..].to_vec()
    } else {
        list_possible_gateways()
    });
}

fn try_gateways(gateways: Vec<String>) {
    // Read password from stdin
    if let Some((host, info)) = get_available_gateway(gateways) {
        while let Err(e) = enable_telnet(
            &host,
            &info,
            &prompt(&format!("Password for {}:", host)),
        ) {
            eprintln!("{:?}", e);
        }
    } else {
        eprintln!("\n{:?}", ApiError::Network("No gateway responded".into()));
        return try_gateways(vec![prompt("Specify the IP manually:")]);
    }

    prompt("\nPress ENTER to exit...");
}

fn prompt(msg: &str) -> String {
    print!("{} ", msg);
    io::Write::flush(&mut io::stdout()).ok();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).ok();
    buf.trim_end().to_owned()
}
