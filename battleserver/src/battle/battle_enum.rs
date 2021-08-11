use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

///默认每个turn移动点数
pub const TURN_DEFAULT_MOVEMENT_POINTS: u8 = 2;

///最大turn次数限制
pub const LIMIT_TOTAL_TURN_TIMES: u16 = 999;

///触发范围一圈不包括中心
pub const TRIGGER_SCOPE_NEAR: [isize; 6] = [-6, -5, -1, 1, 5, 6];
///触发范围一圈不包括中心
pub const TRIGGER_SCOPE_NEAR_TEMP_ID: u32 = 2;

///触发范围一圈包括中心
pub const TRIGGER_SCOPE_CENTER_NEAR_TEMP_ID: u32 = 3;

///技能限制类型
pub mod skill_judge_type {
    ///hp限定：大于
    pub const HP_LIMIT_GT: u32 = 1;
    ///回合限制
    pub const LIMIT_ROUND_TIMES: u32 = 2;
    ///turn限制
    pub const LIMIT_TURN_TIMES: u32 = 3;
    ///配对限制
    pub const PAIR_LIMIT: u32 = 4;
}

///技能类型
pub mod skill_type {

    ///技能翻开地图块
    pub const SKILL_OPEN_MAP_CELL: [u32; 1] = [223];
    ///自残加buff
    pub const HURT_SELF_ADD_BUFF: [u32; 1] = [311];
    ///自动配对地图块
    pub const AUTO_PAIR_MAP_CELL: [u32; 1] = [212];
    ///上buff
    pub const ADD_BUFF: [u32; 9] = [122, 211, 221, 311, 312, 322, 324, 11004, 20002];
    ///地图块换位置
    pub const CHANGE_MAP_CELL_INDEX: [u32; 1] = [111];
    ///展示地图块
    pub const SHOW_MAP_CELL: [u32; 5] = [112, 113, 421, 423, 20001];
    ///移动玩家
    pub const MOVE_USER: [u32; 1] = [222];
    ///相临玩家造成技能伤害并恢复生命
    pub const NEAR_SKILL_DAMAGE_AND_CURE: [u32; 1] = [321];
    ///技能伤害
    pub const SKILL_DAMAGE: [u32; 8] = [999, 123, 20004, 20005, 323, 433, 331, 11001];
    ///技能aoe
    pub const SKILL_AOE: [u32; 5] = [121, 411, 412, 432, 11005];
    ///减技能cd
    pub const RED_SKILL_CD: [u32; 1] = [20003];
    ///对已其他翻开元素块上对玩家造成技能伤害
    pub const SKILL_DAMAGE_OPENED_ELEMENT: [u32; 1] = [213];
    ///范围治疗
    pub const SCOPE_CURE: [u32; 1] = [313];
    ///变身
    pub const TRANSFORM: [u32; 1] = [431];
    ///展示地图块
    pub const SHOW_INDEX: [u32; 1] = [422];

    ///---------------------------以下为了方便单独定义出来
    ///水炮
    pub const WATER_TURRET: u32 = 323;
    ///翻开附近地图块
    pub const SKILL_OPEN_NEAR_CELL: u32 = 223;
    ///向所有玩家展示一个随机地图块，优先展示生命元素的
    pub const SHOW_ALL_USERS_CELL: u32 = 113;
    ///技能伤害，若目标在附近，则伤害加深
    pub const SKILL_DAMAGE_NEAR_DEEP: u32 = 999;
    ///展示所有相同元素的地图块给所有玩家并治疗
    pub const SHOW_SAME_ELMENT_CELL_ALL_AND_CURE: u32 = 423;
    ///技能aoe中心伤害加深
    pub const SKILL_AOE_CENTER_DAMAGE_DEEP: u32 = 432;
    ///造成aoe伤害，并且减技能cd
    pub const SKILL_AOE_RED_SKILL_CD: u32 = 121;
    ///选择一个地图块，展示其相同元素地图块位置
    pub const SHOW_INDEX_SAME_ELEMENT: u32 = 422;
    ///配对可用，造成伤害
    pub const SKILL_PAIR_LIMIT_DAMAGE: [u32; 1] = [331];
    ///召唤宠物
    pub const SUMMON_MINON: [u32; 1] = [11002];

    ///减少最大行动点数
    pub const SUB_MAX_MOVE_POINT: u32 = 11004;

    ///范围变化的aoe技能
    pub const SCOPE_CHANGE_SKILL_AOE: u32 = 11005;
}

