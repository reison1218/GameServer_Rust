use tools::thread_pool::{MyThreadPool, ThreadPoolHandler};
use ws::{
    connect, Builder, CloseCode, Factory, Handler, Handshake, Message as WMessage, Request,
    Response, Result, Sender as WsSender, Settings, WebSocket,
};
use std::time::Duration;
use protobuf::Message;
use tools::cmd_code::GameCode;
use tools::util::packet::Packet;

pub fn test_websocket() {
    let mtp = MyThreadPool::init("test".to_owned(), 12, "test1".to_owned(), 8, "test2".to_owned(), 2);
    for id in 1..10 {
        println!("{}",id);
        let d = Duration::from_millis(2000);
        std::thread::sleep(d);
        let m = move|| {
            let result = connect("ws://127.0.0.1:16801", |out| {
                // Queue a message to be sent when the WebSocket is open
                let mut packet = Packet::default();
                packet.set_cmd(GameCode::Login as u32);

                let mut s_l = tools::protos::protocol::C_USER_LOGIN::new();
                // s_l.set_avatar("test".to_owned());
                // s_l.set_nickName("test".to_owned());
                s_l.set_user_id(1011000002 as u32);
                packet.set_data_from_vec(s_l.write_to_bytes().unwrap());
                packet.set_len(16+packet.get_data().len() as u32);
                out.send(&packet.build_client_bytes()[..]).unwrap();

                // The handler needs to take ownership of out, so we use move
                move |msg| {
                    // Handle messages received on this connection
                    println!("Client got message '{}'. ", msg);

                    // Close the connection
                    //out.close(CloseCode::Normal)
                    Ok(())
                }
            });
            if result.is_err(){
                // Inform the user of failure
                println!("Failed to create WebSocket due to: {:?}", result.err().unwrap());
            };
        };
        mtp.submit_game(m);
    };

}