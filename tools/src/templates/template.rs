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
}

impl TemplatesMgr {
    pub fn execute_init(&self) {
        self.get_constant_temp_mgr_ref();
    }

    pub fn get_character_temp_mgr_ref(&self) -> &CharacterTempMgr {
        self.character_temp_mgr.borrow()
    }

    pub fn get_tile_map_temp_mgr_ref(&self) -> &TileMapTempMgr {
        self.tile_map_temp_mgr.borrow()
    }

    pub fn get_emoji_temp_mgr_ref(&self) -> &EmojiTempMgr {
        self.emoji_temp_mgr.borrow()
    }

    pub fn get_constant_temp_mgr_ref(&self) -> &ConstantTempMgr {
        self.constant_temp_mgr.borrow()
    }

    pub fn get_world_cell_temp_mgr_ref(&self) -> &WorldCellTempMgr {
        self.world_cell_temp_mgr.borrow()
    }

    pub fn get_cell_temp_mgr_ref(&self) -> &CellTempMgr {
        self.cell_temp_mgr.borrow()
    }

    pub fn get_skill_temp_mgr_ref(&self) -> &SkillTempMgr {
        self.skill_temp_mgr.borrow()
    }

    pub fn get_skill_scope_temp_mgr_ref(&self) -> &SkillScopeTempMgr {
        self.skill_scope_temp_mgr.borrow()
    }

    pub fn get_item_temp_mgr_ref(&self) -> &ItemTempMgr {
        self.item_temp_mgr.borrow()
    }

    pub fn get_buff_temp_mgr_ref(&self) -> &BuffTempMgr {
        self.buff_temp_mgr.borrow()
    }

    pub fn get_skill_judge_temp_mgr_ref(&self) -> &SkillJudgeTempMgr {
        self.skill_judge_temp_mgr.borrow()
    }

    pub fn get_season_temp_mgr_ref(&self) -> &SeasonTempMgr {
        self.season_temp_mgr.borrow()
    }

    pub fn get_robot_temp_mgr_ref(&self) -> &RobotTempMgr {
        self.robot_temp_mgr.borrow()
    }

    pub fn get_league_temp_mgr_ref(&self) -> &LeagueTempMgr {
        self.league_temp_mgr.borrow()
    }

    pub fn get_summary_award_temp_mgr_ref(&self) -> &SummaryAwardTempMgr {
        self.summary_award_temp_mgr.borrow()
    }

    pub fn get_battle_limit_time_temp_mgr_ref(&self) -> &BattleLimitTimeTempMgr {
        self.battle_limit_time_temp_mgr.borrow()
    }

    pub fn get_punish_temp_mgr_ref(&self) -> &PunishTempMgr {
        self.punish_temp_mgr.borrow()
    }

    pub fn get_grade_frame_temp_mgr_ref(&self) -> &GradeFrameTempMgr {
        self.grade_frame_temp_mgr.borrow()
    }

    pub fn get_soul_temp_mgr_ref(&self) -> &SoulTempMgr {
        self.soul_temp_mgr.borrow()
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
            let res = read_templates_from_dir(path, mgr_mut);
            if let Err(e) = res {
                error!("{:?}", e);
                return Ok(());
            }
        }
        Ok(())
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
        let mut string = String::new();
        reader.read_line(&mut string)?;
        let mut name = name.to_str().unwrap().to_string();
        let beta_offset = name.find('.').unwrap_or(name.len());
        name.replace_range(beta_offset.., "");

        if name.eq_ignore_ascii_case(TILE_MAP_TEMPLATE) {
            let v: Vec<TileMapTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.tile_map_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(CHARACTER_TEMPLATE) {
            let v: Vec<CharacterTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.character_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(EMOJI_TEMPLATE) {
            let v: Vec<EmojiTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.emoji_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(CONSTANT_TEMPLATE) {
            let v: Vec<ConstantTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.constant_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(WORLD_CELL_TEMPLATE) {
            let v: Vec<WorldCellTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.world_cell_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(CELL_TEMPLATE) {
            let v: Vec<CellTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.cell_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(SKILL_TEMPLATE) {
            let v: Vec<SkillTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.skill_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(SKILL_SCOPE_TEMPLATE) {
            let v: Vec<SkillScopeTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.skill_scope_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(ITEM_TEMPLATE) {
            let v: Vec<ItemTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.item_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(SKILL_JUDGE_TEMPLATE) {
            let v: Vec<SkillJudgeTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.skill_judge_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(BUFF) {
            let v: Vec<BuffTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.buff_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(SEASON) {
            let v: Vec<SeasonTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.season_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(ROBOT) {
            let v: Vec<RobotTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.robot_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(LEAGUE) {
            let v: Vec<LeagueTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.league_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(SUMMARY_AWARD) {
            let v: Vec<SummaryAwardTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.summary_award_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(BATTLE_LIMIT_TIME) {
            let v: Vec<BattleLimitTimeTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.battle_limit_time_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(PUNISH) {
            let v: Vec<PunishTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.punish_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(GRADE_FRAME) {
            let v: Vec<GradeFrameTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.grade_frame_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(SOUL) {
            let v: Vec<SoulTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.soul_temp_mgr.init(v);
        }
    }
    Ok(())
}
