<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  import BottomBar from "$lib/components/BottomBar.svelte";
  import HeaderBar from "$lib/components/HeaderBar.svelte";
  import ImageThumbs from "$lib/components/ImageThumbs.svelte";
  import ImageViewer from "$lib/components/ImageViewer.svelte";
  import MessageBanner from "$lib/components/MessageBanner.svelte";
  import PdfViewer from "$lib/components/PdfViewer.svelte";
  import RegionPanel from "$lib/components/RegionPanel.svelte";

  import type { Region, RegionFilter, Selection } from "$lib/types";
  import {
    startAudioTimer,
    stopAudioTimer,
    getCurrentPlayingRegion,
  } from "$lib/audio";
  import { createRegion } from "$lib/regions";
  import {
    audioFilter,
    clamp,
    clampRegion,
    fileName,
    formatSeconds,
    imageFilter,
    pdfFilter,
    REGION_COLOR,
  } from "$lib/utils";

  let scoreImages = $state<string[]>([]);
  let scorePdf = $state<string | null>(null);
  let audioFile = $state<string | null>(null);
  let message = $state("");
  let activeImageIndex = $state(0);
  let tauriReady = $state(false);

  let regions = $state<Region[]>([]);
  let selectedRegionId = $state<string | null>(null);
  let regionFilter = $state<RegionFilter>("current");
  let audioRef = $state<HTMLAudioElement | null>(null);
  let audioTime = $state(0);
  let imageSize = $state({
    naturalWidth: 0,
    naturalHeight: 0,
    displayHeight: 0,
  });

  let isPlayingState = $state(false);

  const currentRegions = $derived.by(() =>
    regions.filter((region) => region.pageIndex === activeImageIndex),
  );

  const visibleRegions = $derived.by(() =>
    regionFilter === "all" ? regions : currentRegions,
  );

  const selectedRegion = $derived.by(
    () => regions.find((region) => region.id === selectedRegionId) ?? null,
  );

  let sortedRegionsWithStart = $derived.by(() =>
    regions
      .filter((region) => region.start != null)
      .sort((a, b) => (a.start ?? 0) - (b.start ?? 0)),
  );

  const currentPlaying = $derived.by(() =>
    getCurrentPlayingRegion(sortedRegionsWithStart, audioTime),
  );

  const currentPlayingRegionId = $derived.by(() => currentPlaying?.id ?? null);

  onMount(() => {
    tauriReady = true;
  });

  const toUrl = (path: string) => convertFileSrc(path);

  $effect(() => {
    // keep active index valid
    if (activeImageIndex >= scoreImages.length) {
      activeImageIndex = 0;
    }

    if (!scoreImages.length) {
      selectedRegionId = null;
      return;
    }

    if (selectedRegionId && !selectedRegion) {
      selectedRegionId = null;
    }

    // const playing = isPlayingState;
    // const current = currentPlaying;
    // // auto page-turn: follow current playing region (across pages)
    // if (playing && current) {
    //   if (current.pageIndex !== activeImageIndex) {
    //     activeImageIndex = current.pageIndex;
    //   }
    // }

    if (selectedRegion && selectedRegion.pageIndex !== activeImageIndex) {
      selectedRegionId = currentRegions[0]?.id ?? null;
    }

    if (!selectedRegionId) {
      selectedRegionId = currentRegions[0]?.id ?? null;
    }
  });

  function handleKeyDown(e: KeyboardEvent) {
    if (e.target instanceof HTMLInputElement) return; // ignore when typing
    const availableKeys = [
      "Space",
      "ArrowLeft",
      "ArrowRight",
      "ArrowUp",
      "ArrowDown",
      "Enter",
      "KeyN",
      "KeyD",
    ];
    if (!availableKeys.includes(e.code)) return;
    if (e.repeat) return; // ignore held-down keys
    if (e.code === "Space") {
      e.preventDefault();
      if (audioRef) {
        if (audioRef.paused) {
          audioRef.play();
        } else {
          audioRef.pause();
        }
      }
    } else if (e.code === "ArrowLeft") {
      e.preventDefault();
      if (audioRef) {
        audioRef.currentTime = Math.max(0, (audioRef.currentTime ?? 0) - 5);
      }
    }
    else if (e.code === "ArrowRight") {
      e.preventDefault();
      if (audioRef) {
        audioRef.currentTime = Math.max(0, (audioRef.currentTime ?? 0) + 5);
      }
    } 
    else if (e.code === "ArrowUp") {
      e.preventDefault();
      jumpToNextRegion(-1);
    } else if (e.code === "ArrowDown") {
      e.preventDefault();
      jumpToNextRegion(1);
    }
    else if (e.code === "Enter") {
      e.preventDefault(); 
      if (selectedRegionId) {
        applyCurrentTime(selectedRegionId);
        jumpToNextRegion();
      }
    } else if (e.code === "KeyN") {
      e.preventDefault();
      addRegion();
    } else if (e.code === "KeyD") {
      e.preventDefault();
      if (selectedRegionId) removeRegion(selectedRegionId);
    }
  }

  async function pickScore() {
    message = "";
    const selection: Selection = await open({
      multiple: true,
      filters: [imageFilter, pdfFilter],
    });
    if (!selection) return;

    const files = Array.isArray(selection) ? selection : [selection];
    const hasPdf = files.some((f) => f.toLowerCase().endsWith(".pdf"));
    const hasImg = files.some((f) => /\.(png|jpe?g|webp)$/i.test(f));

    // if (hasPdf && files.length > 1) {
    //   message = "PDF 仅支持单文件";
    //   return;
    // }

    if (hasPdf) {
      message = "PDF 暂不支持，建议转换成图片后使用";
      return;
    }

    if (!hasPdf && !hasImg) {
      message = "请选择 PNG/JPG/WebP 或 PDF 文件";
      return;
    }

    if (hasPdf) {
      scorePdf = files[0];
      scoreImages = [];
      activeImageIndex = 0;
    } else {
      scoreImages = files;
      scorePdf = null;
      activeImageIndex = 0;
    }
    regions = [];
    selectedRegionId = null;
  }

  async function pickAudio() {
    message = "";
    const selection: Selection = await open({
      multiple: false,
      filters: [audioFilter],
    });
    if (!selection) return;
    audioFile = Array.isArray(selection) ? selection[0] : selection;
  }

  function countRegionsOnPage(pageIndex: number) {
    return regions.filter((region) => region.pageIndex === pageIndex).length;
  }

  function addRegion() {
    if (!scoreImages.length) return;
    const pageIndex = activeImageIndex;
    const seq = countRegionsOnPage(pageIndex) + 1;
    const region = createRegion(pageIndex, seq, REGION_COLOR);
    regions = [...regions, region];
    selectedRegionId = region.id;
  }

  function updateRegion(id: string, patch: Partial<Region>) {
    const idx = regions.findIndex((r) => r.id === id);
    if (idx === -1) return;
    const merged = { ...regions[idx], ...patch } as Region;
    const next = [...regions];
    next[idx] = clampRegion(merged);
    regions = next;
  }

  function removeRegion(id: string) {
    regions = regions.filter((r) => r.id !== id);
    if (selectedRegionId === id) {
      selectedRegionId = null;
    }
  }

  function jumpToNextRegion(direction: 1 | -1 = 1) {
    const currentIndex = regions.findIndex((r) => r.id === selectedRegionId);
    if (currentIndex === -1) return;
    const nextRegion = regions[currentIndex + direction] ?? null;
    if (nextRegion) {
      focusRegion(nextRegion);
    } else {
      // end jumping
      return;
    }
  }

  function applyCurrentTime(id: string) {
    if (!audioRef) return;
    const t = Number((audioRef.currentTime ?? 0).toFixed(3));
    updateRegion(id, { start: t });
  }

  function focusRegion(region: Region) {
    selectedRegionId = region.id;
    if (region.pageIndex !== activeImageIndex) {
      activeImageIndex = region.pageIndex;
    }
  }

  function fillWidth(region: Region) {
    updateRegion(region.id, { x: 0, width: 1 });
  }

  function exportRegion(region: Region) {
    void exportRegionAsync(region);
  }

  async function exportRegionAsync(region: Region) {
    if (scorePdf) {
      message = "PDF 暂不支持截图导出";
      return;
    }
    const imgPath = scoreImages[region.pageIndex];
    if (!imgPath) return;

    const img = new Image();
    img.crossOrigin = "anonymous";
    img.src = toUrl(imgPath);

    try {
      await img.decode();
    } catch (err) {
      message = "图片解码失败，无法导出截图";
      return;
    }

    const naturalW = img.naturalWidth || imageSize.naturalWidth;
    const naturalH = img.naturalHeight || imageSize.naturalHeight;
    if (!naturalW || !naturalH) return;

    const x = Math.floor(clamp(region.x, 0, 1) * naturalW);
    const y = Math.floor(clamp(region.y, 0, 1) * naturalH);
    const w = Math.max(
      1,
      Math.floor(Math.min(region.width, 1 - region.x) * naturalW),
    );
    const h = Math.max(
      1,
      Math.floor(Math.min(region.height, 1 - region.y) * naturalH),
    );

    const canvas = document.createElement("canvas");
    canvas.width = w;
    canvas.height = h;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.drawImage(img, x, y, w, h, 0, 0, w, h);

    canvas.toBlob((blob) => {
      if (!blob) return;
      const url = URL.createObjectURL(blob);
      const base = fileName(imgPath).replace(/\.[^.]+$/, "");
      const safeLabel = (region.label || "region").replace(/[^\w-]+/g, "_");
      const a = document.createElement("a");
      a.href = url;
      a.download = `${base}_${safeLabel}.png`;
      a.click();
      URL.revokeObjectURL(url);
    }, "image/png");
  }

  function handleImageLoad(e: Event) {
    const img = e.currentTarget as HTMLImageElement;
    imageSize = {
      naturalWidth: img.naturalWidth,
      naturalHeight: img.naturalHeight,
      displayHeight: img.clientHeight,
    };
  }

  function handleFilterChange(value: RegionFilter) {
    regionFilter = value;
  }

  function handlePlay() {
    startAudioTimer(() => {
      audioTime = audioRef?.currentTime ?? 0;
    });
    isPlayingState = true;
  }

  function handlePause() {
    stopAudioTimer();
    isPlayingState = false;
  }

  function handleEnded() {
    stopAudioTimer();
    isPlayingState = false;
  }

  function bindAudio(el: HTMLAudioElement | null) {
    audioRef = el;
  }
