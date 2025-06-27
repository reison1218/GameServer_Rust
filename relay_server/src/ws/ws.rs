use futures::{SinkExt, StreamExt};
use log::{error, info};
use std::error::Error;
use std::str::FromStr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tools::json::{JsonValue, JsonValueTrait};

pub async fn handle_port(port: u16) -> Result<(), Box<dyn Error>> {
    // 监听指定端口
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("Ws-Server Listening on port {}", port);

    // 接受连接并处理
    while let Ok((client_stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            let target_ip = crate::CONF_MAP.get_str("http_ip", "192.168.110.130");
            // 目标服务器地址（根据你的需求修改）
            let target_addr = format!("{}:{}", target_ip, port);

            if let Err(e) = proxy_connection(client_stream, target_addr.as_str()).await {
                error!("Error in connection on port {}: {}", port, e);
            }
        });
    }

    Ok(())
}

pub async fn proxy_connection(
    client_stream: TcpStream,
    target_addr: &str,
) -> Result<(), Box<dyn Error>> {
    // 升级为WebSocket连接
    let client_ws = accept_async(client_stream).await?;
    info!(
        "Client WebSocket connected {}",
        client_ws.get_ref().peer_addr().unwrap()
    );

    // 连接到目标服务器
    let target_stream = TcpStream::connect(target_addr).await?;
    let target_addr = format!("ws://{}/gateway/", target_addr);
    let (target_ws, _) =
        tokio_tungstenite::client_async(target_addr.as_str(), target_stream).await?;
    let (mut target_tx, mut target_rx) = target_ws.split();
    println!("Connected to target server at {}", target_addr);

    // 分离客户端WebSocket的读写
    let (mut client_tx, mut client_rx) = client_ws.split();

    // 创建两个任务来双向转发消息
    let client_to_target = tokio::spawn(async move {
        while let Some(msg) = client_rx.next().await {
            match msg {
                Ok(msg) => {
                    if let Err(e) = target_tx.send(msg).await {
                        error!("Error forwarding to target: {}   error:{}", e, target_addr);
                        break;
                    }
                }
                Err(e) => {
                    error!("Error receiving from client: {}", e);
                    break;
                }
            }
        }
    });

    let target_to_client = tokio::spawn(async move {
        while let Some(msg) = target_rx.next().await {
            match msg {
                Ok(msg) => {
                    if let Err(e) = client_tx.send(msg).await {
                        error!("Error forwarding to client: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    error!("Error receiving from target: {}", e);
                    break;
                }
            }
        }
    });

    // 等待任一任务完成
    tokio::select! {
        _ = client_to_target => {},
        _ = target_to_client => {},
    };

    Ok(())
}

pub async fn get_zones() -> Option<Vec<(String, u16)>> {
    // tcp_server::tcp_server::tcp_server_build();
    let ip = crate::CONF_MAP.get_str("http_ip", "localhost");
    let url = format!("http://{}:8500/get_json_config/zone.json", ip);
    let res = tools::http::send_get(url.as_str(), None, None);
    if let Err(e) = res {
        log::error!("{}", e);
        return None;
    }
    let res = res.unwrap();
    let res = JsonValue::from_str(res.as_str());
    if let Err(e) = res {
        log::error!("{}", e);
        return None;
    }
    let res = res.unwrap();
    if let None = res.get_object("monkey_config") {
        error!("monkey_config is None!");
        return None;
    }

    let zone_map = res.get_object("monkey_config").unwrap();

    let mut vec = Vec::new();
    for (key, value) in zone_map.iter() {
        let json: &JsonValue = value;
        let json_list = json.as_array().unwrap();
        let ip_port = json_list.get(1).unwrap().as_array().unwrap();
        if ip_port.get(0).unwrap().as_str().unwrap().eq("") {
            continue;
        }
        let _ = key;
        let port = ip_port.get(1).unwrap().as_i64().unwrap();
        let ip = ip_port.get(0).unwrap().as_str().unwrap();
        // let res = format!("{} {} {}", zone_id, ip, port);

        let port = port as u16;
        vec.push((ip.to_string(), port));

        // let runtime = tokio::runtime::Handle::current();
        // tokio::runtime::Runtime::new().unwrap().spawn(async move {
        //     build_websocket_server(port).await;
        // });
    }
    Some(vec)
}
