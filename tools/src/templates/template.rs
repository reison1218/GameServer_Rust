use crate::templates::battle_limit_time_temp::{BattleLimitTimeTemp, BattleLimitTimeTempMgr};
use crate::templates::buff_temp::{BuffTemp, BuffTempMgr};
use crate::templates::cell_temp::{CellTemp, CellTempMgr};
use crate::templates::character_temp::{CharacterTemp, CharacterTempMgr};
use crate::templates::constant_temp::{ConstantTemp, ConstantTempMgr};
use crate::templates::emoji_temp::{EmojiTemp, EmojiTempMgr};
use crate::templates::grade_frame_temp::{GradeFrameTemp, GradeFrameTempMgr};
use crate::templates::item_temp::{ItemTemp, ItemTempMgr};
use crate::templates::league_temp::{LeagueTemp, LeagueTempMgr};
use crate::templates::punish_temp::{PunishTemp, PunishTempMgr};
use crate::templates::robot_temp::{RobotTemp, RobotTempMgr};
use crate::templates::season_temp::{SeasonTemp, SeasonTempMgr};
use crate::templates::skill_judge_temp::{SkillJudgeTemp, SkillJudgeTempMgr};
use crate::templates::skill_scope_temp::{SkillScopeTemp, SkillScopeTempMgr};
use crate::templates::skill_temp::{SkillTemp, SkillTempMgr};
use crate::templates::soul_temp::{SoulTemp, SoulTempMgr};
use crate::templates::summary_award_temp::{SummaryAwardTemp, SummaryAwardTempMgr};
use crate::templates::template_name_constants::{
    BATTLE_LIMIT_TIME, BUFF, CELL_TEMPLATE, CHARACTER_TEMPLATE, CONSTANT_TEMPLATE, EMOJI_TEMPLATE,
    GRADE_FRAME, ITEM_TEMPLATE, LEAGUE, PUNISH, ROBOT, SEASON, SKILL_JUDGE_TEMPLATE,
    SKILL_SCOPE_TEMPLATE, SKILL_TEMPLATE, SOUL, SUMMARY_AWARD, TILE_MAP_TEMPLATE,
    WORLD_CELL_TEMPLATE,
};
use crate::templates::tile_map_temp::{TileMapTemp, TileMapTempMgr};
use crate::templates::world_cell_temp::{WorldCellTemp, WorldCellTempMgr};
use log::error;
use std::borrow::{Borrow, BorrowMut};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use super::market_temp::MarketTemp;
use super::market_temp::MarketTempMgr;
use super::merchandise_temp::{MerchandiseTemp, MerchandiseTempMgr};
use super::mission_temp::{MissionTemp, MissionTempMgr};
use super::template_name_constants::{MARKET, MERCHANDISE, MISSION};

pub trait Template {}

pub trait TemplateMgrTrait: Send + Sync {
    fn is_empty(&self) -> bool;
    fn clear(&mut self);
}

//配置表mgr
#[derive(Debug, Default)]
pub struct TemplatesMgr {
    character_temp_mgr: CharacterTempMgr,               //角色配置mgr
    tile_map_temp_mgr: TileMapTempMgr,                  //地图配置mgr
    emoji_temp_mgr: EmojiTempMgr,                       //表情配置mgr
    constant_temp_mgr: ConstantTempMgr,                 //常量配置mgr
    world_cell_temp_mgr: WorldCellTempMgr,              //worldcell配置mgr
    cell_temp_mgr: CellTempMgr,                         //cell配置mgr
    skill_temp_mgr: SkillTempMgr,                       //技能配置mgr
    item_temp_mgr: ItemTempMgr,                         //道具配置mgr
    skill_scope_temp_mgr: SkillScopeTempMgr,            //技能范围配置mgr
    buff_temp_mgr: BuffTempMgr,                         //buff配置mgr
    skill_judge_temp_mgr: SkillJudgeTempMgr,            //判定条件配置mgr
    season_temp_mgr: SeasonTempMgr,                     //赛季配置mgr
    robot_temp_mgr: RobotTempMgr,                       //机器人配置mgr
    league_temp_mgr: LeagueTempMgr,                     //段位配置mgr
    summary_award_temp_mgr: SummaryAwardTempMgr,        //结算奖励配置mgr
    battle_limit_time_temp_mgr: BattleLimitTimeTempMgr, //战斗turn时间限制模版
    punish_temp_mgr: PunishTempMgr,                     //惩罚时间
    grade_frame_temp_mgr: GradeFrameTempMgr,            //gradeframe
    soul_temp_mgr: SoulTempMgr,                         //灵魂头像
    market_temp_mgr: MarketTempMgr,                     //市场配置
    merchandise_temp_mgr: MerchandiseTempMgr,           //商品配置
    mission_temp_mgr: MissionTempMgr,                   //任务配置
}

