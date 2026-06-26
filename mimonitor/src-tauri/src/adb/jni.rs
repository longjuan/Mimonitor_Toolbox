use super::{AdbClient, AdbError, AdbResult};

const JAR_NAME: &str = "MonitorTool.jar";
const JAR_REMOTE: &str = "/sdcard/MonitorTool.jar";
const JAR_CACHE: &str = "/data/data/mitv.service/cache/MonitorTool.jar";

/// Ensure MonitorTool.jar is deployed to the device
pub fn ensure_jar(adb: &AdbClient) -> AdbResult<bool> {
    let size_output = adb.shell(&format!("stat -c %s {} 2>/dev/null || echo 0", JAR_REMOTE))?;
    let size: i64 = size_output
        .lines()
        .last()
        .unwrap_or("0")
        .trim()
        .parse()
        .unwrap_or(0);

    if size < 1000 {
        let local_jar = find_jar()?;
        adb.push(&local_jar, JAR_REMOTE)?;
        log::info!("Pushed {} to device", JAR_NAME);
    }

    adb.shell(&format!(
        "service call TvService 3 s16 \"cp {} {}\"",
        JAR_REMOTE, JAR_CACHE
    ))?;
    Ok(true)
}

fn find_jar() -> AdbResult<String> {
    let candidates = vec![
        format!("resources/{}", JAR_NAME),
        format!("../resources/{}", JAR_NAME),
        format!("../../mimonitor/resources/{}", JAR_NAME),
        format!("assets/runtime/{}", JAR_NAME),
        format!("../assets/runtime/{}", JAR_NAME),
    ];
    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }
    Err(AdbError::JniFailed(format!("{} not found locally", JAR_NAME)))
}

/// Run a MonitorTool command (JNI or LED)
pub fn run_command(adb: &AdbClient, args: &[&str]) -> AdbResult<String> {
    ensure_jar(adb)?;

    let parts: Vec<String> = std::iter::once("MonitorTool".to_string())
        .chain(args.iter().map(|s| s.to_string()))
        .collect();
    let cmd_args: String = parts
        .iter()
        .map(|p| format!("\\${{IFS}}{}", p))
        .collect();
    let cmd = format!(
        "service call TvService 3 s16 \"sh -c eval\\${{IFS}}CLASSPATH={}\\${{IFS}}/system/bin/app_process\\${{IFS}}/data/data/mitv.service/cache{}\"",
        JAR_CACHE, cmd_args
    );
    adb.shell(&cmd)
}

/// Get a JNI config value via logcat
pub fn jni_get(adb: &AdbClient, key: &str) -> AdbResult<String> {
    ensure_jar(adb)?;

    adb.shell("logcat -c")?;
    run_command(adb, &["get", key])?;

    std::thread::sleep(std::time::Duration::from_millis(800));

    let log = adb.shell(&format!("logcat -d | grep 'GET {}' | tail -1", key))?;
    let val = if let Some(pos) = log.find("= ") {
        log[pos + 2..].trim().to_string()
    } else {
        "N/A".to_string()
    };
    log::debug!("jni_get {} => {}", key, val);
    Ok(val)
}

/// Set a JNI config value
pub fn jni_set(adb: &AdbClient, key: &str, value: i32, is_update: i32) -> AdbResult<String> {
    log::debug!("jni_set {} = {}", key, value);
    run_command(adb, &["set", key, &value.to_string(), &is_update.to_string()])
}

/// Set RGB color gains atomically
pub fn jni_set_color_gains(adb: &AdbClient, r: i32, g: i32, b: i32) -> AdbResult<String> {
    log::debug!("jni_set_color_gains r={} g={} b={}", r, g, b);
    run_command(adb, &["setColorGains", &r.to_string(), &g.to_string(), &b.to_string()])
}

/// LED control via MonitorTool led subcommand
pub fn led_command(adb: &AdbClient, subcmd: &str, args: &[&str]) -> AdbResult<String> {
    let all_args: Vec<&str> = std::iter::once("led")
        .chain(std::iter::once(subcmd))
        .chain(args.iter().copied())
        .collect();
    run_command(adb, &all_args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_jar_not_found() {
        let result = find_jar();
        assert!(result.is_err());
    }

    #[test]
    fn test_jni_set_calls_adb() {
        let client = AdbClient::new("127.0.0.1");
        let result = jni_set(&client, "g_disp__disp_back_light", 50, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_jni_set_color_gains_calls_adb() {
        let client = AdbClient::new("127.0.0.1");
        let result = jni_set_color_gains(&client, 1024, 1024, 1024);
        assert!(result.is_err());
    }

    #[test]
    fn test_led_command_calls_adb() {
        let client = AdbClient::new("127.0.0.1");
        let _ = led_command(&client, "off", &[]);
    }
}
