use netroute::RouteFamily::Ipv4;
use serde::Deserialize;
use std::io;

#[derive(Debug, Deserialize)]
struct BasicInfo {
    brand: String,
    model: String,
    sn: String,
    mac: String,
}

#[derive(Debug, Deserialize)]
struct Resp<T> {
    code: i32,
    message: String,
    data: T,
}

#[derive(Debug)]
enum ApiError {
    Network(String),
    Status(i32, String),
    Parse(String),
}

fn call_api<T, E>(ip: &str, path: &str) -> Result<T, E>
where
    T: for<'de> Deserialize<'de>,
    E: From<ApiError>,
{
    let url = format!("http://{}{}", ip, path);
    eprintln!("\nGET {}", url);
    let response = minreq::get(&url)
        .with_timeout(1)
        .send()
        .map_err(|e| ApiError::Network(e.to_string()))?;
    if response.status_code != 200 {
        return Err(ApiError::Status(response.status_code, response.reason_phrase).into());
    }
    let contents = response
        .as_str()
        .map_err(|e| ApiError::Parse(e.to_string()))?;
    eprintln!("{}", contents);
    let resp: Resp<T> =
        serde_json::from_str(contents).map_err(|e| ApiError::Parse(e.to_string()))?;
    if resp.code != 0 {
        return Err(ApiError::Status(resp.code, resp.message).into());
    }
    Ok(resp.data)
}

fn get_basic_info(host: &str) -> Result<BasicInfo, ApiError> {
    call_api(host, "/api/wizard/getBasicInfo")
}

fn enable_telnet(ip: &str, md5: &str) -> Result<serde_json::Value, ApiError> {
    let path = format!("/api/debug?status=enable{}", md5);
    call_api(ip, &path)
}

fn get_gateways() -> Vec<String> {
    let mut hosts = Vec::new();

    // Try to detect gateway from default routes
    if let Ok(routes) = netroute::list_routes() {
        for route in routes {
            if route.family != Ipv4 || route.destination.prefix_length > 0 {
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
        if !hosts.contains(&fallback.to_string()) {
            hosts.push(fallback.to_string());
        }
    }

    hosts
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Find gateways
    let hosts: Vec<String> = if args.len() > 1 {
        vec![args[1].clone()]
    } else {
        get_gateways()
    };

    // Try gateways in turn, and get hardware info
    let mut found: Option<(String, BasicInfo)> = None;
    for host in &hosts {
        match get_basic_info(host) {
            Ok(data) => {
                found = Some((host.clone(), data));
                break;
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }
    let (host, info) = found.expect("no gateway responded");

    // Read password from stdin
    let md5_input = format!(
        "sn={}--ethaddr={}--usrpwd={}\n--H3C.MAGIC.ZH",
        info.sn.to_lowercase(),
        info.mac.to_lowercase(),
        &prompt("Password: "),
    );

    let md5_bytes = md5::compute(md5_input.as_bytes());
    let md5_hex = format!("{:x}", md5_bytes);
    enable_telnet(&host, &md5_hex);

    prompt("\nPress ENTER to exit...");
}

fn prompt(msg: &str) -> String {
    print!("{}", msg);
    io::Write::flush(&mut io::stdout()).ok();
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).ok();
    buf.trim_end().to_string()
}