impl TemplatesMgr {
    pub fn execute_init(&self) {
        self.constant_temp_mgr();
    }

    pub fn reload_temps(&self, path: &str) -> anyhow::Result<()> {
        let mgr_ptr = self as *const TemplatesMgr as *mut TemplatesMgr;
        unsafe {
            let mgr_mut = mgr_ptr.as_mut().unwrap();
            mgr_mut.character_temp_mgr.clear();
            mgr_mut.tile_map_temp_mgr.clear();
            mgr_mut.emoji_temp_mgr.clear();
            mgr_mut.constant_temp_mgr.clear();
            mgr_mut.world_cell_temp_mgr.clear();
            mgr_mut.cell_temp_mgr.clear();
            mgr_mut.skill_temp_mgr.clear();
            mgr_mut.item_temp_mgr.clear();
            mgr_mut.skill_scope_temp_mgr.clear();
            mgr_mut.buff_temp_mgr.clear();
            mgr_mut.skill_judge_temp_mgr.clear();
            mgr_mut.season_temp_mgr.clear();
            mgr_mut.robot_temp_mgr.clear();
            mgr_mut.league_temp_mgr.clear();
            mgr_mut.summary_award_temp_mgr.clear();
            mgr_mut.battle_limit_time_temp_mgr.clear();
            mgr_mut.punish_temp_mgr.clear();
            mgr_mut.grade_frame_temp_mgr.clear();
            mgr_mut.soul_temp_mgr.clear();
            mgr_mut.market_temp_mgr.clear();
            mgr_mut.merchandise_temp_mgr.clear();
            mgr_mut.mission_temp_mgr.clear();
            let res = read_templates_from_dir(path, mgr_mut);
            if let Err(e) = res {
                error!("{:?}", e);
                return Ok(());
            }
        }
        Ok(())
    }

    /// Get a reference to the templates mgr's character temp mgr.
    pub fn character_temp_mgr(&self) -> &CharacterTempMgr {
        &self.character_temp_mgr
    }

    /// Get a reference to the templates mgr's tile map temp mgr.
    pub fn tile_map_temp_mgr(&self) -> &TileMapTempMgr {
        &self.tile_map_temp_mgr
    }

    /// Get a reference to the templates mgr's emoji temp mgr.
    pub fn emoji_temp_mgr(&self) -> &EmojiTempMgr {
        &self.emoji_temp_mgr
    }

    /// Get a reference to the templates mgr's constant temp mgr.
    pub fn constant_temp_mgr(&self) -> &ConstantTempMgr {
        &self.constant_temp_mgr
    }

    /// Get a reference to the templates mgr's world cell temp mgr.
    pub fn world_cell_temp_mgr(&self) -> &WorldCellTempMgr {
        &self.world_cell_temp_mgr
    }

    /// Get a reference to the templates mgr's cell temp mgr.
    pub fn cell_temp_mgr(&self) -> &CellTempMgr {
        &self.cell_temp_mgr
    }

    /// Get a reference to the templates mgr's skill temp mgr.
    pub fn skill_temp_mgr(&self) -> &SkillTempMgr {
        &self.skill_temp_mgr
    }

    /// Get a reference to the templates mgr's item temp mgr.
    pub fn item_temp_mgr(&self) -> &ItemTempMgr {
        &self.item_temp_mgr
    }

    /// Get a reference to the templates mgr's skill scope temp mgr.
    pub fn skill_scope_temp_mgr(&self) -> &SkillScopeTempMgr {
        &self.skill_scope_temp_mgr
    }

    /// Get a reference to the templates mgr's buff temp mgr.
    pub fn buff_temp_mgr(&self) -> &BuffTempMgr {
        &self.buff_temp_mgr
    }

