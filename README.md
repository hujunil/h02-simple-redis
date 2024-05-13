# Rust 异步编程

实现一个简单的 redis server，支持: get, set, hget, hset, hgetall, sadd, sismember, echo 命令

# 作业

1、支持 echo, sadd, sismember 命令。
2、重构了 BulkString, 兼容 NullBulkString 的场景；重构了 RespArray, 兼容 NullArray 的场景。

# 运行

启动服务

```bash
RUST_LOG=debug cargo run
```

使用 `redis-cli` 进行功能测试

```bash
$ ./redis-cli
127.0.0.1:6379> set hello world
OK
127.0.0.1:6379> get hello
"world"
127.0.0.1:6379> hset user name zhangs
OK
127.0.0.1:6379> hset user age 18
OK
127.0.0.1:6379> hmget user name age
1) "zhangs"
2) "18"
127.0.0.1:6379> hget user name
"zhangs"
127.0.0.1:6379> hget user age
"18"
127.0.0.1:6379> hgetall user
1) "age"
2) "18"
3) "name"
4) "zhangs"
127.0.0.1:6379> sadd tools rust java javascript python
(integer) 4
127.0.0.1:6379> sismember tools java
(integer) 1
127.0.0.1:6379> sismember tools c++
(integer) 0
127.0.0.1:6379> echo hello
"hello"
```
