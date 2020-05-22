use super::*;
use serde_json::{Map, Value};
use tools::protos::base::{CellPt, TileMapPt};

///地图
#[derive(Debug, Default, Clone)]
pub struct TileMap {
    pub id: u64,
    pub cell_array: Vec<Cell>,
}

///单元格数据
#[derive(Debug, Default, Copy, Clone)]
pub struct Cell {
    pub value: u32,
    pub value_type: u8,
}

impl TileMap {
    pub fn new(json: &Value) -> Result<Self, String> {
        let res = json.as_object();
        if res.is_none() {
            let s = format!("could not convert JsonValue to Map<String,JsonValue>");
            error!("{}", s.as_str());
            return Err(s);
        }
        let res = res.unwrap();

        let id = res.get("ID");
        if id.is_none() {
            let s = format!("ID is None in TileMap!");
            error!("{}", s.as_str());
            return Err(s);
        }
        let id = id.unwrap().as_u64().unwrap();

        let map = json.get("Map");
        if map.is_none() {
            let s = format!("Map is None in TileMap,ID:{}", id);
            error!("{}", s.as_str());
            return Err(s);
        }
        let map = map.unwrap();
        let map = map.as_array();
        if map.is_none() {
            let s = format!("Map is not an array in TileMap!ID:{}", id);
            error!("{}", s.as_str());
            return Err(s);
        }
        let map = map.unwrap();

        let mut cell_array = Vec::new();
        for v in map {
            let cell_id = v.as_u64();
            if cell_id.is_none() {
                let s = format!("Map'value is not number in TileMap!ID:{}", id);
                error!("{}", s.as_str());
                return Err(s);
            }
            let cell = Cell {
                value: cell_id.unwrap() as u32,
                value_type: 0,
            };
            cell_array.push(cell);
        }
        Ok(TileMap { id, cell_array })
    }

    pub fn convert_pt(&self) -> TileMapPt {
        let mut tp = TileMapPt::new();
        tp.id = self.id;

        let mut v = Vec::new();
        for i in self.cell_array.iter() {
            let mut cp = CellPt::new();
            cp.set_field_type(i.value_type as u32);
            cp.set_value(i.value);
            v.push(cp);
        }
        let res = protobuf::RepeatedField::from(v);
        tp.set_cell_array(res);
        tp
    }
}
