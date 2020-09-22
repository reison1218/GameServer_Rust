朋友的项目中要用到layui上传至阿里云OSS,自己花了一个晚上的时候.写了一个组件.直接直传的.但是appid与appkey都暴露在了前端.这样很不合适,所以后来又花了一个晚上将这个组件完善,但是获取签名需要后端配合,前端只去请求获取签名的接口,<!--more-->废话不多说,先看效果图:  
![image](https://raw.githubusercontent.com/xieyushi/blog/master/blogimg/bloggif7.gif)
插件分为单个文件上传与多个文件上传.成功与失败后,都有回调.大家在回调中对返回的url做操作即可  
插件已经上传至github,地址为[https://github.com/xieyushi/layui-aliossUploader](https://github.com/xieyushi/layui-aliossUploader)  
下面就直接说下这个插件的基本用法与配置吧:  
模块化这个就不再介绍.相信用过layui的都知道了.关键代码是下图中的这一段:  
![image](https://raw.githubusercontent.com/xieyushi/blog/master/blogimg/blogimg43.png)  
所有的配置字段都在这张图中了  

属性名称 | 作用 | 是否必填 | 默认值
---|---|---|---
elm | 绑定按钮的jq选择器 | 是 | 
fileType | 指定允许上传时校验的文件类型，可选值有：images（图片）、file（所有文件）、video（视频）、audio（音频） | 否 | 'file'
multiple | 是否多文件上传 | 否 | false
layerArea | 多文件上传打开的弹窗的大小 | 否 | 'auto'
layerTitle | 上传多文件弹窗标题 | 否 | 上传文件到阿里云OSS
policyUrl | 请求签名的url | 是 | 
policyData | 请求签名的参数 | 否 | {}
policyHeader | 请求签名的headers | 否 | {}
codeField | 请求签名返回的状态码字段 | 否 | 'code'
codeStatus | 请求签名返回的成功状态码值 | 否 | 0
policyMethod | 请求签名的方法 | 否 | 'GET'
accessidFiled | 返回签名的accessid字段名称 | 否 | 'accessid'
policyFiled | 返回签名的policy字段名称 | 否 | 'policy'
signatureFiled | 返回签名的signature字段名称 | 否 | 'signature'
httpStr | 上传至OSS时是否https | 否 | 'https'
region | OSS的数据中心所在的地域 | 是 | 
bucket | OSS的存储空间命名 | 是 | 
prefixPath | 上传多文件的前缀(相当于文件夹,可写多级,但不能以/开头必须以/结尾,如'aaa/bbb/'') | 否 | ''
allUploaded | 文件上传成功后的回调(多文件为所有文件上传完成后的回调) | 是 | 
allUploaded-res | allUploaded回调参数中的res结构为:{name:文件名称,type:文件类型,ossUrl:上传成功后的文件请求url(形状与上面的httpStr参数一致)} | 是 | 
policyFailed | 请求policy失败后的回调 | 是 | 
uploadRenderData | 支持部分layui的upload的参数配置 | 否 | {}
  
    
    
uploadRenderData的参数配置功能大致如下(并未测试,目前只是肓敲代码...)  
![image](https://raw.githubusercontent.com/xieyushi/blog/master/blogimg/blogimg44.png)  
参数解读就这么多了.不过插件也并完全完善,中间也有一些不足:  
1.文件上传至阿里云的名称,采用了随机命名:prefixPath + new Date().getTime() + '-' + (Math.random() + "").substring(2, 7) + '-' + file.name,如果有朋友不想这样的.请自行修改源码.  
2.uploadRenderData的参数并没有做太多的测试,可能有bug...  
3.签名的请求实在是没办法由前端来完成.我虽然做了一个纯前端的,但是真的不推荐,这里我在github上上传了一个json文件,里面是和这个插件匹配的返回数据结构展示,大致如下:  

```
{
	"code": 0,
	"success": true,
	"msg": "签名成功",
	"data": {
		"accessid": "XXXXX",
		"host": "http://XXXXX.oss-cn-shanghai.aliyuncs.com",
		"policy": "XXXX==",
		"signature": "XXXX=",
		"expire": 1554851252
	}
}
```
前端请将这个数组结构交给后台程序员来配合做签名.

  
使用此插件还需要注意的是需要配置阿里oss的跨域,设置post可进行跨域访问,不然图片上传是不会成功的.  
使用此插件还需要注意的是需要配置阿里oss的跨域,设置post可进行跨域访问,不然图片上传是不会成功的.  
使用此插件还需要注意的是需要配置阿里oss的跨域,设置post可进行跨域访问,不然图片上传是不会成功的.  
重要的事情说三遍,暂时就这么多.
  