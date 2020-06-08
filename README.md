# BOINC Supervisor

This application implements enough of [BOINC IPC](https://boinc.berkeley.edu/trac/wiki/ProjectMain#DevelopingBOINCapplications) to support running simple apps without real BOINC client.

For reference see in BOINC sources:
- api/boinc_api.*
- lib/api_ipc.*
- lib/shmem.*


## Preparing files from Rosetta

- Prepare a destination directory for your files:
  - Put absolute path to it in `dest` variable.
  - Run `mkdir -p $dest/{slots,projects}`.
- Copy Rosetta files:
  - Save boinc directory path (probably `/var/lib/boinc`, if it's running as system daemon) as `boinc` variable.
  - `cp -r $boinc/projects/boinc.bakerlab.org_rosetta $dest/projects/`
  - Find pid (save as `rosetta_pid`) and a slot directory name (save as `slot`) of a running Rosetta process using `lsof +d $boinc/slots/`.
  - `cp -r $boinc/slots/$slot $dest/slots/`
  - Save the command line using `ps -p $rosetta_pid -fww > $dest/slots/$slot/run.sh`. Edit the file so it looks like
    ```sh
    #!/bin/sh
    exec ../../projects/boinc.bakerlab.org_rosetta/rosetta_4.20_x86_64-pc-linux-gnu [...]
    ```
  - `sed 's|<client_pid>[0-9]*</client_pid>|<client_pid>1</client_pid>|' $dest/slots/$slot/init_data.xml`


## Running

- [Build this as a static binary.](https://doc.rust-lang.org/edition-guide/rust-2018/platform-and-target-support/musl-support-for-fully-static-binaries.html)
- `cp target/x86_64-unknown-linux-musl/release/boinc-supervisor $dest/slots/$slot/`
- `docker run -v "${dest}:${boinc}" -ti --rm -u "$(id -u):$(id -g)" alpine sh` Warning: it's important that the `$boinc` path inside container is the same as it was on host system.
- Inside docker:
  - Remember, that's it's a new shell, so you don't have your `boinc` and `slot` variables.
  - `cd $boinc/slots/$slot`
  - `./boinc-supervisor &`
  - `./run.sh`
