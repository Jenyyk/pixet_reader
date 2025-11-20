# Pixet Reader

This is a library daemon for projects that read data from [Advacam Radiation imagers](https://advacam.com/)
It can also be used as a primitive standalone reader

## Daemon
All needed libraries for specific distributions are included in the repo
Here is a simple showcase of how to use the daemon:

In Python:
```Python
import subprocess
import time

# launch the daemon
proc = subprocess.Popen(
    ["pixet_reader"],
    stdin=subrocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    text=True
)

# creates a device with index 0
proc.stdin.write('add 0\n')
proc.stdin.flush()

# prompts the daemon to write the captured buffers to stdout
proc.stdin.write('get 0\n')
proc.stdin.flush()

# reads a single line from the output (first line specifies ammount of frames stored)
len = int(proc.stdout.readline().strip())

# print out the rest of the lines
for i in range(len):
    frame = proc.stdout.readline().rstrip("\n")
    print("Frame: ", frame)

# clear the accrued buffers
proc.stdin.write('clear 0\n')
proc.stdin.flush()
```


## Standalone
The project can also be used as a standalone reader:
```bash
./pixet_reader -S -M json -F muon -I
```
- `-S`: enabled standalone mode
  - `--standalone`: same as above
- `-M`: format in which to save the data, options: ( json, rak )
  - `--save-mode`: same as above
- `-F`: which particles to save, if not specified saves all, delimited by a single comma, options: ( muon )
  - `--filter`: same as above
- `-I`: whether to also save images of the particles
  - `--save-images`: same as above
