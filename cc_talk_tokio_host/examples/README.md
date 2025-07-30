# ccTalk Tokio Host Examples

## Requirements

You will need to have a `serial port` bridged as a `unix domain socket`.

### socat example

```sh
DEVICE=/dev/ttyUSB0

# Linux
socat -d1 -x -v $DEVICE,clocal=1,nonblock=1,b9600,cs8,rawer,start=1,stop=1,parenb=0 UNIX-LISTEN:/tmp/cctalk.sock,fork,ignoreeof

# MacOs
socat -d2 -x -v $DEVICE,clocal=1,nonblock=1,ispeed=9600,ospeed=9600,cs8,rawer,echo=0 UNIX-LISTEN:/tmp/cctalk.sock,fork,ignoreeof
```
