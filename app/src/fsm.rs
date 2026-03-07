#![allow(dead_code)]

use num_derive::{FromPrimitive, ToPrimitive};
pub use num_traits::FromPrimitive;
use rust_sfsm::{StateBehavior, rust_sfsm};
// StateMachine
use crate::fsm_led;
use crate::settings;
use crate::swindle_state_trait::SwindleStateTrait;
/// List of protocol states.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum MachineStates {
    #[default]
    Idle,
    //Start,
    DhcpIng,
    Configuring,
    Waiting,
    Attached,
    Reset,
}

/// List of protocol events.
#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq)]
pub enum SwindleEvents {
    Start = 1,
    Configured,
    Configuring,
    IpReady,
    Attach,
    Detach,
    NetworkLoss,
    ResetConfiguration,
}

/// SwindleState state machine context (data shared between states).
//#[derive(Default)]
pub struct MachineContext {
    executer: &'static dyn SwindleStateTrait,
    pub outbox: Option<SwindleEvents>,
}

impl MachineContext {
    //fn new() -> Self {
    //let m: MachineContext = MachineContext {};
    //m
    //}
    fn emit(&mut self, event: SwindleEvents) {
        if self.outbox.is_some() {
            panic!("outbox is full");
        }
        self.outbox = Some(event);
    }
    fn start_dhcp(&self) {
        println!("start_dhcp");
        fsm_led::set_color(settings::WS2812_DHCPING);
        self.executer.start_dhcp();
    }
    fn reset_provisioning(&self) {
        println!("reset_provisioning");
        self.executer.reset_provisioning();
    }
    fn start_ble_provisioning(&self) {
        println!("start_ble_prov");
        fsm_led::set_color(settings::WS2812_PROVISIONING);
        self.executer.start_ble_provisioning();
    }
    fn start_sockets(&self) {
        println!("start sockets");
        self.executer.start_sockets();
    }
    fn stop_sockets(&self) {
        println!("stop sockets");
    }
    fn start_swindle(&self) {
        println!("start swindle");
    }
    fn stop_swindle(&self) {
        println!("stop swindle");
    }
}

impl StateBehavior for MachineStates {
    type State = Self;
    type Event<'a> = SwindleEvents;
    type Context = MachineContext;

    fn enter(&self, _context: &mut Self::Context) {
        println!("====> [{:?}]", self);
    }

    fn handle_event(
        &self,
        event: &Self::Event<'_>,
        context: &mut Self::Context,
    ) -> Option<Self::State> {
        println!("[{:?}]<==== {:?}", self, event);
        match (self, event) {
            (&MachineStates::Idle, &SwindleEvents::ResetConfiguration) => {
                context.reset_provisioning();
                Some(MachineStates::Reset)
            }
            (&MachineStates::Idle, &SwindleEvents::Start) => {
                if context.executer.has_provisioning() {
                    context.start_dhcp();
                    Some(MachineStates::DhcpIng)
                } else {
                    context.start_ble_provisioning();
                    Some(MachineStates::Configuring)
                }
            }
            (&MachineStates::Configuring, &SwindleEvents::Configured) => {
                context.start_dhcp();
                Some(MachineStates::DhcpIng)
            }
            (&MachineStates::DhcpIng, &SwindleEvents::IpReady) => {
                context.start_sockets();
                Some(MachineStates::Waiting)
            }
            (&MachineStates::DhcpIng, &SwindleEvents::NetworkLoss) => {
                context.emit(SwindleEvents::Start);
                Some(MachineStates::Idle)
            }
            (&MachineStates::Waiting, &SwindleEvents::Attach) => {
                context.start_swindle();
                Some(MachineStates::Attached)
            }
            (&MachineStates::Attached, &SwindleEvents::Detach) => {
                context.stop_sockets();
                Some(MachineStates::Waiting)
            }
            (&MachineStates::Attached, &SwindleEvents::NetworkLoss) => {
                // todo:
                context.stop_sockets();
                Some(MachineStates::DhcpIng)
            }
            (&MachineStates::Waiting, &SwindleEvents::NetworkLoss) => {
                // todo:
                context.stop_sockets();
                Some(MachineStates::DhcpIng)
            }
            _ => {
                println!(
                    "<<<<<<<Ignored Event {:?} in state {:?} >>>>>>>",
                    self, event
                );
                None
            }
        }
    }
}

#[rust_sfsm(states = MachineStates, context = MachineContext)]
pub struct SwindleState {}

impl SwindleState {
    pub fn take_outbox(&mut self) -> Option<SwindleEvents> {
        self.context.outbox.take()
    }
    pub fn new(executer: &'static dyn SwindleStateTrait) -> Self {
        Self {
            current_state: MachineStates::Idle,
            context: MachineContext {
                executer,
                outbox: None,
            },
        }
    }
}
//
