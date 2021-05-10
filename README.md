# 8080 Memory Bus

The 21st century emulation architecture calls for 4 key data endpoints to exist: readByte, writeByte, initialize & readRange. This is a simple implementation of all these
endpoints in a single high performance rust microservice using in memory state.

## Testing

`cargo test`