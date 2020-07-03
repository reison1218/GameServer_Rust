use tools::protos::base::{CellPt, TileMapPt};
use tools::templates::tile_map_temp::TileMapTemp;

///地图
#[derive(Debug, Default, Clone)]
pub struct TileMap {
    pub id: u32,
    pub cell_array: Vec<Cell>,
}

///单元格数据
#[derive(Debug, Default, Copy, Clone)]
pub struct Cell {
    pub value: u32,
    pub value_type: u8,
}

impl TileMap {
    #[warn(dead_code)]
    pub fn new(temp: &TileMapTemp) -> anyhow::Result<Self> {
        let id = temp.id;
        let map = temp.map.clone();
        let mut cell_array = Vec::new();
        for v in map {
            let mut cell = Cell::default();
            cell.value = v;
            cell.value_type = 0;
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