    /// Get a reference to the templates mgr's skill judge temp mgr.
    pub fn skill_judge_temp_mgr(&self) -> &SkillJudgeTempMgr {
        &self.skill_judge_temp_mgr
    }

    /// Get a reference to the templates mgr's season temp mgr.
    pub fn season_temp_mgr(&self) -> &SeasonTempMgr {
        &self.season_temp_mgr
    }

    /// Get a reference to the templates mgr's robot temp mgr.
    pub fn robot_temp_mgr(&self) -> &RobotTempMgr {
        &self.robot_temp_mgr
    }

    /// Get a reference to the templates mgr's league temp mgr.
    pub fn league_temp_mgr(&self) -> &LeagueTempMgr {
        &self.league_temp_mgr
    }

    /// Get a reference to the templates mgr's summary award temp mgr.
    pub fn summary_award_temp_mgr(&self) -> &SummaryAwardTempMgr {
        &self.summary_award_temp_mgr
    }

    /// Get a reference to the templates mgr's battle limit time temp mgr.
    pub fn battle_limit_time_temp_mgr(&self) -> &BattleLimitTimeTempMgr {
        &self.battle_limit_time_temp_mgr
    }

    /// Get a reference to the templates mgr's punish temp mgr.
    pub fn punish_temp_mgr(&self) -> &PunishTempMgr {
        &self.punish_temp_mgr
    }

    /// Get a reference to the templates mgr's grade frame temp mgr.
    pub fn grade_frame_temp_mgr(&self) -> &GradeFrameTempMgr {
        &self.grade_frame_temp_mgr
    }

    /// Get a reference to the templates mgr's soul temp mgr.
    pub fn soul_temp_mgr(&self) -> &SoulTempMgr {
        &self.soul_temp_mgr
    }

    /// Get a reference to the templates mgr's market temp mgr.
    pub fn market_temp_mgr(&self) -> &MarketTempMgr {
        &self.market_temp_mgr
    }

    /// Get a reference to the templates mgr's merchandise temp mgr.
    pub fn merchandise_temp_mgr(&self) -> &MerchandiseTempMgr {
        &self.merchandise_temp_mgr
    }

    /// Get a reference to the templates mgr's mession temp mgr.
    pub fn mission_temp_mgr(&self) -> &MissionTempMgr {
        &self.mission_temp_mgr
    }
}

pub fn init_temps_mgr(path: &str) -> TemplatesMgr {
    let mut temps_mgr = TemplatesMgr::default();
    read_templates_from_dir(path, temps_mgr.borrow_mut()).unwrap();
    temps_mgr
}

///读取配置文件
fn read_templates_from_dir<P: AsRef<Path>>(
    path: P,
    temps_mgr: &mut TemplatesMgr,
) -> anyhow::Result<()> {
    // Open the file in read-only mode with buffer.
    let result = std::fs::read_dir(path)?;
    for f in result {
        let file = f.unwrap();
        let name = file.file_name();
        if name.eq(".DS_Store") {
            continue;
        }
        let mut str = String::new();
        str.push_str(file.path().parent().unwrap().to_str().unwrap().borrow());
        str.push_str("/");
        str.push_str(name.to_str().unwrap());
        let file = File::open(str)?;
        let mut reader = BufReader::new(file);
        let mut context = String::new();
        reader.read_line(&mut context)?;
        let mut name = name.to_str().unwrap().to_string();
        let beta_offset = name.find('.').unwrap_or(name.len());
        name.replace_range(beta_offset.., "");
        init_temps(temps_mgr, name, context.as_str());
    }
    Ok(())
}

