use crate::templates::buff_temp::{BuffTemp, BuffTempMgr};
use crate::templates::cell_temp::{CellTemp, CellTempMgr};
use crate::templates::character_temp::{CharacterTemp, CharacterTempMgr};
use crate::templates::constant_temp::{ConstantTemp, ConstantTempMgr};
use crate::templates::emoji_temp::{EmojiTemp, EmojiTempMgr};
use crate::templates::item_temp::{ItemTemp, ItemTempMgr};
use crate::templates::robot_temp::{RobotTemp, RobotTempMgr};
use crate::templates::season_temp::{SeasonTemp, SeasonTempMgr};
use crate::templates::skill_judge_temp::{SkillJudgeTemp, SkillJudgeTempMgr};
use crate::templates::skill_scope_temp::{SkillScopeTemp, SkillScopeTempMgr};
use crate::templates::skill_temp::{SkillTemp, SkillTempMgr};
use crate::templates::template_name_constants::{
    BUFF, CELL_TEMPLATE, CHARACTER_TEMPLATE, CONSTANT_TEMPLATE, EMOJI_TEMPLATE, ITEM_TEMPLATE,
    ROBOT, SEASON, SKILL_JUDGE_TEMPLATE, SKILL_SCOPE_TEMPLATE, SKILL_TEMPLATE, TILE_MAP_TEMPLATE,
    WORLD_CELL_TEMPLATE,
};
use crate::templates::tile_map_temp::{TileMapTemp, TileMapTempMgr};
use crate::templates::world_cell_temp::{WorldCellTemp, WorldCellTempMgr};
use std::borrow::Borrow;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub trait Template {}

pub trait TemplateMgrTrait: Send + Sync {
    fn is_empty(&self) -> bool;
}

//配置表mgr
#[derive(Debug, Default)]
pub struct TemplatesMgr {
    character_temp_mgr: CharacterTempMgr,    //角色配置mgr
    tile_map_temp_mgr: TileMapTempMgr,       //地图配置mgr
    emoji_temp_mgr: EmojiTempMgr,            //表情配置mgr
    constant_temp_mgr: ConstantTempMgr,      //常量配置mgr
    world_cell_temp_mgr: WorldCellTempMgr,   //worldcell配置mgr
    cell_temp_mgr: CellTempMgr,              //cell配置mgr
    skill_temp_mgr: SkillTempMgr,            //技能配置mgr
    item_temp_mgr: ItemTempMgr,              //道具配置mgr
    skill_scope_temp_mgr: SkillScopeTempMgr, //技能范围配置mgr
    buff_temp_mgr: BuffTempMgr,              //buff配置mgr
    skill_judge_temp_mgr: SkillJudgeTempMgr, //判定条件配置mgr
    season_temp_mgr: SeasonTempMgr,          //赛季配置mgr
    robot_temp_mgr: RobotTempMgr,            //机器人配置mgr
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
}

pub fn init_temps_mgr(path: &str) -> TemplatesMgr {
    let res = read_templates_from_dir(path).unwrap();
    res
}

///读取配置文件
fn read_templates_from_dir<P: AsRef<Path>>(path: P) -> Result<TemplatesMgr, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let result = std::fs::read_dir(path)?;
    let mut temps_mgr = TemplatesMgr::default();
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
            temps_mgr.tile_map_temp_mgr = TileMapTempMgr::default();
            temps_mgr.tile_map_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(CHARACTER_TEMPLATE) {
            let v: Vec<CharacterTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.character_temp_mgr = CharacterTempMgr::default();
            temps_mgr.character_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(EMOJI_TEMPLATE) {
            let v: Vec<EmojiTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.emoji_temp_mgr = EmojiTempMgr::default();
            temps_mgr.emoji_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(CONSTANT_TEMPLATE) {
            let v: Vec<ConstantTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.constant_temp_mgr = ConstantTempMgr::default();
            temps_mgr.constant_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(WORLD_CELL_TEMPLATE) {
            let v: Vec<WorldCellTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.world_cell_temp_mgr = WorldCellTempMgr::default();
            temps_mgr.world_cell_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(CELL_TEMPLATE) {
            let v: Vec<CellTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.cell_temp_mgr = CellTempMgr::default();
            temps_mgr.cell_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(SKILL_TEMPLATE) {
            let v: Vec<SkillTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.skill_temp_mgr = SkillTempMgr::default();
            temps_mgr.skill_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(SKILL_SCOPE_TEMPLATE) {
            let v: Vec<SkillScopeTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.skill_scope_temp_mgr = SkillScopeTempMgr::default();
            temps_mgr.skill_scope_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(ITEM_TEMPLATE) {
            let v: Vec<ItemTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.item_temp_mgr = ItemTempMgr::default();
            temps_mgr.item_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(SKILL_JUDGE_TEMPLATE) {
            let v: Vec<SkillJudgeTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.skill_judge_temp_mgr = SkillJudgeTempMgr::default();
            temps_mgr.skill_judge_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(BUFF) {
            let v: Vec<BuffTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.buff_temp_mgr = BuffTempMgr::default();
            temps_mgr.buff_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(SEASON) {
            let v: Vec<SeasonTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.season_temp_mgr = SeasonTempMgr::default();
            temps_mgr.season_temp_mgr.init(v);
        } else if name.eq_ignore_ascii_case(ROBOT) {
            let v: Vec<RobotTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.robot_temp_mgr = RobotTempMgr::default();
            temps_mgr.robot_temp_mgr.init(v);
        }
    }
    Ok(temps_mgr)
}
