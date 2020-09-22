/** 添加菜单选项 **/
INSERT INTO menus (parent_id, name, level_id, state, url, is_show) VALUES 
(0, '系统管理', 0, 1, '#', 1);
set @last_id = LAST_INSERT_ID();
set @parent_id = @last_id;
UPDATE admin_roles SET menu_ids = concat(menu_ids, ',', @last_id) WHERE id = 1;

INSERT INTO menus (parent_id, name, level_id, state, url, is_show) VALUES 
(@parent_id, '网站设置', 1, 1, '/configs/edit/1', 1);
set @last_id = LAST_INSERT_ID();
UPDATE admin_roles SET menu_ids = concat(menu_ids, ',', @last_id) WHERE id = 1;

INSERT INTO menus (parent_id, name, level_id, state, url, is_show) VALUES 
(@parent_id, '网站设置保存', 1, 1, '/configs/save/1', 0);
set @last_id = LAST_INSERT_ID();
UPDATE admin_roles SET menu_ids = concat(menu_ids, ',', @last_id) WHERE id = 1;

INSERT INTO menus (parent_id, name, level_id, state, url, is_show) VALUES 
(@parent_id, '网站导航', 1, 1, '/navs', 1);
set @last_id = LAST_INSERT_ID();
UPDATE admin_roles SET menu_ids = concat(menu_ids, ',', @last_id) WHERE id = 1;

INSERT INTO menus (parent_id, name, level_id, state, url, is_show) VALUES 
(@parent_id, '网站导航编辑', 1, 1, '/navs/edit/\\d+|/navs/save/\\d+', 0);
set @last_id = LAST_INSERT_ID();
UPDATE admin_roles SET menu_ids = concat(menu_ids, ',', @last_id) WHERE id = 1;

INSERT INTO menus (parent_id, name, level_id, state, url, is_show) VALUES 
(@parent_id, '网站导航删除', 1, 1, '/navs/delete/\\d+', 0);
set @last_id = LAST_INSERT_ID();
UPDATE admin_roles SET menu_ids = concat(menu_ids, ',', @last_id) WHERE id = 1;

/** 导航 **/
DROP TABLE IF EXISTS navs;
CREATE TABLE IF NOT EXISTS navs (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    name VARCHAR(20) NOT NULL DEFAULT '' COMMENT '名称',
    url VARCHAR(200) NOT NULL DEFAULT '' COMMENT '链接地址',
    is_blank TINYINT UNSIGNED NOT NULL DEFAULT 0 COMMENT '是否外链',
    remark VARCHAR(100) NOT NULL DEFAULT '' COMMENT '说明',
    seq INT NOT NULL DEFAULT 0 COMMENT '排序',
    PRIMARY KEY(id)
) ENGINE=INNODB DEFAULT CHARSET=UTF8 COLLATE=UTF8_GENERAL_CI;

/** 系统配置 **/
DROP TABLE IF EXISTS configs;
CREATE TABLE IF NOT EXISTS configs (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    site_name VARCHAR(50) NOT NULL DEFAULT '' COMMENT '站点名称',
    site_url VARCHAR(200) NOT NULL DEFAULT '' COMMENT '主页地址',
    seo_keyword VARCHAR(250) NOT NULL DEFAULT '' COMMENT 'SEO关键字',
    seo_desc VARCHAR(250) NOT NULL DEFAULT '' COMMENT 'SEO描述',
    copyright VARCHAR(200) NOT NULL DEFAULT '' COMMENT '版权',
    PRIMARY KEY(id)
) ENGINE=INNODB DEFAULT CHARSET=UTF8 COLLATE=UTF8_GENERAL_CI;
INSERT INTO configs (site_name, site_url, seo_keyword, seo_desc, copyright) VALUES 
('网站名称', 'http://site.cn/', '用于SEO的网站关键字', '用于SEO的网站描述', '网站版权信息');


TRUNCATE TABLE navs;

INSERT INTO navs (name, url, seq) VALUES 
('网站首页', '/', 9999),
('全部视频', '/videos', 988),
('关于我们', '/about', 800),
('客户留言', '/feedback', 700),
('联系我们', '/contact', 600);

