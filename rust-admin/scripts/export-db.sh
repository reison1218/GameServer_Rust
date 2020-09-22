#!/bin/bash

#主机地址
#host='locahost'
#数据库名
name='rust_admin'
#用户名称
user='rust_admin'
#登录密码
password='rust-x-lsl'

#备份的文件名称, 格式: 年月日.SQL
sql_file="`date '+%Y%M%d'`.SQL"
if [ -f $sql_file ]; then
    rm -rf $sql_file
fi

#mysqldump路径
dump_bin='mysqldump'

#执行备份
$dump_bin -u$user -p"$password" $name > $sql_file
