mod termios;

use clap::{arg, command, value_parser, ArgAction};
use crossbeam_channel::{bounded, select, Receiver};
use std::error::Error;
use std::fs::canonicalize;
use std::io::{stdin, stdout};
use std::path::PathBuf;
use std::process;

use crate::termios::{get_terminal_attr, set_raw_mode, set_terminal_attr};

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = command!()
        .arg(
            arg!(-k --kernel <PATH> "Path to bZImage kernel file.")
                .required(true)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(-i --initrd <PATH> "Path to initrd.")
                .required(false)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(-c --commandline <ARGS> "Command line arguments passed to the kernel on startup.")
                .required(false)
                .default_value("console=hvc0 root=/dev/vda"),
                //.default_value("console=hvc0 root=/dev/vda rw"), // *.ext4
        )
        .arg(
            arg!(-d --disk <DISKS> "Path to disks.")
                .required(false)
                .action(ArgAction::Append)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(--cpu <COUNT> "Amount of CPUs to assign to the VM.")
                .required(false)
                .value_parser(value_parser!(usize))
                .default_value("4"),
        )
        .arg(
            arg!(--memory <SIZE> "Amount of memory to assign to the VM.")
                .required(false)
                .value_parser(value_parser!(u64))
                .default_value("2147483648"),
        )
        .get_matches();

    let kernel = matches
        .get_one::<PathBuf>("kernel")
        .expect("Kernel parameter should be provided!");

    let command_line = matches.get_one::<String>("commandline").unwrap();

    let disks = matches
        .get_many::<PathBuf>("disk")
        .unwrap_or_default()
        .map(|v| v.as_os_str())
        .collect::<Vec<_>>();

    let cpu_count = matches.get_one::<usize>("cpu").unwrap();
    let memory_size = matches.get_one::<u64>("memory").unwrap();

    if !vz::VirtualMachine::supported() {
        println!("VirtualMachine not supported");
        process::exit(1);
    }

    let kernel_url = canonicalize(kernel)
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap();

    let initrd_url = matches.get_one::<PathBuf>("initrd")
        .map_or(String::new(), |v| {
            canonicalize(v)
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
        });

    let boot_loader = vz::LinuxBootLoader::new(&kernel_url, &initrd_url, command_line);

    let std_in = stdin();
    let std_out = stdout();

    let attachment = vz::FileHandleSerialPortAttachment::new(&std_in, &std_out);
    let serial_port =
        vz::VirtioConsoleDeviceSerialPortConfiguration::new_with_attachment(attachment);

    //let memory_balloon = vz::VirtioTraditionalMemoryBalloonDeviceConfiguration::new();
    //let entropy_device = vz::VirtioEntropyDeviceConfiguration::new();

    let config = vz::VirtualMachineConfiguration::new(boot_loader, *cpu_count, *memory_size);
    let block_devices: Vec<vz::VirtioBlockDeviceConfiguration> = disks
        .iter()
        .map(|disk| {
            let path = canonicalize(disk)
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap();

            let attachment = vz::DiskImageStorageDeviceAttachment::new(&path, false);

            vz::VirtioBlockDeviceConfiguration::new(attachment)
        })
        .collect();
/*
    let network_device = vz::VirtioNetworkDeviceConfiguration::new_with_attachment(
        vz::NATNetworkDeviceAttachment::new(),
    );
    network_device.set_mac_address(vz::MACAddress::new_with_random_locally_administered_address());
 */
    //config.set_entropy_devices(vec![entropy_device]);
    config.set_serial_ports(vec![serial_port]);
    //config.set_memory_balloon_devices(vec![memory_balloon]);
    config.set_storage_devices(block_devices);
    //config.set_network_devices(vec![network_device]);

    if let Err(msg) = config.validate() {
        println!("Invalid Configuration: {}", msg);
        process::exit(1);
    }

    let vm = vz::VirtualMachine::new(&config);

    if !vm.can_start() {
        println!("VM can't start!");
        process::exit(1);
    }

    println!("Starting VM...");
    vm.start()?;
    println!("VM started!");

    let termios = get_terminal_attr(&std_in)?;
    set_raw_mode(&std_in)?;

    let ctrl_c_events = ctrl_channel()?;
    let state_changes = vm.get_state_channel();

    println!("Waiting for VM state changes...");
    loop {
        select! {
            recv(state_changes) -> state => {
                match state {
                    Ok(vz::VirtualMachineState::Running) => println!("Virtual machine is running!"),
                    Ok(vz::VirtualMachineState::Stopped) => {
                        println!("Virtual machine has stopped, exiting!");
                        break;
                    }
                    _ => {
                        println!("Virtual machine state: {:?}", state);
                    }
                }
            }
            recv(ctrl_c_events) -> _ => {
                set_terminal_attr(&std_in, &termios).expect("Failed to reset tty back to original state!");

                if vm.can_stop() {
                    let _  = vm.stop();
                }

                break;
            }
        }
    }

    println!("\nExiting");
    Ok(())
}
