//use std::ffi::c_void;

use block2::ConcreteBlock;
use crossbeam_channel::{bounded, Receiver, Sender};
use objc2::rc::{/* autoreleasepool,  */Id, Shared};
//use objc2::runtime::{NSObject, NSObjectProtocol, Object};
use objc2::ClassType;
//use objc2::{declare_class, msg_send_id};

use crate::sealed::UnsafeGetId;
use crate::sys::foundation::NSError/* {, NSKeyValueObservingOptions, NSString} */;
use crate::sys::virtualization::VZVirtualMachine;

use crate::configuration::VirtualMachineConfiguration;
use crate::sys::queue::{Queue, QueueAttribute};
use std::fmt::Display;

#[derive(Debug)]
pub enum VirtualMachineState {
    Stopped = 0,
    Running = 1,
    Paused = 2,
    Error = 3,
    Starting = 4,
    Pausing = 5,
    Resuming = 6,
    Stopping = 7,
    Unknown = -1,
}
impl Display for VirtualMachineState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VirtualMachineState::Stopped => write!(f, "Stopped"),
            VirtualMachineState::Running => write!(f, "Running"),
            VirtualMachineState::Paused => write!(f, "Paused"),
            VirtualMachineState::Error => write!(f, "Error"),
            VirtualMachineState::Starting => write!(f, "Starting"),
            VirtualMachineState::Pausing => write!(f, "Pausing"),
            VirtualMachineState::Resuming => write!(f, "Resuming"),
            VirtualMachineState::Stopping => write!(f, "Stopping"),
            VirtualMachineState::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug)]
pub struct VirtualMachine {
    machine: Id<VZVirtualMachine, Shared>,
    queue: Queue,
//    observer: Id<VirtualMachineStateObserver, Shared>,

    _notifier: Sender<VirtualMachineState>,
    pub state_notifications: Receiver<VirtualMachineState>,
}

/// VirtualMachine represents the entire state of a single virtual machine.
///
/// **Support**: macOS 11.0+
///
/// A VirtualMachine object emulates a complete hardware machine of the same architecture as the underlying Mac computer.
/// Use the VM to execute a guest operating system and any other apps you install.
/// The VM manages the resources that the guest operating system uses, providing access to some hardware resources while emulating others.
///
/// Creating a virtual machine using the Virtualization framework requires the app to have the "com.apple.security.virtualization" entitlement.
/// see: <https://developer.apple.com/documentation/virtualization/vzvirtualmachine?language=objc>
impl VirtualMachine {
    pub fn new(config: &VirtualMachineConfiguration) -> Self {
        unsafe {
            let queue = Queue::create("com.virt.fwk.rs", QueueAttribute::Serial);
            let machine = VZVirtualMachine::initWithConfiguration_queue(
                VZVirtualMachine::alloc(),
                &config.id(),
                queue.ptr,
            );

            let (sender, receiver) = bounded(1);
/*             let observer = VirtualMachineStateObserver::new();

            let key = NSString::from_str("state"); */

            /* let mut vm =  */VirtualMachine {
                machine,
                queue,
//                observer,
                _notifier: sender,
                state_notifications: receiver,
            } /* ; */
/*             let vm_ptr: *mut c_void = &mut vm as *mut _ as *mut c_void;

            vm.machine.addObserver_forKeyPath_options_context(
                &vm.observer,
                &key,
                NSKeyValueObservingOptions::NSKeyValueObservingOptionNew,
                vm_ptr,
            );

            vm
*/
        }
    }

    /// Returns a crossbeam receiver channel that can be used to receive updates about the VirtualMachine's state changes.
    pub fn get_state_channel(&self) -> Receiver<VirtualMachineState> {
        self.state_notifications.clone()
    }

    /// Returns a boolean value that indicates whether the system supports virtualization.
    pub fn supported() -> bool {
        unsafe { VZVirtualMachine::isSupported() }
    }

