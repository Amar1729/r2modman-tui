use std::{
    fs,
    process::{exit, Command, Stdio},
    thread::sleep,
    time::Duration,
};

use nix::{
    sys::wait::waitpid,
    unistd::{fork, ForkResult, setsid},
};

// todo - this should be picked up from r2mm::Config
const BEPIN_PATH: &'static str = "/tmp/mods/bbepis-BepInExPack-5.3.1/BepInExPack/BepInEx/core/BepInEx.Preloader.dll";

// const ROR2_ARGS: [str; 2] = ["-applaunch", "632360"];

fn do_call(args: Vec<String>) {

    match fork().expect("Failed to fork process") {
        ForkResult::Parent { child } => {
            // println!("Try to kill me to check if the target process will be killed");

            // Do not forget to wait for the fork in order to prevent it from becoming a zombie!!!
            waitpid(Some(child), None).unwrap();

            // You have 120 seconds to kill the process :)
            // sleep(Duration::from_secs(120));
        }

        ForkResult::Child => {
            let output = fs::File::create("/tmp/steam.log").unwrap();
            let errors = fs::File::create("/tmp/steam.err").unwrap();
            // replace with your executable
            Command::new("steam")
                .args(args)
                .stdout(Stdio::from(output))
                .stderr(Stdio::from(errors))
                .spawn()
                .expect("failed to spawn the target process");

            setsid();

            sleep(Duration::from_secs(1));
            exit(0);
        }
    }
}

pub fn launch_game(modded: bool) {
    if cfg!(target_os = "linux") {
        let mut args = vec!(
            "-applaunch".to_owned(),
            "632360".to_owned()
        );

        if modded {
            args.extend(
                vec!(
                    "--doorstop-enable".to_owned(),
                    "true".to_owned(),
                    "--doorstop-target".to_owned(),
                    BEPIN_PATH.to_owned(),
                )
            );
        }

        do_call(args);
    }
}
