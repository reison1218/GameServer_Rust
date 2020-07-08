///游戏服务专用命令号段枚举
pub enum GameCode{
    //最小值
    Min = 1000,
    //心跳
    HeartBeat = 1001,
    //离线
    LineOff = 1002,
    //登陆
    Login = 1003,
    //同步数据
    SyncData = 1004,
    //请求修改昵称
    ModifyNickName = 1005,
    //创建房间
    CreateRoom = 1006,
    //加入房间
    JoinRoom = 1007,
    //匹配房间
    SearchRoom = 1008,
    //范围最大值
    Max = 10000,
}

///房间服专属命令号段枚举
pub enum RoomCode{
    //范围最小值
    Min = 20001,
    //离线
    LineOff = 20002,
    //创建房间
    CreateRoom = 20003,
    //离开房间
    LeaveRoom = 20004,
    //T人
    Kick = 20005,
    //换队伍
    ChangeTeam = 20006,
    //准备与取消
    PrepareCancel = 20007,
    //开始游戏
    StartGame = 20008,
    //加入房间
    JoinRoom = 20009,
    //匹配房间
    SearchRoom = 20010,
    //房间设置
    RoomSetting = 20011,
    //选择角色和技能
    ChoiceCharacter = 20012,
    //表情符号
    Emoji = 20013,
    //选择位置
    ChoiceLoaction = 20014,
    //选择回合顺序
    ChoiceRoundOrder = 20015,
    //返回最大值
    Max = 30000,
}

///客户端专属命令号段枚举
pub enum ClientCode{
    //最小命令号
    Min = 10001,
    //返回心跳包
    HeartBeat = 10002,
    //返回登陆命令号
    Login = 10003,
    //返回同步命令号
    SyncData = 10004,
    //请求修改昵称返回
    NickNameModify = 10005,
    //房间命令号
    Room = 10006,
    //离开房间命令号
    LeaveRoom = 10007,
    //开始游戏
    Start = 10008,
    //换队伍
    ChangeTeam = 10009,
    //准备与取消准备
    PrepareCancel=10010,
    //房间设置
    RoomSetting=10011,
    //房间成员变动推送消息
    RoomMemberNotice=10012,
    //T人返回
    KickMember=10013,
    //选择角色
    ChooseCharacter=10014,
    //房间推送
    RoomNotice=10015,
    //表情符号
    Emoji=10016,
    //表情推送
    EmojiNotice = 10017,
    //离开房间推送
    MemberLeaveNotice = 10018,
    //游戏开始推送
    StartNotice = 10019,
    //选择位置返回
    ChoiceLoaction = 10020,
    //选择回合顺序返回
    ChoiceRoundOrder = 10021,
    //选择位置通知
    ChoiceLoactionNotice = 10022,
    //选择回合顺序通知
    ChoiceRoundOrderNotice = 10023,
    //战斗开始
    BattleStartNotice = 10024,
    //最大命令号
    Max = 20000,
}

impl From<u32> for ClientCode{
    fn from(v: u32) -> Self {
        if v == ClientCode::Login as u32{
            return ClientCode::Login;
        }
        //返回同步命令号
        if v == ClientCode::SyncData as u32{
            return ClientCode::SyncData
        }
        //请求修改昵称返回
        if v == ClientCode::NickNameModify as u32{
            return ClientCode::NickNameModify
        }
        //房间命令返回
        if v == ClientCode::Room as u32{
            return ClientCode::Room
        }
        //离开房间
        if v == ClientCode::LeaveRoom as u32{
            return ClientCode::LeaveRoom
        }
        //开始游戏
        if v == ClientCode::Start as u32{
            return ClientCode::Start
        }
        //改变队伍
        if v == ClientCode::ChangeTeam as u32{
            return ClientCode::ChangeTeam
        }
        //准备与取消
        if v == ClientCode::PrepareCancel as u32{
            return ClientCode::PrepareCancel
        }
        //房间设置
        if v == ClientCode::RoomSetting as u32{
            return ClientCode::RoomSetting
        }
        //成员变动推送
        if v == ClientCode::RoomMemberNotice as u32{
            return ClientCode::RoomMemberNotice
        }
        //T玩家
        if v == ClientCode::KickMember as u32{
            return ClientCode::KickMember
        }
        //选择角色和技能
        if v == ClientCode::ChooseCharacter as u32{
            return ClientCode::ChooseCharacter
        }
        //房间推送
        if v == ClientCode::RoomNotice as u32{
            return ClientCode::RoomNotice
        }
        //表情
        if v == ClientCode::Emoji as u32{
            return ClientCode::Emoji
        }
        //表情推送
        if v == ClientCode::EmojiNotice as u32{
            return ClientCode::EmojiNotice
        }
        //成员离开房间推送
        if v == ClientCode::MemberLeaveNotice as u32{
            return ClientCode::MemberLeaveNotice
        }
        //开始游戏推送
        if v == ClientCode::StartNotice as u32{
            return ClientCode::StartNotice
        }
        //选择位置返回
        if v == ClientCode::ChoiceLoaction as u32{
            return ClientCode::ChoiceLoaction
        }
        //选择回合顺序返回
        if v == ClientCode::ChoiceRoundOrder as u32{
            return ClientCode::ChoiceRoundOrder
        }
        //选择位置通知
        if v == ClientCode::ChoiceLoactionNotice as u32{
            return ClientCode::ChoiceLoactionNotice
        }
        //选择回合顺序通知
        if v == ClientCode::ChoiceRoundOrderNotice as u32{
            return ClientCode::ChoiceRoundOrderNotice
        }
        ClientCode::Login
    }
}