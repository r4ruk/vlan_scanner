mod parameters;

use config::{Config, File};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::{env, thread};
use std::time::Duration;
use chrono::Local;
use std::fs::File as FsFile;
use std::io::Write;
use std::str::FromStr;
use ipnetwork::IpNetwork;

fn main() {
    let args: Vec<String> = env::args().collect();
    let configuration = parameters::handle_arguments(args);

    let mut settings = Settings::new();
    settings.vlan_check_range_start = configuration.range_min;
    settings.vlan_check_range_start = configuration.range_min;
    settings.interface = configuration.interface;
    settings.dhcp_wait_time = configuration.wait;


    let mut vlans_with_ips = Vec::new();

    println!("wait time: {}", settings.dhcp_wait_time);

    println!("Starting checks on interface: {}", settings.interface);

    for vlan_id in settings.vlan_check_range_start..=settings.vlan_check_range_end {
        if let Some(vlan_info) = check_vlan(&settings.interface, vlan_id, settings.dhcp_wait_time) {
            vlans_with_ips.push(vlan_info);
        }
    }

    log_vlans(&vlans_with_ips);
    println!("Finished checking all VLANs.");
}

fn log_vlans(vlans: &Vec<VlanInfo>) {
    let current_datetime = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let log_file = format!("vlan_ips_{}.json", current_datetime);
    let json = serde_json::to_string_pretty(vlans).expect("Failed to serialize JSON");

    let mut file = FsFile::create(log_file).expect("Failed to create log file");
    file.write_all(json.as_bytes()).expect("Failed to write JSON to log file");
}



fn extract_ip_from_interface(interface: &str) -> Option<IpNetwork> {
    let command = format!("ip addr show {}", interface);
    if let Ok(output) = run_command(&command) {

        println!("{}", output.clone());

        // should match "inet 192.168.1.1/XX" where XX could be any number
        let re = Regex::new(r"inet (\d+\.\d+\.\d+\.\d+/\d+)").unwrap();
        if let Some(caps) = re.captures(&output) {
            let ip = caps.get(1).map_or("", |m| m.as_str());
            if !ip.starts_with("169.") {
                return match IpNetwork::from_str(ip) {
                    Ok(ipnetwork) => {
                        Some(ipnetwork)
                    }
                    _ => None
                }
            }
        }
    }
    None
}

fn check_vlan(interface: &str, vlan_id: u32, wait_time: u16) -> Option<VlanInfo> {
    let interface_vlan = format!("{}.{}", interface, vlan_id);
    println!("Checking VLAN {}...", vlan_id);

    // Create VLAN
    println!("ip link add link {} name {} type vlan id {}", interface, interface_vlan, vlan_id);
    let res = run_command(&format!("ip link add link {} name {} type vlan id {}", interface, interface_vlan, vlan_id));
    match res {
        Ok(value) => println!("Success: {}", value),
        Err(err) => println!("Error: {}", err),
    }

    println!("setting ip link up for interface vlan: {}", interface_vlan);
    let res = run_command(&format!("ip link set up {}", interface_vlan));
    match res {
        Ok(value) => println!("Success: {}", value),
        Err(err) => println!("Error: {}", err),
    }

    // Wait for DHCP
    println!("Waiting for {} seconds...", wait_time);
    thread::sleep(Duration::from_secs(wait_time as u64));

    // command which could force retrieving ip address:
    // format!("sudo dhclient {}", interface_vlan)

    // Extract IP
    if let Some(ip) = extract_ip_from_interface(&interface_vlan) {
        println!("VLAN {} has IP: {}", vlan_id, ip);
        Some(VlanInfo { vlan_id, ip_address: Some(ip.to_string())})
    } else {
        println!("NO IP");
        println!("");
        println!("");
        None
    }
}

#[derive(Debug, Serialize)]
struct VlanInfo {
    vlan_id: u32,
    ip_address: Option<String>,
}


fn run_command(command: &str) -> Result<String, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}


#[derive(Debug, Deserialize)]
struct Settings {
    interface: String,
    dhcp_wait_time: u16,
    calculate_possible_hosts: bool,
    calculate_subnet_mask: bool,
    vlan_check_range_start: u32,
    vlan_check_range_end: u32
}

impl Settings {
    fn new() -> Self {
        return Settings {
            interface: "eth1".to_string(),
            dhcp_wait_time: 3,
            calculate_possible_hosts: false,
            calculate_subnet_mask: true,
            vlan_check_range_start: 1,
            vlan_check_range_end: 4094
        }

    }
}
