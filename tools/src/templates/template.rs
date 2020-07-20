use crate::templates::character_temp::{CharacterTemp, CharacterTempMgr};
use crate::templates::tile_map_temp::{TileMapTemp, TileMapTempMgr};
use std::borrow::Borrow;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use crate::templates::template_name_constants::{TILE_MAP_TEMPLATE, CHARACTER_TEMPLATE, EMOJI_TEMPLATE, CONSTANT_TEMPLATE, WORLD_CELL_TEMPLATE, CELL_TEMPLATE, SKILL_TEMPLATE, SKILL_SCOPE_TEMPLATE, ITEM_TEMPLATE, SKILL_JUDGE_TEMPLATE, TRIGGER_TIME_TEMPLATE};
use crate::templates::emoji_temp::{EmojiTempMgr, EmojiTemp};
use crate::templates::constant_temp::{ConstantTempMgr,ConstantTemp};
use crate::templates::world_cell_temp::{WorldCellTempMgr, WorldCellTemp};
use crate::templates::cell_temp::{CellTempMgr, CellTemp};
use crate::templates::skill_temp::{SkillTempMgr, SkillTemp};
use crate::templates::item_temp::{ItemTempMgr, ItemTemp};
use crate::templates::skill_scope_temp::{SkillScopeTempMgr, SkillScopeTemp};
use crate::templates::trigger_time_temp::{TriggerTimeTempMgr, TriggerTimeTemp};
use crate::templates::skill_judge_temp::{SkillJudgeTempMgr, SkillJudgeTemp};

pub trait Template {}

pub trait TemplateMgrTrait: Send + Sync {
    fn is_empty(&self) -> bool;
}

//配置表mgr
#[derive(Debug, Default)]
pub struct TemplatesMgr {
    character_temp_mgr: CharacterTempMgr,//角色配置mgr
    tile_map_temp_mgr: TileMapTempMgr,//地图配置mgr
    emoji_temp_mgr:EmojiTempMgr,//表情配置mgr
    constant_temp_mgr:ConstantTempMgr,//常量配置mgr
    world_cell_temp_mgr:WorldCellTempMgr,//worldcell配置mgr
    cell_temp_mgr:CellTempMgr,//cell配置mgr
    skill_temp_mgr:SkillTempMgr,//技能配置mgr
    item_temp_mgr:ItemTempMgr,//道具配置mgr
    skill_scope_temp_mgr:SkillScopeTempMgr,//技能范围配置mgr
    trigger_time_temp_mgr:TriggerTimeTempMgr,//触发条件配置mgr
    skill_judge_temp_mgr:SkillJudgeTempMgr,//判定条件配置mgr
}

impl TemplatesMgr {
    pub fn execute_init(&self){
        self.get_constant_ref();
    }

    pub fn get_character_ref(&self) -> &CharacterTempMgr {
        self.character_temp_mgr.borrow()
    }

    pub fn get_tile_map_ref(&self) -> &TileMapTempMgr {
        self.tile_map_temp_mgr.borrow()
    }

    pub fn get_emoji_ref(&self) -> &EmojiTempMgr {
        self.emoji_temp_mgr.borrow()
    }

    pub fn get_constant_ref(&self) -> &ConstantTempMgr {
        self.constant_temp_mgr.borrow()
    }

    pub fn get_world_cell_ref(&self) -> &WorldCellTempMgr {
        self.world_cell_temp_mgr.borrow()
    }

    pub fn get_cell_ref(&self) -> &CellTempMgr {
        self.cell_temp_mgr.borrow()
    }

    pub fn get_skill_ref(&self) -> &SkillTempMgr {
        self.skill_temp_mgr.borrow()
    }

    pub fn get_skill_scope_ref(&self) -> &SkillScopeTempMgr { self.skill_scope_temp_mgr.borrow() }

    pub fn get_item_ref(&self) -> &ItemTempMgr {
        self.item_temp_mgr.borrow()
    }

    pub fn get_trigger_time_ref(&self) -> &TriggerTimeTempMgr {
        self.trigger_time_temp_mgr.borrow()
    }

    pub fn get_skill_judge_ref(&self) -> &SkillJudgeTempMgr {
        self.skill_judge_temp_mgr.borrow()
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
        if name.eq(".DS_Store"){
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
        }else if name.eq_ignore_ascii_case(EMOJI_TEMPLATE) {
            let v: Vec<EmojiTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.emoji_temp_mgr = EmojiTempMgr::default();
            temps_mgr.emoji_temp_mgr.init(v);
        }else if name.eq_ignore_ascii_case(CONSTANT_TEMPLATE) {
            let v: Vec<ConstantTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.constant_temp_mgr = ConstantTempMgr::default();
            temps_mgr.constant_temp_mgr.init(v);
        }else if name.eq_ignore_ascii_case(WORLD_CELL_TEMPLATE) {
            let v: Vec<WorldCellTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.world_cell_temp_mgr = WorldCellTempMgr::default();
            temps_mgr.world_cell_temp_mgr.init(v);
        }else if name.eq_ignore_ascii_case(CELL_TEMPLATE) {
            let v: Vec<CellTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.cell_temp_mgr = CellTempMgr::default();
            temps_mgr.cell_temp_mgr.init(v);
        }else if name.eq_ignore_ascii_case(SKILL_TEMPLATE) {
            let v: Vec<SkillTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.skill_temp_mgr = SkillTempMgr::default();
            temps_mgr.skill_temp_mgr.init(v);
        }else if name.eq_ignore_ascii_case(SKILL_SCOPE_TEMPLATE) {
           let v: Vec<SkillScopeTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.skill_scope_temp_mgr = SkillScopeTempMgr::default();
            temps_mgr.skill_scope_temp_mgr.init(v);
        }else if name.eq_ignore_ascii_case(ITEM_TEMPLATE) {
            let v: Vec<ItemTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.item_temp_mgr = ItemTempMgr::default();
            temps_mgr.item_temp_mgr.init(v);
        }else if name.eq_ignore_ascii_case(SKILL_JUDGE_TEMPLATE) {
            let v: Vec<SkillJudgeTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.skill_judge_temp_mgr = SkillJudgeTempMgr::default();
            temps_mgr.skill_judge_temp_mgr.init(v);
        }else if name.eq_ignore_ascii_case(TRIGGER_TIME_TEMPLATE) {
            let v: Vec<TriggerTimeTemp> = serde_json::from_str(string.as_ref()).unwrap();
            temps_mgr.trigger_time_temp_mgr = TriggerTimeTempMgr::default();
            temps_mgr.trigger_time_temp_mgr.init(v);
        }
    }
    Ok(temps_mgr)
}
