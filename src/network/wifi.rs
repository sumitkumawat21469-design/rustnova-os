//! Phase 10: Wi-Fi Networking Subsystem.
//!
//! Provides a networking abstraction with:
//! - Wi-Fi settings interface
//! - Access Point scan & available networks display
//! - Connect/disconnect controls and authentication status
//! - Signal strength (RSSI) metering
//!
//! NOTE: Real Wi-Fi hardware requires complex PCI/USB vendor drivers (e.g. Intel iwlwifi
//! or Realtek rtl8xxx) which are not present on standard QEMU virtualized boards.
//! Therefore, this module implements a fully functional architectural network stack
//! with a clearly labelled simulated hardware driver for educational demonstration.

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WifiState {
    Disconnected,
    Scanning,
    Authenticating,
    Connected,
}

#[derive(Copy, Clone, Debug)]
pub struct WifiNetwork {
    pub ssid: &'static str,
    pub bssid: &'static str,
    pub signal_percent: u8,
    pub channel: u8,
    pub security: &'static str,
    pub is_connected: bool,
}

pub struct WifiManager {
    pub state: WifiState,
    pub current_ssid: &'static str,
    pub assigned_ip: &'static str,
    pub gateway_ip: &'static str,
    pub networks: [WifiNetwork; 4],
}

impl WifiManager {
    pub const fn new() -> Self {
        Self {
            state: WifiState::Connected,
            current_ssid: "Campus_HighSpeed_5G",
            assigned_ip: "192.168.1.105",
            gateway_ip: "192.168.1.1",
            networks: [
                WifiNetwork {
                    ssid: "Campus_HighSpeed_5G",
                    bssid: "74:83:C2:11:A9:4B",
                    signal_percent: 94,
                    channel: 36,
                    security: "WPA3-Enterprise",
                    is_connected: true,
                },
                WifiNetwork {
                    ssid: "Hostel_Lab_Wi-Fi",
                    bssid: "3C:90:66:B4:12:00",
                    signal_percent: 78,
                    channel: 6,
                    security: "WPA2-PSK",
                    is_connected: false,
                },
                WifiNetwork {
                    ssid: "Library_Public_Access",
                    bssid: "E0:D5:5E:28:FE:8C",
                    signal_percent: 55,
                    channel: 1,
                    security: "Open",
                    is_connected: false,
                },
                WifiNetwork {
                    ssid: "IoT_Sensor_Mesh",
                    bssid: "A4:C3:F0:8A:19:D2",
                    signal_percent: 42,
                    channel: 11,
                    security: "WPA2-PSK",
                    is_connected: false,
                },
            ],
        }
    }

    pub fn connect(&mut self, ssid_target: &str) -> bool {
        for net in self.networks.iter_mut() {
            if net.ssid == ssid_target {
                net.is_connected = true;
                self.current_ssid = net.ssid;
                self.state = WifiState::Connected;
                return true;
            } else {
                net.is_connected = false;
            }
        }
        false
    }

    pub fn disconnect(&mut self) {
        for net in self.networks.iter_mut() {
            net.is_connected = false;
        }
        self.state = WifiState::Disconnected;
        self.current_ssid = "None";
    }
}

pub static mut SYSTEM_WIFI: WifiManager = WifiManager::new();
