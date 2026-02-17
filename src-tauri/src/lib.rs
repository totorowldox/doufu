// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod types;
use crate::types::Region;
use std::io::{BufRead, BufReader, Write};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};
use tauri::{Emitter, Window};
use tempfile::NamedTempFile;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![greet, render_video])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn render_video(
    window: Window, // 自动注入当前窗口
    output_path: String,
    music_path: String,
    image_paths: Vec<String>,
    regions: Vec<Region>,
    duration: f64,
    target_width: u32,
    target_height: u32,
    video_encoder: String,
    audio_encoder: String,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        render_video_internal(
            &window,
            &output_path,
            &music_path,
            image_paths,
            regions,
            duration,
            target_width,
            target_height,
            &video_encoder,
            &audio_encoder,
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

fn render_video_internal(
    window: &Window,
    output_path: &str,
    music_path: &str,
    image_paths: Vec<String>,
    regions: Vec<Region>,
    vid_duration: f64,
    target_width: u32,
    target_height: u32,
    video_encoder: &str,
    audio_encoder: &str,
) -> Result<(), String> {
    // 1. 数据准备与校验
    // 过滤掉没有开始时间的 Region，并按开始时间排序
    let mut sorted_regions: Vec<Region> =
        regions.into_iter().filter(|r| r.start.is_some()).collect();
    sorted_regions.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());

    if sorted_regions.is_empty() {
        return Err("No regions with valid start time found".into());
    }

    // 校验 page_index 是否越界
    let num_images = image_paths.len();
    for region in &sorted_regions {
        if region.page_index as usize >= num_images {
            return Err(format!(
                "Region {} refers to invalid page_index {}",
                region.id, region.page_index
            ));
        }
    }

    // 2. 构建 FFmpeg 基础命令
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y");
    cmd.arg("-hide_banner").arg("-loglevel").arg("warning");
    cmd.arg("-progress").arg("pipe:1");

    // 添加所有图片作为输入源
    // 使用 -loop 1 将图片作为无限循环的视频流输入，方便后续 trim 截取
    for path in &image_paths {
        cmd.arg("-loop").arg("1").arg("-i").arg(path);
    }

    cmd.arg("-i").arg(music_path);
    let audio_input_index = num_images;

    // 3. 构建核心滤镜链 (Filter Complex)
    let mut filter_chains = Vec::new();
    let total_regions = sorted_regions.len();

    // 遍历每个 Region，构建处理单个片段的滤镜链
    for (i, region) in sorted_regions.iter().enumerate() {
        // 计算持续时间
        let start_time = region.start.unwrap();
        let end_time = if i + 1 < total_regions {
            sorted_regions[i + 1].start.unwrap()
        } else {
            vid_duration
        };
        let duration = end_time - start_time;

        // 确保数值在合理范围内 (防止裁剪出界)
        let crop_w = region.width.max(0.001).min(1.0);
        let crop_h = region.height.max(0.001).min(1.0);
        // 确保 x + width 不超过 1.0
        let crop_x = region.x.max(0.0).min(1.0 - crop_w);
        let crop_y = region.y.max(0.0).min(1.0 - crop_h);

        // 构建单个片段的滤镜串
        // [N:v] -> crop -> scale -> pad -> trim -> setpts -> [segN]
        let segment_filter = format!(
            "[{input_idx}:v]crop=iw*{w:.4}:ih*{h:.4}:iw*{x:.4}:ih*{y:.4},\
             scale={tw}:{th}:force_original_aspect_ratio=decrease,\
             pad={tw}:{th}:(ow-iw)/2:(oh-ih)/2:white,\
             trim=duration={dur:.3},setpts=PTS-STARTPTS,setsar=1[seg{seg_idx}]",
            input_idx = region.page_index,
            w = crop_w,
            h = crop_h,
            x = crop_x,
            y = crop_y,
            tw = target_width,
            th = target_height,
            dur = duration,
            seg_idx = i
        );
        filter_chains.push(segment_filter);
    }

    // 构建拼接 (concat) 滤镜
    // 将上面生成的所有 [seg0][seg1]... 拼接起来
    let concat_inputs: String = (0..total_regions).map(|i| format!("[seg{}]", i)).collect();

    // concat=n={}:v=1:a=0 表示拼接N个视频流，0个音频流 (音频直接用背景音乐)
    let concat_filter = format!("{}concat=n={}:v=1:a=0[outv]", concat_inputs, total_regions);
    filter_chains.push(concat_filter);

    // 连接所有滤镜链，写入临时文件
    let final_filter_complex = filter_chains.join(";\n");

    let mut filter_script =
        NamedTempFile::new().map_err(|e| format!("Failed to create temp filter file: {}", e))?;
    write!(filter_script, "{}", final_filter_complex)
        .map_err(|e| format!("Failed to write filter script: {}", e))?;
    let filter_script_path = filter_script.path().to_str().ok_or("Invalid temp path")?;

    // 优化：避免 filter 链太长超出命令行长度限制
    cmd.arg("-filter_complex_script").arg(filter_script_path);

    // 4. 配置输出参数并执行
    // 映射最终的视频流 [outv]
    cmd.arg("-map")
        .arg("[outv]")
        // 映射背景音乐音频流
        .arg("-map")
        .arg(format!("{}:a", audio_input_index))
        // 视频编码
        .arg("-c:v")
        .arg(video_encoder);

    if video_encoder.contains("nvenc") {
        cmd.arg("-preset").arg("p4").arg("-cq").arg("23");
    } else {
        cmd.arg("-preset").arg("medium").arg("-crf").arg("23");
    }

    cmd.arg("-pix_fmt")
        .arg("yuv420p")
        // 音频编码
        .arg("-c:a")
        .arg(audio_encoder)
        .arg("-b:a")
        .arg("192k")
        // 其他通用设置
        .arg("-shortest")
        .arg(output_path);

    println!("Executing FFmpeg command...");
    println!("Full Command: {:?}", cmd);

    // 捕获输出
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    const CREATE_NO_WINDOW: u32 = 0x08000000;
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to start ffmpeg: {}", e))?;
    let stdout = child.stdout.take().ok_or("Failed to open stdout")?;
    let reader = BufReader::new(stdout);

    // 解析进度
    for line in reader.lines() {
        if let Ok(l) = line {
            if l.starts_with("out_time_us=") {
                let us_str = l.replace("out_time_us=", "");
                if let Ok(us) = us_str.trim().parse::<f64>() {
                    let current_seconds = us / 1_000_000.0;
                    let percentage = (current_seconds / vid_duration * 100.0).min(100.0);

                    // 回传给前端
                    let _ = window.emit("render-progress", percentage);
                }
            }
            if l == "progress=end" {
                let _ = window.emit("render-progress", 100.0);
            }
        }
    }

    let status = child.wait().map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        // 也可以从 child.stderr 获取详细错误
        Err("FFmpeg render failed".into())
    }
}
