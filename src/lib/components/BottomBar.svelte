<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  import { save } from "@tauri-apps/plugin-dialog";
  import type { Region } from "$lib/types";

  export let audioFile: string | null = null;
  export let toUrl: (path: string) => string;
  export let fileName: (path: string) => string;
  export let audioTime = 0;
  export let formatSeconds: (value?: number) => string;

  export let onPlay: () => void;
  export let onPause: () => void;
  export let onEnded: () => void;
  export let bindAudio: (el: HTMLAudioElement | null) => void;

  export let scoreImages: string[] = [];
  export let scorePdf: string | null = null;
  export let regions: Region[] = [];

  let audioEl: HTMLAudioElement | null = null;
  let renderProgress: number = 0;
  let unlisten: (() => void) | undefined;

  let showRenderModal = false;
  let outputPath = "";
  let targetWidth = 1920;
  let targetHeight = 1080;
  let videoEncoder = "libx264";
  let audioEncoder = "aac";
  let audioDuration = 0;
  let renderStatus = "";
  let renderError = "";
  let isRendering = false;

  $: renderProgressValue = Number.isFinite(renderProgress)
    ? Math.min(100, Math.max(0, renderProgress))
    : 0;
  $: renderProgressLabel = `${Math.round(renderProgressValue)}%`;

  $: bindAudio(audioEl);

  function updateAudioDuration() {
    audioDuration =
      audioEl && Number.isFinite(audioEl.duration)
        ? Number(audioEl.duration.toFixed(3))
        : 0;
  }

  $: if (audioEl) {
    updateAudioDuration();
  } else {
    audioDuration = 0;
  }

  const encoderOptions = [
    { label: "H.264 (libx264)", value: "libx264" },
    { label: "H.265 (libx265)", value: "libx265" },
    { label: "NVIDIA H.264 (h264_nvenc)", value: "h264_nvenc" },
    { label: "NVIDIA H.265 (hevc_nvenc)", value: "hevc_nvenc" },
  ];

  const audioEncoderOptions = [
    { label: "AAC (aac)", value: "aac" },
    { label: "MP3 (libmp3lame)", value: "libmp3lame" },
    { label: "Copy (copy)", value: "copy" },
  ];

  $: canRenderImages = scoreImages.length > 0 && !scorePdf;
  $: hasTimedRegions = regions.some((region) => region.start != null);
  $: canStartRender =
    !isRendering &&
    !!audioFile &&
    canRenderImages &&
    hasTimedRegions &&
    outputPath.trim().length > 0 &&
    Number.isFinite(targetWidth) &&
    targetWidth > 0 &&
    Number.isFinite(targetHeight) &&
    targetHeight > 0 &&
    Number.isFinite(audioDuration) &&
    audioDuration > 0;

  const eventHandler = (event: { payload: number }) => {
    console.log("Event received:", event.payload);
    renderProgress = event.payload;
  };

  onMount(async () => {
    unlisten = await listen<number>("render-progress", eventHandler);
  });

  onDestroy(() => {
    if (unlisten) {
      unlisten();
    }
  });

  function openRenderModal() {
    showRenderModal = true;
    renderStatus = "";
    renderError = "";
    if (!outputPath && audioFile) {
      outputPath = audioFile.replace(/\.[^.]+$/, ".mp4");
    }
  }

  function closeRenderModal() {
    showRenderModal = false;
  }

  async function pickOutputPath() {
    const selected = await save({
      title: "保存渲染视频",
      filters: [{ name: "MP4", extensions: ["mp4"] }],
      defaultPath: outputPath || undefined,
    });
    if (selected) {
      outputPath = selected;
    }
  }

  async function startRender() {
    if (!audioFile) return;
    renderError = "";
    renderStatus = "";
    if (!canRenderImages) {
      renderError = "请先选择图片谱面";
      return;
    }
    if (!hasTimedRegions) {
      renderError = "请先为区域设置开始时间";
      return;
    }
    if (!outputPath.trim()) {
      renderError = "请选择输出路径";
      return;
    }

    const width = Math.floor(Number(targetWidth));
    const height = Math.floor(Number(targetHeight));
    if (
      !Number.isFinite(width) ||
      width <= 0 ||
      !Number.isFinite(height) ||
      height <= 0
    ) {
      renderError = "目标分辨率无效";
      return;
    }

    const duration = Number(audioDuration);
    if (!Number.isFinite(duration) || duration <= 0) {
      renderError = "渲染时长无效";
      return;
    }

    const payloadRegions = regions.map((region) => ({
      id: region.id,
      x: region.x,
      y: region.y,
      width: region.width,
      height: region.height,
      label: region.label,
      color: region.color,
      page_index: region.pageIndex,
      start: region.start ?? null,
    }));

    const sortedRegions = payloadRegions
      .filter((region) => region.start != null)
      .sort((a, b) => (a.start ?? 0) - (b.start ?? 0));

    if (sortedRegions[0].start! > 0) {
      renderError = "请确保第一个区域的开始时间设置为 0";
      return;
    }

    isRendering = true;
    renderStatus = "开始渲染...";
    try {
      await invoke("render_video", {
        outputPath: outputPath,
        musicPath: audioFile,
        imagePaths: scoreImages,
        regions: payloadRegions,
        duration,
        targetWidth: width,
        targetHeight: height,
        videoEncoder: videoEncoder,
        audioEncoder: audioEncoder,
      });
      renderStatus = "渲染完成";
    } catch (err) {
      renderStatus = "";
      renderError =
        typeof err === "string" && err.trim()
          ? err
          : "渲染失败，请检查 ffmpeg 与输出路径";
    } finally {
      isRendering = false;
    }
  }
