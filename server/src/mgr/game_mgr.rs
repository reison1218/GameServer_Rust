use super::*;
use crate::entity::user::UserData;
use crate::entity::user_info::{create_room, join_room, modify_nick_name, search_room, summary};
use crate::entity::EntityData;
use crate::SEASON;
use chrono::Local;
use protobuf::Message;
use std::str::FromStr;
use tools::cmd_code::{ClientCode, RoomCode};
use tools::protos::protocol::{C_SYNC_DATA, S_SYNC_DATA};
use tools::protos::server_protocol::UPDATE_SEASON_NOTICE;
use tools::tcp::TcpSender;

///gameMgr结构体
pub struct GameMgr {
    pub users: HashMap<u32, UserData>, //玩家数据
    sender: Option<TcpSender>,         //tcpchannel
    pub cmd_map: HashMap<u32, fn(&mut GameMgr, Packet) -> anyhow::Result<()>, RandomState>, //命令管理
}

impl GameMgr {
    ///创建gamemgr结构体
    pub fn new() -> GameMgr {
        let users: HashMap<u32, UserData> = HashMap::new();
        let mut gm = GameMgr {
            users,
            sender: None,
            cmd_map: HashMap::new(),
        };
        //初始化命令
        gm.cmd_init();
        gm
    }

    pub fn set_sender(&mut self, sender: TcpSender) {
        self.sender = Some(sender);
    }

    pub fn get_sender_mut(&mut self) -> &mut TcpSender {
        self.sender.as_mut().unwrap()
    }

    pub fn send_2_client(&mut self, cmd: ClientCode, user_id: u32, bytes: Vec<u8>) {
        let bytes = Packet::build_packet_bytes(cmd as u32, user_id, bytes, true, true);
        self.get_sender_mut().write(bytes);
    }

    pub fn send_2_room(&mut self, cmd: RoomCode, user_id: u32, bytes: Vec<u8>) {
        let bytes = Packet::build_packet_bytes(cmd as u32, user_id, bytes, true, false);
        self.get_sender_mut().write(bytes);
    }

    pub fn save_user_http(&mut self) {
        let time = std::time::SystemTime::now();
        let mut count: u32 = 0;
        for (_, v) in self.users.iter_mut() {
            if v.get_version() <= 0 {
                continue;
            }
            v.update();
            count += 1;
        }
        info!(
            "玩家数据保存结束，保存个数:{},耗时：{}ms",
            count,
            time.elapsed().unwrap().as_millis()
        );
    }

    ///保存玩家数据
    pub fn save_user(&mut self, sender: crossbeam::Sender<Vec<Box<dyn EntityData>>>) {
        let time = std::time::SystemTime::now();
        let mut v: Vec<Box<dyn EntityData>> = Vec::new();
        for ud in self.users.values_mut() {
            if ud.get_version() <= 0 {
                continue;
            };
            //装玩家
            v.push(ud.get_user_info_ref().try_clone());
            //装角色
            let c_v = ud.get_characters_mut_ref().get_need_update_array();
            for i in c_v {
                v.push(i);
            }
            //由于这里是深拷贝，所以在这里提前清空版本号，不然在接收方那边执行update，清空的版本号也是clone的
            ud.clear_version();
        }
        let count = v.len();
        if count > 0 {
            let res = sender.send(v);
            match res {
                Err(e) => {
                    error!("{:?}", e.to_string());
                }
                _ => {}
            }
        }
        info!(
            "开始执行定时保存玩家，发送数量:{},耗时:{}ms",
            count,
            time.elapsed().unwrap().as_millis()
        );
    }

    ///执行函数，通过packet拿到cmd，然后从cmdmap拿到函数指针调用
    pub fn invok(&mut self, packet: Packet) -> anyhow::Result<()> {
        let cmd = packet.get_cmd();
        let f = self.cmd_map.get_mut(&cmd);
        if f.is_none() {
            anyhow::bail!("there is no cmd:{}", cmd)
        }
        f.unwrap()(self, packet)
    }

    ///命令初始化
    fn cmd_init(&mut self) {
        self.cmd_map.insert(UpdateSeason as u32, update_season);
        self.cmd_map.insert(ReloadTemps as u32, reload_temps);
        self.cmd_map.insert(SyncData as u32, sync);
        self.cmd_map.insert(LineOff as u32, off_line);
        self.cmd_map.insert(ModifyNickName as u32, modify_nick_name);
        self.cmd_map.insert(CreateRoom as u32, create_room);
        self.cmd_map.insert(JoinRoom as u32, join_room);
        self.cmd_map.insert(SearchRoom as u32, search_room);
        self.cmd_map.insert(Summary as u32, summary);
    }
}

///热更新配置文件
pub fn reload_temps(_: &mut GameMgr, _: Packet) -> anyhow::Result<()> {
    Ok(())
}

///更新赛季
pub fn update_season(_: &mut GameMgr, packet: Packet) -> anyhow::Result<()> {
    let mut usn = UPDATE_SEASON_NOTICE::new();
    let res = usn.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return Ok(());
    }
    unsafe {
        SEASON.season_id = usn.get_season_id();
        let str = usn.get_last_update_time();
        let last_update_time = chrono::NaiveDateTime::from_str(str).unwrap().timestamp();
        let str = usn.get_next_update_time();
        let next_update_time = chrono::NaiveDateTime::from_str(str).unwrap().timestamp();
        SEASON.last_update_time = last_update_time as u64;
        SEASON.next_update_time = next_update_time as u64;
    }
    Ok(())
}

///同步数据
fn sync(gm: &mut GameMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let user = gm.users.get_mut(&user_id);
    if user.is_none() {
        let str = format!("user data is null for id:{}", user_id);
        error!("{:?}", str.as_str());
        anyhow::bail!(str);
    }
    let user = user.unwrap();

    let mut csd = C_SYNC_DATA::new();
    let res = csd.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        let str = format!(
            "protobuf:C_SYNC_DATA parse has error!cmd:{},err_mess:{:?}",
            packet.get_cmd(),
            e.to_string()
        );
        error!("{:?}", str.as_str());
        anyhow::bail!(str);
    }

    if csd.player_pt.is_some() {
        let pp = csd.player_pt.unwrap();
        let nick_name = user.get_user_info_mut_ref().get_nick_name();
        if pp.get_nick_name() != nick_name {
            user.get_user_info_mut_ref()
                .set_nick_name(pp.get_nick_name());
        }
        user.get_user_info_mut_ref().set_dlc(pp.dlc);
    }

    let mut s_s_d = S_SYNC_DATA::new();
    s_s_d.is_succ = true;
    s_s_d.sync_time = Local::now().timestamp() as u32;
    gm.send_2_client(
        ClientCode::SyncData,
        user_id,
        s_s_d.write_to_bytes().unwrap(),
    );
    info!("执行同步函数");
    Ok(())
}

///玩家离线
fn off_line(gm: &mut GameMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let user = gm.users.remove(&user_id);
    if let Some(mut user_data) = user {
        user_data.update_off();
        info!("游戏服已处理玩家离线 for id:{}", user_data.get_user_id());
    }
    Ok(())
}
