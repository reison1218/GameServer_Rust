use tools::protos::server_protocol::G_S_MODIFY_NICK_NAME;
use tools::tcp::TcpSender;

use crate::entity::character::Characters;
use crate::entity::grade_frame::GradeFrame;
use crate::entity::soul::Soul;
use crate::entity::user::{
    insert_characters, insert_grade_frame, insert_soul, insert_user, UserData,
};
use crate::entity::user_info::User;
use crate::helper::redis_helper::get_user_from_redis;
use crate::mgr::game_mgr::GameMgr;
use crate::Lock;
use async_std::task::block_on;
use async_trait::async_trait;
use log::{error, info, warn};
use protobuf::Message;
use tools::cmd_code::{ClientCode, GameCode, RankCode, ServerCommonCode};
use tools::protos::protocol::C_USER_LOGIN;
use tools::util::packet::Packet;

#[derive(Clone)]
struct TcpServerHandler {
    gm: Lock,
}

unsafe impl Send for TcpServerHandler {}

unsafe impl Sync for TcpServerHandler {}

#[async_trait]
impl tools::tcp::Handler for TcpServerHandler {
    async fn try_clone(&self) -> Self {
        self.clone()
    }

    async fn on_open(&mut self, sender: TcpSender) {
        let mut lock = self.gm.lock().await;
        lock.set_sender(sender);
    }

    async fn on_close(&mut self) {
        info!("与tcp客户端断开连接");
    }

    async fn on_message(&mut self, mess: Vec<u8>) -> bool {
        let packet_array = Packet::build_array_from_server(mess);

        if let Err(e) = packet_array {
            error!("{:?}", e);
            return true;
        }
        let packet_array = packet_array.unwrap();

        for packet in packet_array {
            let gm = self.gm.clone();
            handler_mess_s(gm, packet).await;
        }
        true
    }
}

async fn handler_mess_s(gm: Lock, packet: Packet) {
    let cmd = packet.get_cmd();
    //如果为空，什么都不执行
    if cmd != GameCode::Login.into_u32()
        && cmd != GameCode::UnloadUser.into_u32()
        && cmd != GameCode::SyncRank.into_u32()
        && cmd != ServerCommonCode::ReloadTemps.into_u32()
        && packet.get_data().is_empty()
    {
        error!("packet bytes is null!cmd:{}", packet.get_cmd());
        return;
    }
    //判断是否执行登录
    if cmd == GameCode::Login.into_u32() {
        let mut c_login = C_USER_LOGIN::new();
        let result = c_login.merge_from_bytes(packet.get_data());

        if let Err(e) = result {
            error!("{:?}", e);
            return;
        }
        //执行登录
        let result = login(gm, packet).await;
        if let Err(e) = result {
            error!("{:?}", e);
            return;
        }
    } else {
        //不登录就执行其他命令
        gm.lock().await.invok(packet);
    }
}

//登录函数，执行登录
async fn login(gm: Lock, packet: Packet) -> anyhow::Result<()> {
    //玩家id
    let user_id = packet.get_user_id();
    let mut gm_lock = gm.lock().await;
    let user_data = gm_lock.users.contains_key(&user_id);
    //走登录流程
    //如果内存没有数据，则从数据库里面找
    if !user_data {
        //初始化玩家数据
        let user_data = init_user_data(user_id)?;
        gm_lock.users.insert(user_id, user_data);
    }
    check_nick_name(&mut gm_lock, user_id);
    //封装会话
    let user_data = gm_lock.users.get_mut(&user_id);
    if user_data.is_none() {
        let str = format!("there is no data for userid:{}", &user_id);
        anyhow::bail!(str)
    }
    let user_data = user_data.unwrap();

    let user = user_data.get_user_info_mut_ref();
    //处理更新登陆
    user.update_login();
    //处理重制惩罚时间
    user.reset_punish_match();

    //返回客户端
    let lr = gm_lock.user2proto(user_id);
    gm_lock.send_2_client(ClientCode::Login, user_id, lr.write_to_bytes()?);
    info!("用户完成登录！user_id:{}", &user_id);
    Ok(())
}

fn check_nick_name(gm_lock: &mut async_std::sync::MutexGuard<GameMgr>, user_id: u32) {
    let user_data = gm_lock.users.get_mut(&user_id);
    if user_data.is_none() {
        return;
    }
    let user_data = user_data.unwrap();
    let user = user_data.get_user_info_mut_ref();
    let nick_name = user.nick_name.as_str();
    let value = get_user_from_redis(user.user_id);
    if value.is_none() {
        let str = format!("redis has no data for user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        return;
    }
    let json_value = value.unwrap();

    let redis_nick_name = json_value.get("nick_name");
    if redis_nick_name.is_none() {
        return;
    }
    let redis_nick_name = redis_nick_name.unwrap();
    let redis_nick_name = redis_nick_name.as_str();
    if redis_nick_name.is_none() {
        return;
    }
    let redis_nick_name = redis_nick_name.unwrap();
    if nick_name == redis_nick_name {
        return;
    }

    user.set_nick_name(redis_nick_name);
    //通知排行榜服务器
    let mut grnn = G_S_MODIFY_NICK_NAME::new();
    grnn.set_nick_name(redis_nick_name.to_owned());
    let bytes = grnn.write_to_bytes();
    match bytes {
        Ok(bytes) => {
            info!("执行修改昵称函数!");
            //通知其排行榜服
            gm_lock.send_2_server(RankCode::ModifyNickName.into_u32(), user_id, bytes);
        }
        Err(e) => {
            error!("{:?}", e);
        }
    }
}

///初始化玩家数据
fn init_user_data(user_id: u32) -> anyhow::Result<UserData> {
    //判断redis里面有没有,用户中心没有则直接代表不合法，不与执行
    let value = get_user_from_redis(user_id);
    if value.is_none() {
        let str = format!("redis has no data for user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        anyhow::bail!(str)
    }

    let mut ud = UserData::init_from_db(user_id);

    //数据库没有则创建新号
    if ud.is_none() {
        let json_value = value.unwrap();
        let nick_name = json_value.get("nick_name");
        if nick_name.is_none() {
            let str = format!("nick_name is none for user_id:{}", user_id);
            error!("{:?}", str.as_str());
            anyhow::bail!(str)
        }
        let mut user = User::new(user_id, nick_name.unwrap().as_str().unwrap());
        //以下入库采用异步执行，以免造成io堵塞
        //玩家角色数据
        let c = Characters::new(user.user_id);
        user.set_last_character(c.get_frist());
        //grade相框
        let grade_frame = GradeFrame::new(user.user_id);
        //灵魂头像
        let soul = Soul::new(user.user_id);

        //封装到userdata里
        ud = Some(UserData::new(
            user.clone(),
            c.clone(),
            grade_frame.clone(),
            soul.clone(),
        ));

        //异步持久化到db
        async_std::task::spawn(insert_user(user));
        async_std::task::spawn(insert_characters(c));
        async_std::task::spawn(insert_soul(soul));
        async_std::task::spawn(insert_grade_frame(grade_frame));
    }
    Ok(ud.unwrap())
}

pub fn new(address: &str, gm: Lock) {
    let sh = TcpServerHandler { gm };
    let res = tools::tcp::tcp_server::new(address.to_string(), sh);
    let res = block_on(res);
    if let Err(e) = res {
        error!("{:?}", e);
        std::process::abort();
    }
}
