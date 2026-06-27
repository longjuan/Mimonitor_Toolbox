use super::AdbResult;
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use adb_client::tcp::ADBTcpDevice;
use adb_client::ADBDeviceExt;

/// Discovered device
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiscoveredDevice {
    pub ip: String,
    pub model: String,
}

/// Get all valid LAN subnets from network interfaces.
/// Returns a list of "x.y.z" subnets, prioritized by likelihood of being the target network.
pub fn get_local_subnets() -> Vec<String> {
    let mut subnets = Vec::new();

    // Try ifconfig (macOS/Linux) or ipconfig (Windows)
    let output = if cfg!(target_os = "windows") {
        let mut cmd = std::process::Command::new("ipconfig");
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        cmd.output()
    } else {
        std::process::Command::new("ifconfig").output()
    };

    if let Ok(output) = output {
        let text = String::from_utf8_lossy(&output.stdout);

        if cfg!(target_os = "windows") {
            // Parse ipconfig output: "IPv4 Address. . . . . . . . . . . : 192.168.31.115"
            for line in text.lines() {
                let line = line.trim();
                if line.contains("IPv4") || line.contains("IP Address") {
                    if let Some(ip_str) = line.split(':').last() {
                        if let Some(subnet) = parse_ip_to_subnet(ip_str.trim()) {
                            subnets.push(subnet);
                        }
                    }
                }
            }
        } else {
            // Parse ifconfig output
            let mut current_iface = "";
            let virtual_prefixes = [
                "utun", "bridge", "docker", "vmenet", "awdl", "llw",
                "veth", "tun", "tap", "virbr", "br-", "vmnet",
            ];

            for line in text.lines() {
                // Track interface name (lines not starting with whitespace)
                if !line.starts_with(' ') && !line.starts_with('\t') && line.contains(':') {
                    current_iface = line.split(':').next().unwrap_or("");
                }

                // Skip virtual interfaces
                if virtual_prefixes.iter().any(|p| current_iface.starts_with(p)) {
                    continue;
                }

                // Parse inet lines
                if line.trim().starts_with("inet ") {
                    if let Some(ip_str) = line.trim().strip_prefix("inet ") {
                        let ip_part = ip_str.split_whitespace().next().unwrap_or("");
                        if let Some(subnet) = parse_ip_to_subnet(ip_part) {
                            subnets.push(subnet);
                        }
                    }
                }
            }
        }
    }

    // Fallback: UDP socket trick
    if subnets.is_empty() {
        if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
            if socket.connect("8.8.8.8:80").is_ok() {
                if let Ok(addr) = socket.local_addr() {
                    if let std::net::IpAddr::V4(v4) = addr.ip() {
                        let o = v4.octets();
                        subnets.push(format!("{}.{}.{}", o[0], o[1], o[2]));
                    }
                }
            }
        }
    }

    // Deduplicate and sort: prefer 192.168.x.x > 10.x.x.x > others
    subnets.sort();
    subnets.dedup();
    subnets.sort_by(|a, b| {
        let pa = subnet_priority(a);
        let pb = subnet_priority(b);
        pa.cmp(&pb)
    });

    subnets
}

/// Parse an IP string to a /24 subnet string, filtering out invalid addresses.
fn parse_ip_to_subnet(ip_str: &str) -> Option<String> {
    let ip: std::net::Ipv4Addr = ip_str.parse().ok()?;
    let o = ip.octets();

    // Skip loopback
    if o[0] == 127 {
        return None;
    }

    // Skip 198.18.x.x (common VPN/proxy range)
    if o[0] == 198 && o[1] == 18 {
        return None;
    }

    // Skip 169.254.x.x (link-local)
    if o[0] == 169 && o[1] == 254 {
        return None;
    }

    // Must be a private IP range
    let is_private = (o[0] == 192 && o[1] == 168)
        || (o[0] == 10)
        || (o[0] == 172 && o[1] >= 16 && o[1] <= 31);

    if !is_private {
        return None;
    }

    Some(format!("{}.{}.{}", o[0], o[1], o[2]))
}

/// Priority for sorting: lower = more likely to be the target network
fn subnet_priority(subnet: &str) -> u8 {
    if subnet.starts_with("192.168.") {
        0 // Most common home/office network
    } else if subnet.starts_with("10.") {
        1
    } else if subnet.starts_with("172.") {
        2
    } else {
        3
    }
}

