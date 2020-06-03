use crate::templates::character_temp::{CharacterTemp, CharacterTempMgr};
use crate::templates::tile_map_temp::{TileMapTemp, TileMapTempMgr};
use std::borrow::Borrow;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use crate::templates::template_contants::{TILE_MAP_TEMPLATE, CHARACTER_TEMPLATE};

pub trait Template {}

pub trait TemplateMgrTrait: Send + Sync {
    fn is_empty(&self) -> bool;
}

#[derive(Debug, Default)]
pub struct TemplatesMgr {
    character_temp_mgr: CharacterTempMgr,
    tile_map_temp_mgr: TileMapTempMgr,
}

impl TemplatesMgr {
    pub fn get_character_ref(&self) -> &CharacterTempMgr {
        self.character_temp_mgr.borrow()
    }

    pub fn get_tile_map_ref(&self) -> &TileMapTempMgr {
        self.tile_map_temp_mgr.borrow()
    }
}

pub fn init_temps(path: &str) -> TemplatesMgr {
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
        }
    }
    Ok(temps_mgr)
}
