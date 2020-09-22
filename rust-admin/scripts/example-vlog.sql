-- MySQL dump 10.13  Distrib 5.7.28, for Linux (x86_64)
--
-- Host: localhost    Database: rust_admin
-- ------------------------------------------------------
-- Server version	5.7.28-0ubuntu0.18.04.4

/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!40101 SET NAMES utf8 */;
/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;

--
-- Table structure for table `admin_roles`
--

DROP TABLE IF EXISTS `admin_roles`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `admin_roles` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(20) NOT NULL DEFAULT '' COMMENT '角色名称',
  `remark` varchar(50) NOT NULL DEFAULT '' COMMENT '备注',
  `menu_ids` text COMMENT '菜单编号',
  `seq` int(11) NOT NULL DEFAULT '0' COMMENT '排序',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `admin_roles`
--

LOCK TABLES `admin_roles` WRITE;
/*!40000 ALTER TABLE `admin_roles` DISABLE KEYS */;
INSERT INTO `admin_roles` VALUES (1,'系统管理员','后台用户管理','1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,33,34,35,25,26,27,28,29,30,31,32,36,37,38,39,40,41,42,43,44',0);
/*!40000 ALTER TABLE `admin_roles` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `admins`
--

DROP TABLE IF EXISTS `admins`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `admins` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(20) NOT NULL DEFAULT '' COMMENT '登錄名稱',
  `password` char(32) NOT NULL DEFAULT '' COMMENT '登錄密碼',
  `secret` char(32) NOT NULL DEFAULT '' COMMENT '密钥',
  `last_ip` varchar(32) NOT NULL DEFAULT '' COMMENT '最後IP',
  `state` tinyint(3) unsigned NOT NULL DEFAULT '0' COMMENT '狀態',
  `login_count` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '登錄次數',
  `last_login` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '上次登錄時間',
  `role_id` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '角色编号',
  `created` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '創建時間',
  `updated` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '修改時間',
  `seq` int(11) NOT NULL DEFAULT '0' COMMENT '排序',
  PRIMARY KEY (`id`),
  KEY `name` (`name`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `admins`
--

LOCK TABLES `admins` WRITE;
/*!40000 ALTER TABLE `admins` DISABLE KEYS */;
INSERT INTO `admins` VALUES (1,'admin','a3b12db078cc790812c5e70220cd46b1','BJbZEngUhaBO22MrlEQ2STGqvy1bxr5k','127.0.0.1',1,3,1580886502,1,1580470174,1580886502,0);
/*!40000 ALTER TABLE `admins` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `ads`
--

DROP TABLE IF EXISTS `ads`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `ads` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(20) NOT NULL DEFAULT '' COMMENT '名称',
  `remark` varchar(100) NOT NULL DEFAULT '' COMMENT '备注',
  `image` varchar(200) NOT NULL DEFAULT '' COMMENT '图片地址',
  `page_id` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '页面,0:首页;1:详情页',
  `position_id` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '位置,0:顶部;1:中左;2:中右;3:底部',
  `url` varchar(200) NOT NULL DEFAULT '' COMMENT '链接地址',
  `is_blank` tinyint(3) unsigned NOT NULL DEFAULT '1' COMMENT '是否外链,0:否,1:是',
  `seq` int(11) NOT NULL DEFAULT '0' COMMENT '排序',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `ads`
--

LOCK TABLES `ads` WRITE;
/*!40000 ALTER TABLE `ads` DISABLE KEYS */;
INSERT INTO `ads` VALUES (1,'脚气灵','治脚气一抹就灵','',0,0,'',1,0);
/*!40000 ALTER TABLE `ads` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `configs`
--

DROP TABLE IF EXISTS `configs`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `configs` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `site_name` varchar(50) NOT NULL DEFAULT '' COMMENT '站点名称',
  `site_url` varchar(200) NOT NULL DEFAULT '' COMMENT '主页地址',
  `seo_keyword` varchar(250) NOT NULL DEFAULT '' COMMENT 'SEO关键字',
  `seo_desc` varchar(250) NOT NULL DEFAULT '' COMMENT 'SEO描述',
  `copyright` varchar(200) NOT NULL DEFAULT '' COMMENT '版权',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `configs`
--

LOCK TABLES `configs` WRITE;
/*!40000 ALTER TABLE `configs` DISABLE KEYS */;
INSERT INTO `configs` VALUES (1,'网站名称','http://site.cn/','用于SEO的网站关键字','用于SEO的网站描述','网站版权信息');
/*!40000 ALTER TABLE `configs` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `menus`
--

DROP TABLE IF EXISTS `menus`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `menus` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `parent_id` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '上级编号',
  `name` varchar(50) NOT NULL DEFAULT '' COMMENT '菜单名称',
  `level_id` tinyint(4) NOT NULL DEFAULT '0' COMMENT '级别ID,1:主菜单;2:子菜单',
  `state` tinyint(4) NOT NULL DEFAULT '0' COMMENT '状态,0:隐藏;1:显示',
  `url` varchar(200) NOT NULL DEFAULT '' COMMENT '链接地址',
  `is_blank` tinyint(4) NOT NULL DEFAULT '0' COMMENT '是否外链,0:否,1:是',
  `is_show` tinyint(4) NOT NULL DEFAULT '0' COMMENT '是否显式,0:否,1:是',
  `seq` int(11) NOT NULL DEFAULT '0' COMMENT '排序',
  PRIMARY KEY (`id`),
  KEY `parent_id` (`parent_id`)
) ENGINE=InnoDB AUTO_INCREMENT=48 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `menus`
--

LOCK TABLES `menus` WRITE;
/*!40000 ALTER TABLE `menus` DISABLE KEYS */;
INSERT INTO `menus` VALUES (1,0,'后台管理',0,1,'#',0,1,0),(2,0,'内容管理',0,1,'#',0,1,0),(3,0,'前台管理',0,1,'#',0,1,0),(4,1,'后台用户',1,1,'/admins',0,1,0),(5,1,'后台用户编辑',1,1,'/admins/edit/\\d+|/admins/save/\\d+',0,0,0),(6,1,'后台用户删除',1,1,'/admins/delete/\\d+',0,0,0),(7,1,'菜单列表',1,1,'/menus',0,1,0),(8,1,'菜单添加',1,1,'/menus/edit/\\d+|/menus/save/\\d+',0,0,0),(9,1,'菜单删除',1,1,'/menus/delete/\\d+',0,0,0),(10,1,'后台角色',1,1,'/admin_roles',0,1,0),(11,1,'后台角色编辑',1,1,'/admin_roles/edit/\\d+|/admin_roles/save/\\d+',0,0,0),(12,1,'后台角色删除',1,1,'/admin_roles/\\d+',0,0,0),(13,2,'视频分类',1,1,'/video_categories',0,1,0),(14,2,'视频分类编辑',1,1,'/video_categories/edit/\\d+|/video_categories/save/\\d+',0,0,0),(15,2,'视频分类删除',1,1,'/video_categories/delete/\\d+',0,0,0),(16,2,'视频标签',1,1,'/video_tags',0,1,0),(17,2,'视频标签添加',1,1,'/video_tags/edit/\\d+|/videos/save/\\d+',0,0,0),(18,2,'视频标签删除',1,1,'/video_tags/delete/\\d+',0,0,0),(19,2,'视频管理',1,1,'/videos',0,1,0),(20,2,'视频管理添加',1,1,'/videos/edit/\\d+|/videos/save/\\d+',0,0,0),(21,2,'视频管理删除',1,1,'/videos/delete/\\d+',0,0,0),(22,2,'视频评论',1,1,'/video_replies',0,1,0),(23,2,'视频评论添加',1,1,'/video_replies/edit/\\d+|/video_replies/save/\\d+',0,0,0),(24,2,'视频评论删除',1,1,'/video_replies/delete/\\d+',0,0,0),(25,3,'用户列表',1,1,'/users',0,1,0),(26,3,'用户列表编辑',1,1,'/users/edit/\\d+|/users/save/\\d+',0,0,0),(27,3,'用户列表删除',1,1,'/users/delete/\\d+',0,0,0),(28,3,'用户等级',1,1,'/user_levels',0,1,0),(29,3,'用户等级编辑',1,1,'/user_levels/edit/\\d+|/user_levels/save/\\d+',0,0,0),(30,3,'用户等级删除',1,1,'/user_levels/delete/\\d+',0,0,0),(31,3,'观看记录',1,1,'/watch_records',0,1,0),(32,3,'观看记录',1,1,'/watch_records/delete/\\d+',0,0,0),(33,2,'广告管理',1,1,'/ads',0,1,0),(34,2,'广告管理编辑',1,1,'/ads/edit/\\d+|/ads/save/\\d+',0,0,0),(35,2,'广告管理删除',1,1,'/ads/delete/\\d+',0,0,0),(36,0,'系统管理',0,1,'#',0,1,0),(37,36,'网站设置',1,1,'/configs/edit/1',0,1,0),(38,36,'网站设置保存',1,1,'/configs/save/1',0,0,0),(39,36,'网站导航',1,1,'/navs',0,1,0),(40,36,'网站导航编辑',1,1,'/navs/edit/\\d+|/navs/save/\\d+',0,0,0),(41,36,'网站导航删除',1,1,'/navs/delete/\\d+',0,0,0),(42,2,'视频作者',1,1,'/video_authors',0,1,0),(43,2,'视频作者编辑',1,1,'/video_authors/edit/\\d+|/video_authors/save/\\d+',0,0,0),(44,2,'视频作者删除',1,1,'/video_authors/delete/\\d+',0,0,0);
/*!40000 ALTER TABLE `menus` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `navs`
--

DROP TABLE IF EXISTS `navs`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `navs` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(20) NOT NULL DEFAULT '' COMMENT '名称',
  `url` varchar(200) NOT NULL DEFAULT '' COMMENT '链接地址',
  `is_blank` tinyint(3) unsigned NOT NULL DEFAULT '0' COMMENT '是否外链',
  `remark` varchar(100) NOT NULL DEFAULT '' COMMENT '说明',
  `seq` int(11) NOT NULL DEFAULT '0' COMMENT '排序',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=6 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `navs`
--

LOCK TABLES `navs` WRITE;
/*!40000 ALTER TABLE `navs` DISABLE KEYS */;
INSERT INTO `navs` VALUES (1,'网站首页','/',0,'',9999),(2,'全部视频','/videos',0,'',988),(3,'关于我们','/about',0,'',800),(4,'客户留言','/feedback',0,'',700),(5,'联系我们','/contact',0,'',600);
/*!40000 ALTER TABLE `navs` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `user_levels`
--

DROP TABLE IF EXISTS `user_levels`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `user_levels` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(20) NOT NULL DEFAULT '' COMMENT '等级名称',
  `remark` varchar(100) NOT NULL DEFAULT '' COMMENT '备注',
  `watch_per_day` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '每天观数',
  `score_min` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '最低积分',
  `score_max` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '最高积分',
  `seq` int(11) NOT NULL DEFAULT '0' COMMENT '排序',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=6 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `user_levels`
--

LOCK TABLES `user_levels` WRITE;
/*!40000 ALTER TABLE `user_levels` DISABLE KEYS */;
INSERT INTO `user_levels` VALUES (1,'VIP','基本级VIP',0,0,0,0),(2,'青铜VIP','高等级VIP',0,0,0,0),(3,'白银VIP','高等级VIP',0,0,0,0),(4,'黄金VIP','高等级VIP',0,0,0,0),(5,'钻石VIP','高等级VIP',0,0,0,0);
/*!40000 ALTER TABLE `user_levels` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `users`
--

DROP TABLE IF EXISTS `users`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `users` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(20) NOT NULL DEFAULT '' COMMENT '用户名称',
  `password` char(32) NOT NULL DEFAULT '' COMMENT '登錄密碼',
  `secret` char(32) NOT NULL DEFAULT '' COMMENT '用户密钥',
  `mail` varchar(100) NOT NULL DEFAULT '' COMMENT '电子邮件',
  `last_ip` varchar(32) NOT NULL DEFAULT '' COMMENT '最後IP',
  `state` tinyint(3) unsigned NOT NULL DEFAULT '0' COMMENT '狀態',
  `login_count` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '登錄次數',
  `level_id` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '等级编号',
  `score` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '积分',
  `last_login` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '上次登錄時間',
  `created` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '創建時間',
  `updated` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '修改時間',
  `remark` varchar(200) NOT NULL DEFAULT '' COMMENT '备注',
  PRIMARY KEY (`id`),
  KEY `name` (`name`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `users`
--

LOCK TABLES `users` WRITE;
/*!40000 ALTER TABLE `users` DISABLE KEYS */;
INSERT INTO `users` VALUES (1,'user','200820e3227815ed1756a6b531e7e0d2','','user@gmail.com','',0,0,0,0,0,1580470174,1580470174,'');
/*!40000 ALTER TABLE `users` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `video_authors`
--

DROP TABLE IF EXISTS `video_authors`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `video_authors` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(20) NOT NULL DEFAULT '' COMMENT '名称',
  `remark` varchar(500) DEFAULT '' COMMENT '备注',
  `seq` int(11) NOT NULL DEFAULT '0' COMMENT '排序',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=7 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `video_authors`
--

LOCK TABLES `video_authors` WRITE;
/*!40000 ALTER TABLE `video_authors` DISABLE KEYS */;
INSERT INTO `video_authors` VALUES (1,'默认','默认',0),(2,'唐司令说电影','B站: https://space.bilibili.com/98605231?from=search&seid=7447429227219220401\r\n爱奇艺: https://www.iqiyi.com/u/2322942463/videos',0),(3,'科幻梦工场','用心讲好每一个科幻故事！\r\nB站: https://space.bilibili.com/108425972/',0),(4,'越哥说电影','宇哥只做好看的电影，保留最精华部分，用诙谐幽默的解说娓娓道来，用心为大家奉献有笑料有态度的影视解说。',0),(5,'大聪看电影','从文字撰稿，到剪辑，到发布，都由大聪一人完成。\r\n不追求跑量，只研磨精品。\r\n你们的支持，就是大聪最好的原创动力！',0),(6,'看电影了没','好的电影，改变你的认知 。断更的视频，请订阅频道：补更看电影了没。',0);
/*!40000 ALTER TABLE `video_authors` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `video_categories`
--

DROP TABLE IF EXISTS `video_categories`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `video_categories` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(20) NOT NULL DEFAULT '' COMMENT '名称',
  `remark` varchar(100) NOT NULL DEFAULT '' COMMENT '备注',
  `seq` int(11) NOT NULL DEFAULT '0' COMMENT '排序',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=3 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `video_categories`
--

LOCK TABLES `video_categories` WRITE;
/*!40000 ALTER TABLE `video_categories` DISABLE KEYS */;
INSERT INTO `video_categories` VALUES (1,'电视剧','电视剧',0),(2,'电影','电影',0);
/*!40000 ALTER TABLE `video_categories` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `video_replies`
--

DROP TABLE IF EXISTS `video_replies`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `video_replies` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `video_id` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '视频编号',
  `reply_id` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '评论编号',
  `user_id` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '用户编号',
  `user_name` varchar(200) NOT NULL DEFAULT '' COMMENT '用户名称',
  `content` text COMMENT '内容',
  `seq` int(11) NOT NULL DEFAULT '0' COMMENT '排序',
  `state` tinyint(3) unsigned NOT NULL DEFAULT '0' COMMENT '狀態',
  `created` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '創建時間',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `video_replies`
--

LOCK TABLES `video_replies` WRITE;
/*!40000 ALTER TABLE `video_replies` DISABLE KEYS */;
INSERT INTO `video_replies` VALUES (1,0,0,0,'user','可以, 这波666',0,0,1580470174);
/*!40000 ALTER TABLE `video_replies` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `video_tags`
--

DROP TABLE IF EXISTS `video_tags`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `video_tags` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(20) NOT NULL DEFAULT '' COMMENT '名称',
  `remark` varchar(100) NOT NULL DEFAULT '' COMMENT '备注',
  `seq` int(11) NOT NULL DEFAULT '0' COMMENT '排序',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=63 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `video_tags`
--

LOCK TABLES `video_tags` WRITE;
/*!40000 ALTER TABLE `video_tags` DISABLE KEYS */;
INSERT INTO `video_tags` VALUES (1,'剧情','',9990),(2,'喜剧','',9980),(3,'动作','',9970),(4,'爱情','',9960),(5,'科幻','',9950),(6,'悬疑','',9940),(7,'惊悚','',9930),(8,'恐怖','',9920),(9,'犯罪','',9910),(10,'同性','',9890),(11,'音乐','',9880),(12,'歌舞','',9870),(13,'传记','',9860),(14,'历史','',9840),(15,'战争','',9830),(16,'西部','',9820),(17,'奇幻','',9810),(18,'冒险','',9790),(19,'灾难','',9780),(20,'武侠','',9770),(21,'情色','',9760),(22,'中国大陆','',8990),(23,'美国','',8980),(24,'香港','',8970),(25,'台湾','',8960),(26,'日本','',8950),(27,'韩国','',8940),(28,'英国','',8930),(29,'法国','',8920),(30,'德国','',8910),(31,'意大利','',8900),(32,'西班牙','',8890),(33,'印度','',8880),(34,'泰国','',8870),(35,'俄罗斯','',8860),(36,'加拿大','',8850),(37,'澳大利亚','',8840),(38,'瑞典','',8830),(39,'巴西','',8820),(40,'丹麦','',8810),(41,'其他','',8800),(42,'2019','',7990),(43,'2018','',7980),(44,'2017','',7970),(45,'2016','',7960),(46,'2015','',7950),(47,'2014','',7940),(48,'2013','',7930),(49,'2012','',7920),(50,'2011','',7910),(51,'2010','',7900),(52,'2009','',7890),(53,'2008','',7880),(54,'2007','',7870),(55,'2006','',7860),(56,'2005','',7850),(57,'2004','',7840),(58,'2003','',7830),(59,'2002','',7820),(60,'2001','',7810),(61,'2000','',7800),(62,'更早','',7790);
/*!40000 ALTER TABLE `video_tags` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `videos`
--

DROP TABLE IF EXISTS `videos`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `videos` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `title` varchar(50) NOT NULL DEFAULT '' COMMENT '标题',
  `remark` varchar(100) NOT NULL DEFAULT '' COMMENT '备注',
  `cover_image` varchar(200) NOT NULL DEFAULT '' COMMENT '封面',
  `duration` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '时长(秒)',
  `seq` int(11) NOT NULL DEFAULT '0' COMMENT '排序',
  `state` tinyint(3) unsigned NOT NULL DEFAULT '0' COMMENT '狀態',
  `created` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '創建時間',
  `updated` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '修改時間',
  `content` text COMMENT '内容',
  `category_id` int(10) unsigned DEFAULT '0' COMMENT '分类编号',
  `tag_ids` varchar(500) NOT NULL DEFAULT '' COMMENT '标签编号',
  `author_id` int(10) unsigned DEFAULT '0' COMMENT '作者编号',
  `url` varchar(200) NOT NULL DEFAULT '' COMMENT '观影地址',
  `is_recommended` tinyint(4) NOT NULL DEFAULT '0' COMMENT '是否推荐',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=13 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `videos`
--

LOCK TABLES `videos` WRITE;
/*!40000 ALTER TABLE `videos` DISABLE KEYS */;
INSERT INTO `videos` VALUES (1,'深海之战','科学家发现一种可燃怪鱼，便开始进行改造，结果创造了恐怖的怪物！','/upload/2020/02/05/YpbyOTgJHeYP836D.png',210,0,1,1581335253,1581335253,'贫民窟小伙逆袭带领大军攻破世界头号游戏公司,迎娶白富美',2,'1,10,7',2,'https://www.youtube.com/watch?v=C5IAlShJhhg',1),(2,'流感','一个小小的感冒要死了韩国几十万人','/upload/2020/02/05/YS9XXOcBiToBG5Fp.png',0,0,1,1581335233,1581335233,'asdfasdf',2,'1,19,27,7',2,'https://www.youtube.com/watch?v=aLsplmkS0Vo',1),(3,'猛禽小隊：小丑女大解放','深度彩蛋解析！老娘就是酷！DC宇宙再扳回一局！','/upload/2020/02/10/bks5628jC2bRVUWQ.png',0,0,1,1581335259,1581335259,NULL,2,'1,10,11',5,'https://www.youtube.com/watch?v=4s9hJSloiP4',1),(4,'朱迪','2020奥斯卡电影：找个爱我的人，有多难？','/upload/2020/02/10/J7zZKxBEKSv32xPI.png',0,0,1,1581335264,1581335264,NULL,2,'1,10,26',5,'https://www.youtube.com/watch?v=-oPGx4XNqzo',1),(5,'爱有来生','死后，我等了你50年，俞飞鸿10年处女作','/upload/2020/02/10/Nxlqt4u1rlxh17Wi.png',0,0,1,1581335270,1581335270,NULL,2,'1,10',6,'https://www.youtube.com/watch?v=q9jUkDBLY2w',1),(6,'推拿','盲人推拿室里，无处安放的情与欲','/upload/2020/02/10/gtqqYecbHRwV0j5U.png',0,0,1,1581335275,1581335275,NULL,2,'1,10,11,13',6,'https://www.youtube.com/watch?v=H90E2mP5W5E',1),(7,'依然爱丽丝','确诊后的日子，每一天都是慢性死亡','/upload/2020/02/10/lJGIZW5NkuW0dmbW.png',0,0,1,1581335280,1581335280,NULL,2,'10,11,14,15,18,21',6,'https://www.youtube.com/watch?v=_tQHXp9H-L0',1),(8,'浪潮','一个恐怖的人性实验，真实改编','/upload/2020/02/10/5zpnVyd5g4axS02f.png',0,0,1,1581335286,1581335286,NULL,2,'1,10,11,12,2',6,'https://www.youtube.com/watch?v=vKpLJx1cpds',1),(9,'父辈的旗帜','二战期间，这里成了太平洋绞肉机','/upload/2020/02/10/KjsmhNBD78fUjWhH.png',0,0,1,1581335291,1581335291,NULL,2,'1,10,11,12,15,16',6,'https://www.youtube.com/watch?v=P3qxq0ebH8c',1),(10,'八月：奥色治郡','有一个刻薄的母亲，是种什么体验？','/upload/2020/02/10/GNtRBxrziAhl8T7A.png',0,0,1,1581335296,1581335296,NULL,2,'1,13,17,20',6,'https://www.youtube.com/watch?v=ZwZWDewDsP4',1),(11,'最后的莫西干人','欧洲黑历史，印第安人是怎么消失的？','/upload/2020/02/10/HbqCaFgKEpggORTb.png',0,0,1,1581334731,0,NULL,2,'13,14,17,18,25',6,'https://www.youtube.com/watch?v=69veiSnIfVk',0),(12,'荆轲刺秦王','秦始皇为什么要活埋赵国的孩子？','/upload/2020/02/10/RFA04Q35POWEi5rX.png',0,0,0,1581334791,0,NULL,2,'17,20,24,28',6,'https://www.youtube.com/watch?v=Ovp3zw7PHJU',0);
/*!40000 ALTER TABLE `videos` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `watch_records`
--

DROP TABLE IF EXISTS `watch_records`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `watch_records` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `user_id` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '用户编号',
  `user_name` varchar(20) NOT NULL DEFAULT '' COMMENT '用户各称',
  `video_id` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '视频编号',
  `created` int(10) unsigned NOT NULL DEFAULT '0' COMMENT '創建時間',
  PRIMARY KEY (`id`),
  KEY `user_id` (`user_id`),
  KEY `video_id` (`video_id`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `watch_records`
--

LOCK TABLES `watch_records` WRITE;
/*!40000 ALTER TABLE `watch_records` DISABLE KEYS */;
INSERT INTO `watch_records` VALUES (1,1,'user',1,1580470174);
/*!40000 ALTER TABLE `watch_records` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

-- Dump completed on 2020-02-10 20:00:14
