/*

*/
#[allow(dead_code)]
pub trait SwindleStateTrait {
    fn start_dhcp(&self);
    fn start_ble_provisioning(&self);
    fn start_sockets(&self);
    fn stop_sockets(&self);
    fn start_swindle(&self);
    fn stop_swindle(&self);
    fn has_provisioning(&self) -> bool;
    fn reset_provisioning(&self);
}
