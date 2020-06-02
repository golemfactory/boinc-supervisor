/*
Copyright 2020 Golem Factory <contact@golem.network>

This file is part of boinc-supervisor.

boinc-supervisor is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

boinc-supervisor is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with boinc-supervisor.  If not, see <https://www.gnu.org/licenses/>.
*/

use memmap::MmapMut;
use std::{
    fs::OpenOptions,
    io,
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread, time,
};

#[macro_use]
extern crate arrayref;

pub mod msg_channel;
use crate::msg_channel::{MsgChannel, MSG_CHANNEL_SIZE};

const TIMER_PERIOD: time::Duration = time::Duration::from_millis(100); // 0.1s
const MMAPPED_FILE_NAME: &'static str = "boinc_mmap_file";

enum ChannelId {
    // ProcessControlRequest = 0,
    // ProcessControlReply = 1,
    // GraphicsRequest = 2,
    // GraphicsReply = 3,
    Heartbeat = 4,
    AppStatus = 5,
    // TrickleUp = 6,
    // TrickleDown = 7,
}

struct SharedMem {
    mmap: MmapMut,
}

impl SharedMem {
    fn new(path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        file.set_len(8 * 1024)?;
        let mmap = unsafe { MmapMut::map_mut(&file) }?;
        Ok(Self { mmap })
    }

    fn get_channel(&mut self, channel_id: ChannelId) -> MsgChannel {
        array_mut_ref![
            self.mmap,
            (channel_id as usize) * MSG_CHANNEL_SIZE,
            MSG_CHANNEL_SIZE
        ]
        .into()
    }
}

fn main() {
    let run = Arc::new(AtomicBool::new(true));
    ctrlc::set_handler({
        let run = run.clone();
        move || {
            run.store(false, Ordering::SeqCst);
        }
    })
    .expect("Error setting Ctrl+C handler");

    let mut shared_mem = SharedMem::new(Path::new(MMAPPED_FILE_NAME)).expect("Failed mapping file");

    while run.load(Ordering::SeqCst) {
        if let Some(msg) = shared_mem.get_channel(ChannelId::Heartbeat).get_msg() {
            match msg {
                Ok(msg) => println!("got heartbeat: {}", msg),
                Err(e) => println!("heatbeat error: {}", e),
            }
        }
        if let Some(msg) = shared_mem.get_channel(ChannelId::AppStatus).get_msg() {
            match msg {
                Ok(msg) => println!("got app status: {}", msg),
                Err(e) => println!("app status error: {}", e),
            }
        }
        thread::sleep(TIMER_PERIOD);
    }
}
