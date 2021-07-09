# samp-query-rs

Implements the [SA:MP query mechanism](https://sampwiki.blast.hk/wiki/Query_Mechanism) to retrieve info about a running server.

## Examples

```rs
let query = Query::new("127.0.0.1", 7777).await?;
query.send();
let packet = query.recv().await?; // access struct data afterwards
```
