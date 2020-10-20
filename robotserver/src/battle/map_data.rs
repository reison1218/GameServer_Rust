use crate::battle::battle::Buff;
use std::collections::HashMap;

///地图
#[derive(Debug, Default, Clone)]
pub struct TileMap {
    pub id: u32,                                   //地图id
    pub map_cells: [MapCell; 30],                  //地图格子vec
    pub coord_map: HashMap<(isize, isize), usize>, //坐标对应格子
    pub world_cell_map: HashMap<u32, u32>,         //世界块map，index，map_cellid
    pub un_pair_map: HashMap<usize, u32>,          //未配对的地图块map
}

///块的封装结构体
#[derive(Debug, Default, Clone)]
pub struct MapCell {
    pub id: u32,                   //块的配置id
    pub index: usize,              //块的下标
    pub buffs: HashMap<u32, Buff>, //块的效果
    pub is_world: bool,            //是否世界块
    pub element: u8,               //地图块的属性
    pub passive_buffs: Vec<u32>,   //额外玩家对其添加的buff
    pub user_id: u32,              //这个地图块上面的玩家
    pub pair_index: Option<usize>, //与之配对的下标
    pub x: isize,                  //x轴坐标
    pub y: isize,                  //y轴坐标
}
