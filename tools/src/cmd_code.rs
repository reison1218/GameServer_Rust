use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

///游戏服务专用命令号段枚举 1000-10000
#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum ServerCommonCode {
    //热更新静态配置
    ReloadTemps = 101,
    //更新赛季
    UpdateSeason = 102,
}

impl ServerCommonCode {
    pub fn into_u32(self) -> u32 {
        let res: u32 = self.into();
        res
    }
}

///游戏服务专用命令号段枚举 1000-10000
#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum GameCode {
    //最小值
    Min = 1000,
    //心跳
    HeartBeat = 1001,
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
    //战斗结算
    Summary = 1009,
    //匹配惩罚同步
    SyncPunish = 1010,
    //同步排行榜快照
    SyncRank = 1011,
    //展示排行榜(客户端请求指令)
    ShowRank = 1012,
    //修改grade相框和soul头像
    ModifyGradeFrameAndSoul = 1013,
    //更新上赛季排行榜通知
    UpdateLastSeasonRankPush = 1014,
    //获得上赛季排行榜信息
    GetLastSeasonRank = 1015,
    //卸载玩家数据
    UnloadUser = 9999,
    //范围最大值
    Max = 10000,
}

impl GameCode {
    pub fn into_u32(self) -> u32 {
        let res: u32 = self.into();
        res
    }
}

///房间服专属命令号段枚举 20001-30000
#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum RoomCode {
    //范围最小值
    Min = 20001,
    //离开房间
    OffLine = 20002,
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
    //选择角色
    ChoiceCharacter = 20012,
    //表情符号
    Emoji = 20013,
    //选择角色技能
    ChoiceSkill = 20014,
    //确认进入房间，只针对匹配模式有用
    ConfirmIntoRoom = 20015,
    //取消匹配
    CancelSearch = 20016,
    //--------------------------------------以下战斗相关---------------------------
    //战斗结算
    Summary = 21000,
    //返回最大值
    Max = 30000,
}

impl RoomCode {
    pub fn into_u32(self) -> u32 {
        let res: u32 = self.into();
        res
    }
}

///房间服专属命令号段枚举 30001-40000
#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum BattleCode {
    Min = 30001,
    Start = 30002,       //开始战斗
    ChoiceIndex = 30003, //选择位置
    Action = 30004,      //请求行动
    Pos = 30005,         //架势请求
    Emoji = 30006,       //表情符号
    OffLine = 39998,     //掉线
    LeaveRoom = 39999,   //离开房间
    Max = 40000,
}

impl BattleCode {
    pub fn into_u32(self) -> u32 {
        let res: u32 = self.into();
        res
    }
}

///排行服专属命令号段枚举 40001-50000
#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum RankCode {
    Min = 40001,
    UpdateRank = 40002, //更新排行榜
    GetRank = 40003,    //获得排行榜
    Max = 50000,
}

impl RankCode {
    pub fn into_u32(self) -> u32 {
        let res: u32 = self.into();
        res
    }
}

///客户端专属命令号段枚举 10001-20000
#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum ClientCode {
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
    //换队伍通知
    ChangeTeamNotice = 10009,
    //准备与取消准备
    PrepareCancel = 10010,
    //房间设置
    RoomSetting = 10011,
    //房间新成员推送消息
    RoomAddMemberNotice = 10012,
    //T人返回
    KickMember = 10013,
    //选择角色
    ChoiceCharacter = 10014,
    //房间推送
    RoomNotice = 10015,
    //表情符号
    Emoji = 10016,
    //表情推送
    EmojiNotice = 10017,
    //离开房间推送
    MemberLeaveNotice = 10018,
    //选择角色推送
    ChoiceCharacterNotice = 10019,
    //选择角色技能
    ChoiceSkill = 10020,
    //准备状态推送
    PrepareCancelNotice = 10021,
    //游戏开始推送
    StartNotice = 10022,
    //选择位置返回
    ChoiceIndex = 10023,
    //选择回合顺序返回
    ChoiceRoundOrder = 10024,
    //选择位置通知
    ChoiceIndexNotice = 10025,
    //选择回合顺序通知
    ChoiceRoundOrderNotice = 10026,
    //选择占位推送
    StartChoiceIndexNotice = 10027,
    //“进入房间”取消通知
    IntoRoomCancelNotice = 10028,
    //匹配成功通知
    MatchSuccessNotice = 10029,
    //--------------------------------------以下战斗相关---------------------------
    //战斗开始推送
    BattleStartedNotice = 10030,
    //行动推送
    ActionNotice = 10031,
    //turn结算推送
    BattleTurnNotice = 10032,
    //架势推送
    PosNotice = 10033,
    //地图刷新推送
    MapRefreshNotice = 10040,
    //结算推送
    SummaryNotice = 10041,
    //-------------------------------------战斗结束---------------------------
    //取消匹配
    CancelSearch = 10060,
    //确认进入房间推送
    ConfirmIntoRoomNotice = 10061,
    //展示排行榜
    ShowRank = 10081,
    //修改grade相框和soul头像
    ModifyGradeFrameAndSoul = 10082,
    //获得上赛季排行榜返回
    GetLastSeasonRank = 10083,
    //最大命令号
    Max = 20000,
}

impl ClientCode {
    pub fn into_u32(self) -> u32 {
        let res: u32 = self.into();
        res
    }
}
