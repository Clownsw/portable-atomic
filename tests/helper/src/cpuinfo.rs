#![cfg(target_arch = "aarch64")]

use std::{boxed::Box, path::Path, process::Command, vec::Vec};

use fs_err as fs;

type Result<T, E = Box<dyn std::error::Error + Send + Sync>> = std::result::Result<T, E>;

// CPU feature detection using reading a file or calling a command.
// - On Linux/NetBSD, reading /proc/cpuinfo
// - On FreeBSD/OpenBSD, reading /var/run/dmesg.boot
// - On macOS, calling sysctl hw.optional
// This is used for testing to ensure that the result of the CPU feature
// detection we are using matches the information we get from the other
// approaches.
#[derive(Debug, Clone, Copy)]
pub struct ProcCpuinfo {
    pub lse: bool,
    pub lse2: Option<bool>,
}
impl ProcCpuinfo {
    pub fn new() -> Result<Self> {
        if cfg!(any(target_os = "linux", target_os = "android", target_os = "netbsd")) {
            let text = fs::read_to_string("/proc/cpuinfo")?;
            let features = text
                    .lines()
                    // on qemu-user, there is not 'Features' section
                    // TODO: check whether a runner is set instead.
                    .find_map(|line| line.strip_prefix("Features")).ok_or("no 'Features' section in /proc/cpuinfo")?
                    .splitn(2, ':')
                    .nth(1)
                    .unwrap()
                    .split(' ')
                    .map(str::trim)
                    .collect::<Vec<_>>();
            std::eprintln!("Features={:?}", features);
            Ok(Self { lse: features.contains(&"atomics"), lse2: Some(features.contains(&"uscat")) })
        } else if cfg!(target_os = "freebsd") {
            let text = fs::read_to_string("/var/run/dmesg.boot")?;
            let isa0 = text
                .lines()
                .find(|line| line.contains("Instruction Set Attributes 0"))
                .ok_or("no 'Instruction Set Attributes 0' section in /var/run/dmesg.boot")?
                .splitn(2, '=')
                .nth(1)
                .unwrap()
                .trim()
                .strip_prefix('<')
                .unwrap()
                .strip_suffix('>')
                .unwrap()
                .split(',')
                .collect::<Vec<_>>();
            let mmf2 = text
                .lines()
                .find(|line| line.contains("Memory Model Features 2"))
                .ok_or("no 'Memory Model Features 2' section in /var/run/dmesg.boot")?
                .splitn(2, '=')
                .nth(1)
                .unwrap()
                .trim()
                .strip_prefix('<')
                .unwrap()
                .strip_suffix('>')
                .unwrap()
                .split(',')
                .collect::<Vec<_>>();
            std::eprintln!("Instruction Set Attributes 0={:?}", isa0);
            std::eprintln!("Memory Model Features 2={:?}", mmf2);
            Ok(Self { lse: isa0.contains(&"Atomic"), lse2: Some(mmf2.contains(&"AT")) })
        } else if cfg!(target_os = "openbsd") {
            let text = fs::read_to_string("/var/run/dmesg.boot")?;
            let features = text
                .lines()
                .filter_map(|line| line.strip_prefix("cpu0: "))
                .last()
                .ok_or("no 'cpu0' section in /var/run/dmesg.boot")?
                .trim()
                .split(',')
                .collect::<Vec<_>>();
            std::eprintln!("Features={:?}", features);
            Ok(Self {
                lse: features.contains(&"Atomic"),
                // /var/run/dmesg.boot on OpenBSD doesn't have field for lse2
                lse2: None,
            })
        } else if cfg!(target_os = "macos") {
            let output = Command::new("sysctl").arg("hw.optional").output()?;
            assert!(output.status.success());
            let stdout = std::str::from_utf8(&output.stdout)?.trim();
            std::eprintln!("{}", stdout);
            let sysctl = |name| {
                stdout
                    .lines()
                    .find_map(|s| s.strip_prefix(&std::format!("{}: ", name)))
                    .unwrap_or("0")
                    .parse::<u32>()
                    .unwrap()
            };
            Ok(Self {
                lse: sysctl("hw.optional.arm.FEAT_LSE") != 0
                    || sysctl("hw.optional.armv8_1_atomics") != 0,
                lse2: Some(sysctl("hw.optional.arm.FEAT_LSE2") != 0),
            })
        } else {
            assert!(!Path::new("/proc/cpuinfo").exists());
            assert!(!Path::new("/var/run/dmesg.boot").exists());
            Err("unsupported OS".into())
        }
    }
}
