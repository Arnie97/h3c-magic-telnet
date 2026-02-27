# h3c-magic-telnet


## 简介

H3C Magic 系列路由器（NX15 / NX30 / NX30 Pro / NX54 等）原厂系统中，
曾在 99 端口提供了 root 权限 Telnet 服务，便于进阶用户更换固件，或通过
UCI 调整 VLAN、策略路由、IPv4 / IPv6 防火墙规则等。在 2025 年某次固件
OTA 更新后，该功能被默认禁用。本项目用于恢复新版固件中的 Telnet 服务。


## 下载

### 从源码编译

    cargo install --git https://github.com/fg193/h3c-magic-telnet

### 预编译二进制

请移步 [Releases](https://github.com/fg193/h3c-magic-telnet/releases)
页面下载。


## 使用

运行本程序。如果本机的网关是 H3C Magic 系列路由器，
程序通常会自动找到其地址，打印出设备型号、序列号、MAC 地址等信息。
在其他比较复杂的网络环境中（例如 H3C Magic 路由器不是本机的网关），请在
`Specify the IP manually` 提示符后输入 H3C Magic 路由器的 IP 地址。

之后，程序会询问路由器管理页面登录密码。
一切顺利的话，99 端口的 Telnet 服务应该恢复了：

    GET http://192.168.124.1/api/debug?status=enableXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
    [{"id": 1,"result": {"code":0,"message":"Success","data":{}}}]
    Press ENTER to exit...

不顺利的话，可能遇到各种不同报错：

    GET http://192.168.124.1/api/debug?status=enableXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
    [{"id": 1,"result": {"code":4,"message":"Invalid Params","data":{}}}]
    Status(4, "Invalid Params")

无论成功与否，都欢迎反馈您的固件版本，以便本项目后续优化。


## 常见问题

* 管理页面登录密码是否正确？
* 是否是旧版固件？旧版固件始终开启 Telnet，无需本工具。
  因为没有 Telnet 开关功能，如果强行使用必然报错


## 后记

[茶栗的文章中，详细介绍了多种较复杂的替代方法][chariri]。
本项目尝试站在巨人肩膀上，做一点微小的工作，以简化操作流程。
关于其原理，可以在 Telnet 中运行 `encryptpwd auto` 并阅读相关代码。

[chariri]: https://chariri.moe/archives/2383/h3c-nx30-pro-openwrt-new-stock-firmware-no-ttl/