    /// Synchronously starts the VirtualMachine.
    pub fn start(&self) -> Result<(), String> {
        unsafe {
            let (tx, rx) = std::sync::mpsc::channel();
            let dispatch_block = ConcreteBlock::new(move || {
                let inner_tx = tx.clone();
                let completion_handler = ConcreteBlock::new(move |err: *mut NSError| {
                    if err.is_null() {
                        inner_tx.send(Ok(())).unwrap();
                    } else {
                        inner_tx
                            .send(Err(err.as_mut().unwrap().localized_description()))
                            .unwrap();
                    }
                });

                let completion_handler = completion_handler.copy();
                self.machine.startWithCompletionHandler(&completion_handler);
            });

            let dispatch_block_clone = dispatch_block.clone();
            self.queue.exec_block_async(&dispatch_block_clone);

            let result = rx.recv();

            if result.is_err() {
                return Err("TODO: implement better error handling here!".into());
            }

            result.unwrap()?;

            Ok(())
        }
    }

    /// Synchronously stops the VirtualMachine.
    pub fn stop(&self) -> Result<(), String> {
        let (tx, rx) = std::sync::mpsc::channel();
        let dispatch_block = ConcreteBlock::new(move || {
            let inner_tx = tx.clone();
            unsafe {
                let completion_handler = ConcreteBlock::new(move |err: *mut NSError| {
                    if err.is_null() {
                        inner_tx.send(Ok(())).unwrap();
                    } else {
                        inner_tx
                            .send(Err(err.as_mut().unwrap().localized_description()))
                            .unwrap();
                    }
                });

                let completion_handler = completion_handler.copy();
                self.machine.stopWithCompletionHandler(&completion_handler);
            }
        });

        let dispatch_block_clone = dispatch_block.clone();
        self.queue.exec_block_async(&dispatch_block_clone);

        let result = rx.recv();

        if result.is_err() {
            return Err("TODO: implement better error handling here!".into());
        }

        result.unwrap()?;

        Ok(())
    }

    /// Pause the running VM synchronously.
    pub fn pause(&self) -> Result<(), String> {
        // (Optional) check if can_pause is true.
        // if !self.can_pause() {
        //     return Err("Cannot pause the VM in its current state.".into());
        // }

        let (tx, rx) = std::sync::mpsc::channel();

        // We create a block that calls `pauseWithCompletionHandler:`
        let dispatch_block = ConcreteBlock::new(move || {
            let inner_tx = tx.clone();
            unsafe {
                let completion_handler = ConcreteBlock::new(move |err: *mut NSError| {
                    if err.is_null() {
                        // Pausing succeeded
                        inner_tx.send(Ok(())).unwrap();
                    } else {
                        // Pausing failed, send error
                        inner_tx
                            .send(Err(err.as_mut().unwrap().localized_description()))
                            .unwrap();
                    }
                });
                // We must copy the block to extend its lifetime
                let completion_handler = completion_handler.copy();

                // Call AVF's pause method
                self.machine.pauseWithCompletionHandler(&completion_handler);
            }
        });

        // Clone the block for enqueuing, just like in start() / stop()
        let dispatch_block_clone = dispatch_block.clone();
        self.queue.exec_block_async(&dispatch_block_clone);

        // Wait for result
        match rx.recv() {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(_) => Err("Failed to receive pause completion result!".to_string()),
        }
    }

