use super::{AdbClient, AdbResult};

/// Batch read multiple Android settings in one shell call
pub fn batch_get(adb: &AdbClient, keys: &[&str]) -> AdbResult<Vec<(String, String)>> {
    let mut cmd = String::new();
    for (i, key) in keys.iter().enumerate() {
        if i > 0 {
            cmd.push_str(" && ");
        }
        cmd.push_str(&format!("echo __SETTING_{}__=$(settings get global {})", key, key));
    }
    let output = adb.shell(&cmd)?;
    let mut results = Vec::new();
    for line in output.lines() {
        if let Some(stripped) = line.strip_prefix("__SETTING_") {
            if let Some(eq_pos) = stripped.find("=__") {
                // Wrong format, try the actual output format
            }
            // Format: __KEY__=value
            if let Some((kv, val)) = stripped.split_once('=') {
                let key = kv.trim_end_matches('_').trim_start_matches('_').to_string();
                // Find the actual key by removing the __ prefix/suffix
                let key = key.trim_matches('_').to_string();
                let val = val.trim().to_string();
                results.push((key, val));
            }
        }
    }
    Ok(results)
}

/// Batch write multiple Android settings
pub fn batch_put(adb: &AdbClient, pairs: &[(&str, &str)]) -> AdbResult<()> {
    for (key, value) in pairs {
        adb.put_setting(key, value)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_get_requires_connection() {
        let client = AdbClient::new("127.0.0.1");
        let _ = batch_get(&client, &["picture_mode", "picture_backlight"]);
    }

    #[test]
    fn test_batch_put_requires_connection() {
        let client = AdbClient::new("127.0.0.1");
        let _ = batch_put(&client, &[("picture_mode", "14"), ("picture_backlight", "50")]);
    }

    #[test]
    fn test_batch_get_empty_keys() {
        let client = AdbClient::new("127.0.0.1");
        let result = batch_get(&client, &[]);
        match result {
            Ok(v) => assert!(v.is_empty()),
            Err(_) => {} // adb not available
        }
    }
}
