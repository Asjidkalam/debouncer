# debouncer
rust filewatcher, cache and restore files from memory.
prevents privileged users in the container to remove/modify restricted files.


## usage
```bash
./debouncer config.json
```

`config.json` will contain the a structure of **watched_paths** which will be cached and restored, if any modify/remove events are triggered.


## build from source 
```bash
git clone https://github.com/asjidkalam/debouncer.git
cd debouncer/
cargo build --release
```