    /// Resume the paused VM synchronously.
    pub fn resume(&self) -> Result<(), String> {
        // (Optional) check if can_resume is true.
        // if !self.can_resume() {
        //     return Err("Cannot resume the VM in its current state.".into());
        // }

        let (tx, rx) = std::sync::mpsc::channel();

        // We create a block that calls `resumeWithCompletionHandler:`
        let dispatch_block = ConcreteBlock::new(move || {
            let inner_tx = tx.clone();
            unsafe {
                let completion_handler = ConcreteBlock::new(move |err: *mut NSError| {
                    if err.is_null() {
                        // Resume succeeded
                        inner_tx.send(Ok(())).unwrap();
                    } else {
                        // Resume failed, send error
                        inner_tx
                            .send(Err(err.as_mut().unwrap().localized_description()))
                            .unwrap();
                    }
                });
                // We must copy the block to extend its lifetime
                let completion_handler = completion_handler.copy();

                // Call AVF's resume method
                self.machine.resumeWithCompletionHandler(&completion_handler);
            }
        });

        // Clone the block for enqueuing, just like in start() / stop()
        let dispatch_block_clone = dispatch_block.clone();
        self.queue.exec_block_async(&dispatch_block_clone);

        // Wait for result
        match rx.recv() {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(_) => Err("Failed to receive resume completion result!".to_string()),
        }
    }

    pub fn can_start(&self) -> bool {
        self.queue
            .exec_sync(move || unsafe { println!("can_start: {:?}", self.machine.canStart()); self.machine.canStart() })
    }

    pub fn can_stop(&self) -> bool {
        self.queue
            .exec_sync(move || unsafe { self.machine.canRequestStop() })
    }

    pub fn can_pause(&self) -> bool {
        self.queue
            .exec_sync(move || unsafe { self.machine.canPause() })
    }

    pub fn can_resume(&self) -> bool {
        self.queue
            .exec_sync(move || unsafe { self.machine.canResume() })
    }

    pub fn can_request_stop(&self) -> bool {
        self.queue
            .exec_sync(move || unsafe { self.machine.canRequestStop() })
    }

    /// Returns the current execution state of the VM.
    pub fn state(&self) -> VirtualMachineState {
        unsafe {
            match self.machine.state() {
                0 => VirtualMachineState::Stopped,
                1 => VirtualMachineState::Running,
                2 => VirtualMachineState::Paused,
                3 => VirtualMachineState::Error,
                4 => VirtualMachineState::Starting,
                5 => VirtualMachineState::Pausing,
                6 => VirtualMachineState::Resuming,
                7 => VirtualMachineState::Stopping,
                _ => VirtualMachineState::Unknown,
            }
        }
    }
}
/*
impl Drop for VirtualMachine {
    fn drop(&mut self) {
        let key_path = NSString::from_str("state");

        let vm_ptr: *mut c_void = self as *mut _ as *mut c_void;

        unsafe {
            self.machine
                .removeObserver_forKeyPath_context(&self.observer, &key_path, vm_ptr);
        }
    }
}

declare_class!(
    #[derive(Debug)]
    struct VirtualMachineStateObserver;

    unsafe impl ClassType for VirtualMachineStateObserver {
        type Super = NSObject;
        const NAME: &'static str = "VirtualMachineStateObserver";
    }

    unsafe impl VirtualMachineStateObserver {
        #[method(observeValueForKeyPath:ofObject:change:context:)]
        unsafe fn observe_value_for_key_path(
            &self,
            key_path: Option<&NSString>,
            _object: Option<&NSObject>,
            _change: Option<&Object>,
            context: *mut c_void,
        ) {
            if let Some(msg) = key_path {
                let key = autoreleasepool(|pool| msg.as_str(pool).to_owned());

                if key == "state" {
                    let vm: &mut VirtualMachine = &mut *(context as *mut VirtualMachine);
                    let _ = vm.state_notifications.try_recv();
                    // TODO: There's a race here which could potentially cause us to mis updates. And potentially send a particular state change twice.
                    println!("vm.state(): {:?}", vm.state());
                    vm.notifier.send(vm.state()).expect("Failed to send VM state");
                }
            }
        }
    }
);

unsafe impl NSObjectProtocol for VirtualMachineStateObserver {}

unsafe impl Send for VirtualMachineStateObserver {}

unsafe impl Sync for VirtualMachineStateObserver {}

impl VirtualMachineStateObserver {
    pub fn new() -> Id<Self, Shared> {
        unsafe { msg_send_id![Self::alloc(), init] }
    }
}
*/