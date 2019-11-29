use super::*;

use std::io::{Result, Write};
use crate::protos::base::MessPacketPt;
use protobuf::Message;
use std::rc::Rc;
use std::sync::Arc;
use futures::executor::block_on;
use crate::net::bytebuf::ByteBuf;

pub struct ChannelMgr {
    game_channel: TcpStream,
    pub players: HashMap<u32, GateUser>,
}

impl ChannelMgr {

    ///创建channelmgr结构体
    pub async fn new() -> ChannelMgr {
        let mut game_channel = new_tcp_client();
        let mut players: HashMap<u32, GateUser> = HashMap::new();
        let mut cm = ChannelMgr {
            game_channel: game_channel,
            players: players,
        };
        cm
    }

    ///连接游戏服
    pub fn connect_game(&mut self) {

        let mut v: [u8; 1024] = [0; 1024];
        info!("连接GameServer成功！");
        loop{

            let size = self.game_channel.read(&mut v);
            if size.unwrap() ==0{
                continue;
            }

            let mut  bb = ByteBuf::new();
            bb.push_array(&v);
            let mut packet = Packet::from(bb);

            //判断是否是发给客户端消息
            if packet.is_client(){
                let mut gate_user = self.players.get_mut(&packet.get_user_id().unwrap());
                if gate_user.is_none(){
                    error!("user data is null,id:{}",&packet.get_user_id().unwrap());
                    continue;
                }
                let mut mess = build_Mess(packet);
                let bytes = &mess.write_to_bytes().unwrap()[..];
                gate_user.unwrap().ws.send(bytes);
            }else{//判断是否要转发到其他服务器进程消息

            }
        };
    }



    pub fn connect_room(&mut self) {}

    ///写到游戏服
    pub fn write_to_game(&mut self,packet:Packet){
        self.game_channel.write(packet.get_data());
        self.game_channel.flush();
    }

    ///写到房间服
    pub fn write_to_room(&mut self,packet:Packet){

    }
}

pub fn build_Mess(packet:Packet)->MessPacketPt{
    let mut mess = MessPacketPt::new();
    mess.cmd = packet.get_cmd();
    let v = Vec::from(packet.get_data());
    let len = v.len();
    mess.data = v;
    mess.len = (4+len) as u32;
    mess
}



///新建tpc连接客户端
pub fn new_tcp_client() -> TcpStream {
    let mut ts:Option<Result<TcpStream>> = None;
    let mut result:Option<TcpStream> = None;
    let dur = Duration::from_secs(2);
    loop{
        ts = Some(connect());
        let re = ts.unwrap();
        if re.is_err(){
            error!("连接游戏服失败！{}",re.err().unwrap().to_string());
            //睡2s
            std::thread::sleep(dur);
            continue;
        }
        result = Some(re.unwrap());
        break;
    }

    //设置非阻塞
    let mut result =  result.unwrap();
    //result.set_nonblocking(true);
    //不组包
    result.set_nodelay(true);
    result
}

///tcp连接
fn connect()->Result<TcpStream>{
    let mut ts = TcpStream::connect("127.0.0.1:8888");
    ts
}