TRUNCATE TABLE video_tags;
INSERT INTO video_tags (name, seq) VALUES 
('剧情', 9990),
('喜剧', 9980),
('动作', 9970),
('爱情', 9960),
('科幻', 9950),
('悬疑', 9940),
('惊悚', 9930),
('恐怖', 9920),
('犯罪', 9910),
('同性', 9890),
('音乐', 9880),
('歌舞', 9870),
('传记', 9860),
('历史', 9840),
('战争', 9830),
('西部', 9820),
('奇幻', 9810),
('冒险', 9790),
('灾难', 9780),
('武侠', 9770),
('情色', 9760),
('中国大陆', 8990),
('美国', 8980),
('香港', 8970),
('台湾', 8960),
('日本', 8950),
('韩国', 8940),
('英国', 8930),
('法国', 8920),
('德国', 8910),
('意大利', 8900),
('西班牙', 8890),
('印度', 8880),
('泰国', 8870),
('俄罗斯', 8860),
('加拿大', 8850),
('澳大利亚', 8840),
('瑞典', 8830),
('巴西', 8820),
('丹麦', 8810),
('其他', 8800),
('2019', 7990),
('2018', 7980),
('2017', 7970),
('2016', 7960),
('2015', 7950),
('2014', 7940),
('2013', 7930),
('2012', 7920),
('2011', 7910),
('2010', 7900),
('2009', 7890),
('2008', 7880),
('2007', 7870),
('2006', 7860),
('2005', 7850),
('2004', 7840),
('2003', 7830),
('2002', 7820),
('2001', 7810),
('2000', 7800),
('更早', 7790);

/** 添加 videos 相关字段 **/
ALTER TABLE videos ADD COLUMN category_id INT UNSIGNED DEFAULT 0 COMMENT '分类编号';
ALTER TABLE videos ADD COLUMN tag_ids VARCHAR(500) NOT NULL DEFAULT '' COMMENT '标签编号';
ALTER TABLE videos ADD COLUMN author_id INT UNSIGNED DEFAULT 0 COMMENT '作者编号';
ALTER TABLE videos ADD COLUMN url VARCHAR(200) NOT NULL DEFUALT '' COMMENT '播放地址';

/** 视频作者 **/
DROP TABLE IF EXISTS video_authors;
CREATE TABLE IF NOT EXISTS video_authors (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    name VARCHAR(20) NOT NULL DEFAULT '' COMMENT '名称',
    remark VARCHAR(500) NOT NULL DEFAULT '' COMMENT '备注',
    seq INT NOT NULL DEFAULT 0 COMMENT '排序',
    PRIMARY KEY(id)
) ENGINE=INNODB ENGINE=INNODB DEFAULT CHARSET=UTF8 COLLATE=UTF8_GENERAL_CI;
INSERT INTO video_authors (name, remark) VALUES 
('默认', '默认');

/** 添加菜单选项 **/
set @parent_id = 2; /* 内容管理 */
INSERT INTO menus (parent_id, name, level_id, state, url, is_show) VALUES 
(@parent_id, '视频作者', 1, 1, '/video_authors', 1);
set @last_id = LAST_INSERT_ID();
UPDATE admin_roles SET menu_ids = concat(menu_ids, ',', @last_id) WHERE id = 1;

INSERT INTO menus (parent_id, name, level_id, state, url, is_show) VALUES 
(@parent_id, '视频作者编辑', 1, 1, '/video_authors/edit/\\d+|/video_authors/save/\\d+', 0);
set @last_id = LAST_INSERT_ID();
UPDATE admin_roles SET menu_ids = concat(menu_ids, ',', @last_id) WHERE id = 1;

INSERT INTO menus (parent_id, name, level_id, state, url, is_show) VALUES 
(@parent_id, '视频作者删除', 1, 1, '/video_authors/delete/\\d+', 0);
set @last_id = LAST_INSERT_ID();
UPDATE admin_roles SET menu_ids = concat(menu_ids, ',', @last_id) WHERE id = 1;

ALTER TABLE videos ADD COLUMN is_recommended TINYINT NOT NULL DEFAULT 0 COMMENT '是否推荐';