</script>

{#if audioFile}
  <footer class="audio-bar">
    <div class="audio-inner">
      <div class="audio-info">
        <span class="muted">当前时间：{formatSeconds(audioTime)}</span>
        <span class="muted">{fileName(audioFile)}</span>
      </div>
      <audio
        controls
        src={toUrl(audioFile)}
        class="audio"
        bind:this={audioEl}
        on:loadedmetadata={updateAudioDuration}
        on:durationchange={updateAudioDuration}
        on:play={onPlay}
        on:pause={onPause}
        on:ended={onEnded}
      ></audio>
      <button
        type="button"
        class="render-button"
        on:click={openRenderModal}
        disabled={!canRenderImages}
        title={!canRenderImages ? "请先选择图片谱面（PDF 暂不支持）" : ""}
      >
        渲染视频
      </button>
    </div>
  </footer>
{/if}

{#if showRenderModal}
  <div
    class="modal-backdrop"
    on:click={() => {
      if (!isRendering) closeRenderModal();
    }}
  >
    <div class="modal" role="dialog" aria-modal="true" on:click|stopPropagation>
      <div class="modal-header">
        <div>
          <h3>渲染视频</h3>
          <p class="muted">设置输出路径与渲染参数</p>
        </div>
        <button type="button" class="ghost" on:click={closeRenderModal}
          >关闭</button
        >
      </div>
      <div class="modal-body">
        <div class="modal-row">
          <label>输出文件</label>
          <input
            class="text"
            type="text"
            placeholder="请选择保存路径"
            value={outputPath}
            style="width: 90%;"
            on:input={(e) =>
              (outputPath = (e.currentTarget as HTMLInputElement).value)}
          />
          <button type="button" class="ghost" on:click={pickOutputPath}
            >选择</button
          >
        </div>

        <div class="modal-row">
          <label>分辨率</label>
          <div class="modal-grid">
            <input
              type="number"
              min="1"
              step="1"
              value={targetWidth}
              style="max-width: 100px"
              on:input={(e) =>
                (targetWidth = Number(
                  (e.currentTarget as HTMLInputElement).value,
                ))}
              aria-label="宽度"
            />
            <input
              type="number"
              min="1"
              step="1"
              value={targetHeight}
              style="max-width: 100px"
              on:input={(e) =>
                (targetHeight = Number(
                  (e.currentTarget as HTMLInputElement).value,
                ))}
              aria-label="高度"
            />
          </div>
        </div>

        <div class="modal-row">
          <label>视频编码</label>
          <select
            value={videoEncoder}
            on:change={(e) =>
              (videoEncoder = (e.currentTarget as HTMLSelectElement).value)}
          >
            {#each encoderOptions as option}
              <option value={option.value}>{option.label}</option>
            {/each}
          </select>
        </div>

        <div class="modal-row">
          <label>音频编码</label>
          <select
            value={audioEncoder}
            on:change={(e) =>
              (audioEncoder = (e.currentTarget as HTMLSelectElement).value)}
          >
            {#each audioEncoderOptions as option}
              <option value={option.value}>{option.label}</option>
            {/each}
          </select>
        </div>

        <div class="modal-row progress-row">
          <label>渲染进度</label>
          <div
            class="progress-track"
            role="progressbar"
            aria-valuemin="0"
            aria-valuemax="100"
            aria-valuenow={renderProgressValue}
            aria-label="渲染进度"
          >
            <div
              class="progress-fill"
              style={`width: ${renderProgressValue}%`}
            ></div>
          </div>
          <span class="muted">{renderProgressLabel}</span>
        </div>

        <div class="modal-row">
          <label>内容摘要</label>
          <span class="muted">
            图片 {scoreImages.length} 张，区域 {regions.length} 个
          </span>
        </div>

        {#if renderError}
          <p class="form-error">{renderError}</p>
        {/if}
        {#if renderStatus}
          <p class="form-status">{renderStatus}</p>
        {/if}
      </div>
      <div class="modal-actions">
        <button
          type="button"
          class="ghost"
          on:click={closeRenderModal}
          disabled={isRendering}
        >
          取消
        </button>
        <button type="button" on:click={startRender} disabled={!canStartRender}>
          {isRendering ? `渲染中...` : "开始渲染"}
        </button>
      </div>
    </div>
  </div>
{/if}
