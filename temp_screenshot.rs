fn capture_and_save_screenshot(&mut self, save_path: &PathBuf) {
    self.capture_screenshot_uri(save_path);
}

fn capture_screenshot_uri(&mut self, save_path: &PathBuf) {
    let video_uri = self.video.pipeline().property::<String>("current-uri");
    let current_position = self.position;

    println!("Capturing screenshot at position: {:.2}s", current_position);
    println!("Video URI: {}", video_uri);
    println!("Saving to: {:?}", save_path);

    let decoded_path = if video_uri.starts_with("file://") {
        match url::percent_decode(&video_uri[7..]) {
            Ok(decoded) => format!("file://{}", decoded),
            Err(e) => {
                eprintln!("Failed to decode URI: {}", e);
                return;
            }
        }
    } else {
        video_uri.clone()
    };

    let pipeline_str = format!(
        "gst-launch-1.0 playbin uri={} ! videoconvert ! pngenc ! filesink location={}",
        decoded_path,
        save_path.to_string_lossy()
    );

    println!("Screenshot command: {}", pipeline_str);

    let result = std::process::Command::new("gst-launch-1.0")
        .args(&[
            "playbin",
            &format!("uri={}", decoded_path),
            "!",
            "videoconvert",
            "!",
            "pngenc",
            "!",
            &format!("filesink location={}", save_path.to_string_lossy()),
        ])
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                println!("Screenshot saved successfully to: {:?}", save_path);
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("Screenshot failed: {}", stderr);
            }
        }
        Err(e) => {
            eprintln!("Failed to run gst-launch-1.0: {:?}", e);
        }
    }
}

fn capture_screenshot_direct(&mut self, save_path: &PathBuf) {
    let video_path = self.video_url.to_string_lossy();
    let current_position = self.position;

    println!("Capturing screenshot at position: {:.2}s", current_position);
    println!("Video path: {}", video_path);
    println!("Saving to: {:?}", save_path);

    let pipeline_str = format!(
        "gst-launch-1.0 playbin uri=file://{} ! videoconvert ! pngenc ! filesink location={}",
        video_path,
        save_path.to_string_lossy()
    );

    println!("Screenshot command: {}", pipeline_str);

    let result = std::process::Command::new("gst-launch-1.0")
        .args(&[
            "playbin",
            &format!("uri=file://{}", video_path),
            "!",
            "videoconvert",
            "!",
            "pngenc",
            "!",
            &format!("filesink location={}", save_path.to_string_lossy()),
        ])
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                println!("Screenshot saved successfully to: {:?}", save_path);
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("Screenshot failed: {}", stderr);
            }
        }
        Err(e) => {
            eprintln!("Failed to run gst-launch-1.0: {:?}", e);
        }
    }
}
