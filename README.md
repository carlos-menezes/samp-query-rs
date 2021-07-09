# samp-query-rs

Implements the [SA:MP query mechanism](https://wiki.sa-mp.com/wiki/Query_Mechanism) to retrieve info about a running server.

## Examples

```rs
let query = Query::new(s.0, s.1).await?;
query.send();
let packet = query.recv().await?; // access struct data afterwards
```