/// Scan one /24 subnet for ADB devices using pure Rust ADB protocol
async fn scan_subnet(subnet: &str) -> Vec<DiscoveredDevice> {
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(128));
    let found = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    for i in 1..=254 {
        let ip = format!("{}.{}", subnet, i);
        let sem = semaphore.clone();
        let found = found.clone();

        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();

            // Quick TCP port check — 150ms for LAN
            let addr: SocketAddr = format!("{}:5555", ip).parse().unwrap();
            let port_open = tokio::task::spawn_blocking(move || {
                TcpStream::connect_timeout(&addr, Duration::from_millis(150)).is_ok()
            })
            .await
            .unwrap_or(false);

            if !port_open {
                return;
            }

            // Try connecting via pure Rust ADB protocol
            let result = tokio::task::spawn_blocking(move || -> Option<String> {
                let mut device = ADBTcpDevice::new(addr).ok()?;
                let mut buf = Vec::new();
                device.shell_command(
                    &"getprop ro.product.model".to_string(),
                    Some(&mut buf),
                    None,
                ).ok()?;
                let model = String::from_utf8_lossy(&buf).trim().to_string();
                if model.is_empty() { None } else { Some(model) }
            })
            .await
            .unwrap_or(None);

            if let Some(model) = result {
                found.lock().await.push(DiscoveredDevice { ip, model });
            }
        }));
    }

    for h in handles {
        let _ = h.await;
    }

    let result = found.lock().await.clone();
    result
}

/// Scan local network for ADB devices.
/// Scans all detected subnets for devices with port 5555 open.
/// Returns (devices, scanned_subnets) for logging purposes.
pub async fn scan_network(_subnet_hint: &str) -> AdbResult<(Vec<DiscoveredDevice>, Vec<String>)> {
    let mut found = Vec::new();

    // Scan all detected subnets
    let subnets = get_local_subnets();
    log::info!("Scanning subnets: {:?}", subnets);

    for subnet in &subnets {
        log::info!("Scanning {}.1-254 ...", subnet);
        let devices = scan_subnet(subnet).await;
        if !devices.is_empty() {
            log::info!("Found {} device(s) in {}", devices.len(), subnet);
        }
        found.extend(devices);
    }

    Ok((found, subnets))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_local_subnets_returns_valid() {
        let subnets = get_local_subnets();
        for s in &subnets {
            let parts: Vec<&str> = s.split('.').collect();
            assert_eq!(parts.len(), 3, "Subnet should have 3 octets: {}", s);
            for part in &parts {
                part.parse::<u8>().expect("Each octet should be a number");
            }
        }
    }

    #[test]
    fn test_parse_ip_to_subnet_filters_invalid() {
        assert_eq!(parse_ip_to_subnet("127.0.0.1"), None); // loopback
        assert_eq!(parse_ip_to_subnet("198.18.0.1"), None); // VPN
        assert_eq!(parse_ip_to_subnet("169.254.1.1"), None); // link-local
        assert_eq!(parse_ip_to_subnet("8.8.8.8"), None); // public
        assert_eq!(parse_ip_to_subnet("192.168.31.115"), Some("192.168.31".to_string()));
        assert_eq!(parse_ip_to_subnet("10.0.0.5"), Some("10.0.0".to_string()));
        assert_eq!(parse_ip_to_subnet("172.16.0.1"), Some("172.16.0".to_string()));
    }

    #[test]
    fn test_subnet_priority() {
        assert!(subnet_priority("192.168.1.0") < subnet_priority("10.0.0.0"));
        assert!(subnet_priority("10.0.0.0") < subnet_priority("172.16.0.0"));
    }

    #[test]
    fn test_discovered_device_serialization() {
        let device = DiscoveredDevice {
            ip: "192.168.31.252".to_string(),
            model: "MiTV-MFFU1".to_string(),
        };
        let json = serde_json::to_string(&device).unwrap();
        assert!(json.contains("192.168.31.252"));
        assert!(json.contains("MiTV-MFFU1"));

        let loaded: DiscoveredDevice = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.ip, "192.168.31.252");
        assert_eq!(loaded.model, "MiTV-MFFU1");
    }
}
