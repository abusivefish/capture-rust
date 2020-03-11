extern crate rscam;
extern crate nix;
extern crate tempdir;

use rscam::{ResolutionInfo};
use std::fs;
use std::io::prelude::Write; 
use tempdir::TempDir;
use nix::sys::stat;
use nix::unistd;

//Raw sandbox for tinkering -- obviously this is a WIP

fn main() {

    // new instance of rscam for /dev/video0
    let src = rscam::new("/dev/video0").unwrap();
    for fmt in src.formats() {
        let format = fmt.unwrap();
        println!("{:?}", format);
    }

    //Get Available input formats from v4l2
    //TODO: Dynamically gather supported formats at startup
    //Select Default format; Save others in a model to be used for stream config
    for fmt in src.formats() {
        let format = fmt.unwrap();
        println!("{:?}", format);

        let resolutions = src.resolutions(&format.format).unwrap();

        if let ResolutionInfo::Discretes(d) = resolutions {
            for resol in &d {
                println!(
                    "  {}x{}  {:?}",
                    resol.0,
                    resol.1,
                    src.intervals(&format.format, *resol).unwrap()
                );
            }
        } else {
            println!("  {:?}", resolutions);
        }
    }

    // Set up temp directory and fifo for a stream
        let tmp_dir = TempDir::new("/home/ui-claw/tmp").unwrap();
        let fifo_path = tmp_dir.path().join("hdmi.fifo");

        //make fifo - see: https://pubs.opengroup.org/onlinepubs/9699919799/functions/mkfifo.html
        //with perms set to S_IRWXU - see: https://www.gnu.org/software/libc/manual/html_node/Permission-Bits.html
        match unistd::mkfifo(&fifo_path, stat::Mode::S_IRWXU) {
            Ok(_) => {
                println!("should have made fifo");
                
                capture(&fifo_path);
            },
            Err(err) => println!("Error creating fifo: {}", err),
        }
        //TODO: clean up tempdir on Ctrl-C
}

// static v4l2 settings for now - see: https://docs.rs/rscam/0.5.5/rscam/


fn capture(path: &std::path::PathBuf) {

    let mut hdmi = rscam::new("/dev/video0").unwrap();

    const VID_WIDTH: u32 = 1920;
    const VID_HEIGHT: u32 = 1080;

    hdmi
        .start(&rscam::Config {
            interval: (1, 30),
            resolution: (VID_WIDTH, VID_HEIGHT),  
            format: b"MJPG",
            ..Default::default()
        })
        .unwrap();

        //indefinite range -- iterative endless loop
        for _ in 0.. {
            let frame = hdmi.capture().unwrap();

            let mut file = fs::OpenOptions::new()
                .write(true)
                .open(path)
                .unwrap();
            file.write(&frame[..]).unwrap();
        }
        // this ^^^ is terrible. The buffer isn't actually emptied until something reads the frame on the other side
        // TODO: implement std::sync::mpsc for fifo structure
        // TODO: consider gstreamer-rs for encoding and manipulation pipeline

}

// TODO: Rewrite above to use state machine below
// TODO: change main to setup() -> Stream

// State machine for the input Stream

/*
pub enum Stream {
    Setup,
    Start,
    Stop,
}

fn start_stream(state: &Stream) -> Stream {
    match state {
        Stream::Setup => unimplemented!(),
        Stream::Start => unimplemented!(),
        Stream::Stop =>  unimplemented!(),
    };
}

fn setup() -> Stream {

}

fn start() -> Stream {

}

fn stop() -> Stream {

}

fn main() {

}
*/


//-----------------------------------------------------//