///buff类型
pub mod buff_type {
    ///格挡伤害
    pub const GD_ATTACK_DAMAGE: [u32; 1] = [2];
    ///变成技能
    pub const CHANGE_SKILL: [u32; 1] = [3];
    ///增加攻击力并变成AOE
    pub const ADD_ATTACK_AND_AOE: [u32; 1] = [4];
    ///增加攻击力
    pub const ADD_ATTACK: [u32; 5] = [4, 7, 16, 1001, 1002];
    ///减伤buff
    pub const SUB_ATTACK_DAMAGE: [u32; 2] = [8, 10001];
    ///获得道具
    pub const AWARD_ITEM: [u32; 5] = [10003, 30011, 30021, 30031, 30041];
    ///配对恢复生命
    pub const PAIR_CURE: u32 = 30012;
    ///获得buff
    pub const AWARD_BUFF: u32 = 30022;
    ///相临技能cd增加
    pub const NEAR_ADD_CD: u32 = 30032;
    ///配对成功相临造成技能伤害
    pub const NEAR_SKILL_DAMAGE_PAIR: u32 = 30042;
    ///其他玩家移动到相临造成技能伤害
    pub const DEFENSE_NEAR_MOVE_SKILL_DAMAGE: [u32; 1] = [1];
    ///被攻击时增加能量
    pub const ATTACKED_ADD_ENERGY: [u32; 1] = [10004];
    /// 匹配属性一样的地图块+攻击
    pub const PAIR_SAME_ELEMENT_ADD_ATTACK: u32 = 1001;
    ///当地图重制，+攻击力
    pub const RESET_MAP_ADD_ATTACK: [u32; 1] = [1002];
    /// 移动干点啥，配对又干点啥
    pub const MANUAL_MOVE_AND_PAIR_ADD_ENERGY: [u32; 1] = [1004];
    /// 无法被移动
    pub const CAN_NOT_MOVED: u32 = 10002;
    ///锁buff
    pub const LOCKED: u32 = 6;
    ///配对与自己相同元素时恢复生命
    pub const PAIR_SAME_ELEMENT_CURE: u32 = 9;
    ///陷阱类buff
    pub const TRAPS: [u32; 2] = [10, 12];
    ///配对成功刷新技能cd
    pub const PAIR_CLEAN_SKILL_CD: u32 = 13;
    ///配对成功，刷新技能cd或者减少技能cd
    pub const PAIR_SAME_ELEMENT_CLEAN_OR_SUB_SKILL_CD: u32 = 1003;
    ///变身buff
    pub const TRANSFORM_BUFF: [u32; 1] = [14];

    ///---------------------------以下为了方便单独定义出来
    /// 受到相临攻击时候减伤
    pub const NEAR_SUB_ATTACK_DAMAGE: u32 = 10001;

    ///对移动到此地图块上的玩家施加一个buff
    pub const TRAP_ADD_BUFF: [u32; 1] = [10];

    ///陷阱造成技能伤害
    pub const TRAP_SKILL_DAMAGE: [u32; 1] = [12];

    ///锁定技能
    pub const LOCK_SKILLS: [u32; 1] = [11];

    ///被攻击时减cd
    pub const ATTACKED_SUB_CD: u32 = 15;

    ///被攻击时减伤
    pub const ATTACKED_SUB_DAMAGE: u32 = 17;

    ///扣行动点数上限
    pub const SUB_MOVE_POINT: u32 = 20;

    ///受到攻击伤害变成0
    pub const NEAR_ATTACKED_DAMAGE_ZERO: u32 = 20001;

    ///每当有指定元素地图块被翻开，刷新技能cd
    pub const OPEN_ELEMENT_CELL_CLEAR_CD: u32 = 20002;

    ///死了就结束回合
    pub const DIE_END_TURN: u32 = 20004;

    ///将攻击伤害返伤成技能伤害
    pub const RETURN_ATTACKED_DAMAGE_TO_SKILL_AOE: u32 = 20005;

    ///自杀造成技能伤害
    pub const SUICIDE_SKILL_DAMAGE: u32 = 20006;

    ///世界树buff,在战斗开始的时候就开始加载，
    pub const WORLD_CELL_BUFFS: u32 = 30051;
}

///pos操作类型
#[derive(Debug, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum PosType {
    ChangePos = 1, //切换架势
    CancelPos = 2, //取消架势
}

impl PosType {
    pub fn into_u32(self) -> u32 {
        let value: u8 = self.into();
        value as u32
    }
}

///效果类型
#[derive(Debug, Clone, Copy, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum EffectType {
    ///技能伤害
    SkillDamage = 1,
    ///攻击伤害
    AttackDamage = 2,
    ///治疗血量
    Cure = 3,
    ///减攻击伤害
    SubDamage = 4,
    ///技能减少cd
    SubSkillCd = 5,
    ///获得道具
    RewardItem = 6,
    ///增加技能cd
    AddSkillCd = 7,
    ///增加能量
    AddEnergy = 8,
    ///增加技能
    AddSkill = 9,
    ///刷新技能cd
    RefreshSkillCd = 10,
    ///换地图块位置
    ChangeCellIndex = 11,
}

impl EffectType {
    pub fn into_u32(self) -> u32 {
        let res: u8 = self.into();
        res as u32
    }