</script>

<svelte:window on:keydown={handleKeyDown} />
<main class="page">
  <HeaderBar
    title="加载乐谱与音频"
    hint="支持多张 PNG/JPG/WebP ， PDF 暂不支持，音频支持常见格式"
    onPickScore={pickScore}
    onPickAudio={pickAudio}
  />

  <MessageBanner {message} />

  <section class="section layout">
    <div class="viewer-col">
      <div class="section-heading">
        <h2>谱面预览</h2>
        <div class="inline-actions">
          <button
            type="button"
            on:click={addRegion}
            disabled={!scoreImages.length || !!scorePdf}
          >
            添加裁剪区域
          </button>
          <span class="muted">仅对当前图片生效</span>
        </div>
      </div>

      {#if scorePdf}
        <PdfViewer {scorePdf} {tauriReady} {toUrl} {fileName} />
      {:else if scoreImages.length}
        <ImageThumbs
          {scoreImages}
          activeIndex={activeImageIndex}
          {fileName}
          onSelect={(idx) => (activeImageIndex = idx)}
        />
        <ImageViewer
          imagePath={scoreImages[activeImageIndex]}
          regions={currentRegions}
          {selectedRegionId}
          {currentPlayingRegionId}
          onSelectRegion={(id) => (selectedRegionId = id)}
          {toUrl}
          {updateRegion}
          onImageLoad={handleImageLoad}
        />
      {:else}
        <p class="placeholder">未选择谱面文件</p>
      {/if}
    </div>

    <RegionPanel
      scoreImagesCount={scoreImages.length}
      {regions}
      {currentRegions}
      {visibleRegions}
      {selectedRegionId}
      {currentPlayingRegionId}
      {regionFilter}
      {formatSeconds}
      onFilterChange={handleFilterChange}
      onFocusRegion={focusRegion}
      onUpdateRegion={updateRegion}
      onFillWidth={fillWidth}
      onExportRegion={exportRegion}
      onRemoveRegion={removeRegion}
      onApplyCurrentTime={applyCurrentTime}
    />
  </section>
</main>

<BottomBar
  {audioFile}
  {toUrl}
  {fileName}
  {audioTime}
  {formatSeconds}
  {scoreImages}
  {scorePdf}
  {regions}
  onPlay={handlePlay}
  onPause={handlePause}
  onEnded={handleEnded}
  {bindAudio}
/>
