while true; do
  socat -d2 -x -v /dev/ttyUSB0,clocal=1,nonblock=1,b9600,cs8,rawer,start=1,stop=1,parenb=0 UNIX-LISTEN:/tmp/cctalk.sock
  echo "Connection closed, restarting..."
  sleep 0.1
done