    pub fn into_u8(self) -> u8 {
        let res: u8 = self.into();
        res
    }
}

///技能消耗类型
#[derive(Debug, Clone, Copy, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum SkillConsumeType {
    Energy = 1, //能量
}

impl SkillConsumeType {
    pub fn into_u8(self) -> u8 {
        let res: u8 = self.into();
        res
    }
    pub fn into_u32(self) -> u32 {
        let res = self.into_u8();
        res as u32
    }
}

///回合行为类型
#[derive(Debug, Clone, Copy, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum BattlePlayerState {
    Normal = 0,
    Died = 1,    //淘汰
    OffLine = 2, //离线
}

impl Default for BattlePlayerState {
    fn default() -> Self {
        BattlePlayerState::Normal
    }
}

impl BattlePlayerState {
    pub fn into_u8(self) -> u8 {
        let value: u8 = self.into();
        value
    }
}

///角色状态
#[derive(Debug, Clone, Copy, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum BattleCterState {
    Alive = 0,
    Died = 1,
}

impl Default for BattleCterState {
    fn default() -> Self {
        BattleCterState::Alive
    }
}

impl BattleCterState {
    pub fn into_u8(self) -> u8 {
        let value: u8 = self.into();
        value
    }
}

///攻击状态
#[derive(Debug, Clone, Copy, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum AttackState {
    None = 0,   //无效
    Able = 1,   //有效
    Locked = 2, //锁定，不可攻击
}

impl Default for AttackState {
    fn default() -> Self {
        AttackState::None
    }
}

impl AttackState {
    pub fn into_u8(self) -> u8 {
        let value: u8 = self.into();
        value
    }
}

///回合行为类型
#[derive(Debug, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ActionType {
    ///无效值
    None = 0,
    ///普通攻击
    Attack = 1,
    ///使用道具
    UseItem = 2,
    ///跳过turn
    Skip = 3,
    ///翻块
    Open = 4,
    ///使用技能
    Skill = 5,
    ///触发buff
    Buff = 6,
    ///结束展示地图块(解锁玩家状态)
    EndShowMapCell = 7,
}

impl ActionType {
    pub fn into_u32(self) -> u32 {
        let res: u8 = self.into();
        res as u32
    }

    pub fn into_u8(self) -> u8 {
        self.into()
    }
}

///目标类型枚举
#[derive(Debug, Clone, Copy, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum TargetType {
    None = 0,                    //无效目标
    MapCell = 1,                 //地图块
    AnyPlayer = 2,               //任意玩家
    PlayerSelf = 3,              //玩家自己
    AllPlayer = 4,               //所有玩家
    OtherAllPlayer = 5,          //除自己外所有玩家
    OtherAnyPlayer = 6,          //除自己外任意玩家
    UnOpenMapCell = 7,           //未翻开的地图块
    UnPairMapCell = 8,           //未配对的地图块
    NullMapCell = 9,             //空的地图块，上面没人
    UnPairNullMapCell = 10,      //未配对的空地图块
    MapCellPlayer = 11,          //地图块上的玩家
    SelfScopeOthers = 12,        //以自己为中心某个范围内的所有其他玩家
    SelfScopeAnyOthers = 13,     //以自己为中心某个范围内的任意其他玩家
    SelfScopeAll = 14,           //以自己为中心某个范围内的所有玩家（包括自己）
    SelfScopeAny = 15,           //以自己为中心某个范围内的任意玩家（包括自己）
    OpenedMapCell = 16,          //已翻开的地图块
    MapCellOtherPlayer = 17,     //地图块上，除自己以外的玩家
    UnOpenMapCellAndUnLock = 18, //未翻开，且未锁定
    UnLockNullMapCell = 19,      //未锁定空地图块
    UnOpenNullMapCell = 20,      //未翻开的空地图快
    AllEnemyCters = 21,          //所有敌方角色
    AllTeamCters = 22,           //所有友方角色
    AnyEnemyCter = 23,           //任意敌方角色
    AnyTeamCter = 24,            //任意友方角色
    SelfScopeAllEnemyCters = 25, //以自身为一圈的所有敌方角色
    SelfScopeAnyEnemyCters = 26, //以自身为一圈的任意敌方角色
    MapCellEnemys = 27,          //地图块上的敌人
}

impl TargetType {
    pub fn into_u8(self) -> u8 {
        let res: u8 = self.into();
        res
    }

    pub fn into_u32(self) -> u32 {
        let res = self.into_u8() as u32;
        res
    }
}

///元素类型
#[derive(Debug, Clone, Copy, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ElementType {
    Nature = 1, //生命元素
    Earth = 2,  //土元素
    Water = 3,  //水元素
    Fire = 4,   //火元素
}

impl ElementType {
    pub fn into_u8(self) -> u8 {
        let res: u8 = self.into();
        res
    }
}
