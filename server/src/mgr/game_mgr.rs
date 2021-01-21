use crate::entity::user::UserData;
use crate::entity::user_info::{
    create_room, join_room, modify_nick_name, punish_match, search_room, show_rank, summary,
    sync_rank,
};
use crate::entity::{Entity, EntityData};
use crate::SEASON;
use chrono::Local;
use log::{error, info, warn};
use protobuf::Message;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::str::FromStr;
use tools::cmd_code::{ClientCode, GameCode, ServerCommonCode};
use tools::cmd_code::{GameCode::SyncData, RankCode};
use tools::protos::base::RankInfoPt;
use tools::protos::protocol::{C_SYNC_DATA, S_SYNC_DATA};
use tools::protos::server_protocol::UPDATE_SEASON_NOTICE;
use tools::tcp::TcpSender;
use tools::util::packet::Packet;

///gameMgr结构体
pub struct GameMgr {
    pub users: HashMap<u32, UserData>, //玩家数据
    pub rank: Vec<RankInfoPt>,         //排行榜快照，从排行榜服务器那边过来的
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
            rank: Vec::new(),
            cmd_map: HashMap::new(),
        };
        //初始化命令
        gm.cmd_init();
        gm
    }

    ///初始化排行榜
    pub fn init_rank(&mut self) {
        self.send_2_server(RankCode::GetRank.into_u32(), 0, Vec::new());
    }

    pub fn set_sender(&mut self, sender: TcpSender) {
        self.sender = Some(sender);
    }

    pub fn get_sender_mut(&mut self) -> &mut TcpSender {
        self.sender.as_mut().unwrap()
    }

    pub fn send_2_client(&mut self, cmd: ClientCode, user_id: u32, bytes: Vec<u8>) {
        let bytes = Packet::build_packet_bytes(cmd as u32, user_id, bytes, true, true);
        self.get_sender_mut().send(bytes);
    }

    pub fn send_2_server(&mut self, cmd: u32, user_id: u32, bytes: Vec<u8>) {
        let bytes = Packet::build_packet_bytes(cmd as u32, user_id, bytes, true, false);
        self.get_sender_mut().send(bytes);
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
    pub fn save_user(&mut self, sender: crossbeam::channel::Sender<Vec<Box<dyn EntityData>>>) {
        let time = std::time::SystemTime::now();
        let mut v: Vec<Box<dyn EntityData>> = Vec::new();
        for ud in self.users.values_mut() {
            if ud.get_version() <= 0 {
                continue;
            };
            //装玩家
            if ud.get_user_info_ref().get_version() > 0 {
                v.push(ud.get_user_info_ref().try_clone_for_db());
            }
            //装角色
            let c_v = ud.get_characters_mut_ref().get_need_update_array();
            for i in c_v {
                v.push(i);
            }
            //装段位数据
            if ud.get_league_ref().get_version() > 0 {
                v.push(ud.get_league_mut_ref().try_clone_for_db());
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
        self.cmd_map
            .insert(ServerCommonCode::UpdateSeason.into_u32(), update_season);
        self.cmd_map
            .insert(ServerCommonCode::ReloadTemps.into_u32(), reload_temps);
        self.cmd_map.insert(SyncData.into_u32(), sync);
        self.cmd_map
            .insert(GameCode::UnloadUser.into_u32(), off_line);
        self.cmd_map
            .insert(GameCode::ModifyNickName.into_u32(), modify_nick_name);
        self.cmd_map
            .insert(GameCode::CreateRoom.into_u32(), create_room);
        self.cmd_map
            .insert(GameCode::JoinRoom.into_u32(), join_room);
        self.cmd_map
            .insert(GameCode::SearchRoom.into(), search_room);
        self.cmd_map
            .insert(GameCode::SyncPunish.into_u32(), punish_match);
        self.cmd_map
            .insert(GameCode::SyncRank.into_u32(), sync_rank);
        self.cmd_map
            .insert(GameCode::ShowRank.into_u32(), show_rank);
        self.cmd_map.insert(GameCode::Summary.into_u32(), summary);
    }
}

///热更新配置文件
pub fn reload_temps(_: &mut GameMgr, _: Packet) -> anyhow::Result<()> {
    let path = std::env::current_dir();
    if let Err(e) = path {
        anyhow::bail!("{:?}", e)
    }
    let path = path.unwrap();
    let str = path.as_os_str().to_str();
    if let None = str {
        anyhow::bail!("reload_temps can not path to_str!")
    }
    let str = str.unwrap();
    let res = str.to_string() + "/template";
    crate::TEMPLATES.reload_temps(res.as_str())?;
    info!("reload_temps success!");
    Ok(())
}

///更新赛季
pub fn update_season(gm: &mut GameMgr, packet: Packet) -> anyhow::Result<()> {
    let mut usn = UPDATE_SEASON_NOTICE::new();
    let res = usn.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return Ok(());
    }
    let season_id = usn.get_season_id();
    let next_update_time = usn.get_next_update_time();
    unsafe {
        SEASON.season_id = season_id;
        SEASON.next_update_time = next_update_time;
    }
    //处理更新内存
    let mgr = crate::TEMPLATES.get_constant_temp_mgr_ref();
    let round_season_id = mgr.temps.get("round_season_id");
    if let None = round_season_id {
        warn!("the constant temp is None!key:round_season_id");
        return Ok(());
    }
    let round_season_id = round_season_id.unwrap();
    let res = u32::from_str(round_season_id.value.as_str());
    if let Err(e) = res {
        error!("{:?}", e);
        return Ok(());
    }
    let round_season_id = res.unwrap();
    if round_season_id != season_id {
        return Ok(());
    }
    //更新所有内存数据
    for user in gm.users.values_mut() {
        user.league.round_reset();
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