fn init_temps(temps_mgr: &mut TemplatesMgr, name: String, context: &str) {
    if name.eq_ignore_ascii_case(TILE_MAP_TEMPLATE) {
        let v: Vec<TileMapTemp> = serde_json::from_str(context).unwrap();
        temps_mgr.tile_map_temp_mgr.init(v);
    } else if name.eq_ignore_ascii_case(CHARACTER_TEMPLATE) {
        let v: Vec<CharacterTemp> = serde_json::from_str(context).unwrap();
        temps_mgr.character_temp_mgr.init(v);
    } else if name.eq_ignore_ascii_case(EMOJI_TEMPLATE) {
        let v: Vec<EmojiTemp> = serde_json::from_str(context).unwrap();
        temps_mgr.emoji_temp_mgr.init(v);
    } else if name.eq_ignore_ascii_case(CONSTANT_TEMPLATE) {
        let v: Vec<ConstantTemp> = serde_json::from_str(context).unwrap();
        temps_mgr.constant_temp_mgr.init(v);
    } else if name.eq_ignore_ascii_case(WORLD_CELL_TEMPLATE) {
        let v: Vec<WorldCellTemp> = serde_json::from_str(context).unwrap();
        temps_mgr.world_cell_temp_mgr.init(v);
    } else if name.eq_ignore_ascii_case(CELL_TEMPLATE) {
        let v: Vec<CellTemp> = serde_json::from_str(context).unwrap();
        temps_mgr.cell_temp_mgr.init(v);
    } else if name.eq_ignore_ascii_case(SKILL_TEMPLATE) {
        let v: Vec<SkillTemp> = serde_json::from_str(context).unwrap();
        temps_mgr.skill_temp_mgr.init(v);
    } else if name.eq_ignore_ascii_case(SKILL_SCOPE_TEMPLATE) {
        let v: Vec<SkillScopeTemp> = serde_json::from_str(context).unwrap();
        temps_mgr.skill_scope_temp_mgr.init(v);
    } else if name.eq_ignore_ascii_case(ITEM_TEMPLATE) {
        let v: Vec<ItemTemp> = serde_json::from_str(context).unwrap();
        temps_mgr.item_temp_mgr.init(v);
    } else if name.eq_ignore_ascii_case(SKILL_JUDGE_TEMPLATE) {
        let v: Vec<SkillJudgeTemp> = serde_json::from_str(context).unwrap();
        temps_mgr.skill_judge_temp_mgr.init(v);
    } else if name.eq_ignore_ascii_case(BUFF) {
        let v: Vec<BuffTemp> = serde_json::from_str(context).unwrap();
        temps_mgr.buff_temp_mgr.init(v);
    } else if name.eq_ignore_ascii_case(SEASON) {
        let v: Vec<SeasonTemp> = serde_json::from_str(context).unwrap();
        temps_mgr.season_temp_mgr.init(v);
    } else if name.eq_ignore_ascii_case(ROBOT) {
        let v: Vec<RobotTemp> = serde_json::from_str(context).unwrap();
        temps_mgr.robot_temp_mgr.init(v);
    } else if name.eq_ignore_ascii_case(LEAGUE) {
        let v: Vec<LeagueTemp> = serde_json::from_str(context).unwrap();
        temps_mgr.league_temp_mgr.init(v);
    } else if name.eq_ignore_ascii_case(SUMMARY_AWARD) {
        let v: Vec<SummaryAwardTemp> = serde_json::from_str(context).unwrap();
        temps_mgr.summary_award_temp_mgr.init(v);
    } else if name.eq_ignore_ascii_case(BATTLE_LIMIT_TIME) {
        let v: Vec<BattleLimitTimeTemp> = serde_json::from_str(context).unwrap();
        temps_mgr.battle_limit_time_temp_mgr.init(v);
    } else if name.eq_ignore_ascii_case(PUNISH) {
        let v: Vec<PunishTemp> = serde_json::from_str(context).unwrap();
        temps_mgr.punish_temp_mgr.init(v);
    } else if name.eq_ignore_ascii_case(GRADE_FRAME) {
        let v: Vec<GradeFrameTemp> = serde_json::from_str(context).unwrap();
        temps_mgr.grade_frame_temp_mgr.init(v);
    } else if name.eq_ignore_ascii_case(SOUL) {
        let v: Vec<SoulTemp> = serde_json::from_str(context).unwrap();
        temps_mgr.soul_temp_mgr.init(v);
    } else if name.eq_ignore_ascii_case(MARKET) {
        let v: Vec<MarketTemp> = serde_json::from_str(context).unwrap();
        temps_mgr.market_temp_mgr.init(v);
    } else if name.eq_ignore_ascii_case(MERCHANDISE) {
        let v: Vec<MerchandiseTemp> = serde_json::from_str(context).unwrap();
        temps_mgr.merchandise_temp_mgr.init(v);
    } else if name.eq_ignore_ascii_case(MISSION) {
        let v: Vec<MissionTemp> = serde_json::from_str(context).unwrap();
        temps_mgr.mission_temp_mgr.init(v);
    }
}
