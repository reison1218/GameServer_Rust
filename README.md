# GameServer_Rust
RustProject
完全是用rust编写游戏服务器，设计完全是由之前的java版本的服务器灵感而来。
其中包含几个重要的组建，如下：
1.gameserver：专门处理玩家数据和逻辑，里面已经包含游戏服基本需要的所有组件，如下（以下大部分组件来自于tools）:
  a:tcp server用于监听gateserver发过来的消息，基于mio(0.7.0)实现
  b:集成mysql客户端.基于mysql(16.1.0)
  c:redis客户端，基于redis(0.13.0)
  d:处理的线程池，可以使用async-std(1.5.0)或者ThreadPool(1.7.1)
  e:异步定时器，用于执行一些定时任务，比如每日0点重制，每5分钟异步将玩家数据持久化到mysql服务端
  f:http服务端和http客户端，基于async-h1(1.0.2)和http-types(1.1.0)，用于处理一些其他组件的请求事件，比如关闭服务器，或者别的什么需求
  g:并源码附带简单的逻辑处理，通信协议采用tcp，基于mio(0.7.0）实现
  h:数据存储采用json方式，框架选取的serde和serde_json,之所以设计也是为了灵活性，方便扩展。
  i:日志模块，用于记录日志，并持久化到磁盘，分为error.log和info.log
2.gateserver:用于与gameserver和roomserver进行通信，它是gameserver和roomserver的桥梁，由于它是无状态的，不缓存任何玩家数据，只用来消息转发，所以不存在数据处理的逻辑和持久化的模块，组件如下（以下大部分组件来自于tools）：
  a:tcp server,用于监听游戏客户端发送过来的消息
  b:tcp client,分为gameserver的tcp client和roomserver的tcp client，用于给游戏客户端发送消息
  c:日志模块，用于记录日志，并持久化到磁盘，分为error.log和info.log
  d:websocket server,用于监听游戏客户端发送过来的消息，d和a可以根据需求进行切换，想用哪个完全取决于你
3.roomserver:用来处理战斗相关的任务，可以根据需求扩展，组件如下（以下大部分组件来自于tools）：
  a:tcp server,用于监听gateserver发送过来的消息
  b:reids client
4.tools:其他项目的lib，封装一些tcp server，client，主旨是将底层的组件封装得简单易用，然后暴露出简单的api给其他project使用。其他项目的tcp，http，log，threadpool,protobuf files等等均来自于它.重要等组件
  a:tcp，封装好tcp模块，包括客户端和服务端，暴露出api给上层使用
  b:http,封装好http模块，包括客户端和服务端，暴露出api给上层使用
  c:log,封装好log模块，暴露api给上层使用
  d:protos,生成好protobuf文件，提供给所有项目使用
  e:threadpool，封装好线程池模块，暴露api给上层使用
  f:cmd_code,封装好gameserver，roomserver，gateserver各自需要负责等命令号段
  g:conf,封装好加载json配置文件，暴露api给上层使用，比如mysql的连接地址，tcp的监听地址，redis的连接地址等等。
  h:template,封装好加载json配置文件，暴露api给上层使用，比如加载npc配置，地图配置，关卡配置？取决于你
  i:binary,提供一些位运算的API，比如两个u32合成一个u64,一个u64拆成两个u32
  j:util,提供一些
