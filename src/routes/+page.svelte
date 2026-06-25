<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import * as Card from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { invoke } from "@tauri-apps/api/core";
  import { writeTextFile } from "@tauri-apps/plugin-fs";
  import { convertToCsv } from "$lib/utils";
  import { Settings, FolderOpen, Play, Download, Search, X } from "lucide-svelte";

  interface FileResult {
    path: string;
    status: "OK" | "NG" | "UNKNOWN" | "BINARY";
    encoding: string;
    newline: string;
    size: number;
  }

  let folderPath = $state("");
  let expectedEncoding = $state("UTF-8");
  let expectedNewline = $state("LF");
  let excludeDirs = $state(".git, node_modules, dist, bin, obj");
  let excludeExts = $state(".exe, .dll, .zip, .png, .jpg, .xlsx, .pdf");

  let results = $state<FileResult[]>([]);
  let isLoading = $state(false);
  let globalFilter = $state("");
  let ngOnly = $state(false);
  let hideBinary = $state(false);
  let showSettings = $state(false);

  const filteredResults = $derived(
    results.filter(r => {
        const matchesFilter = r.path.toLowerCase().includes(globalFilter.toLowerCase());
        const matchesNg = ngOnly ? r.status === "NG" : true;
        const matchesBinary = hideBinary ? r.status !== "BINARY" : true;
        return matchesFilter && matchesNg && matchesBinary;
    })
  );

  async function browseFolder() {
    const selected = await open({
      directory: true,
      multiple: false,
    });
    if (selected && typeof selected === "string") {
      folderPath = selected;
    }
  }

  async function checkFolder() {
    if (!folderPath) return;
    isLoading = true;
    try {
      const dirs = excludeDirs.split(",").map(s => s.trim()).filter(s => s);
      const exts = excludeExts.split(",").map(s => s.trim()).filter(s => s);

      results = await invoke("check_folder", {
        folder: folderPath,
        expectedEncoding,
        expectedNewline,
        excludeDirs: dirs,
        excludeExts: exts,
      });
    } catch (e) {
      console.error(e);
      alert("エラー: " + e);
    } finally {
      isLoading = false;
    }
  }

  async function exportCsv() {
    if (results.length === 0) return;
    try {
        const csv = convertToCsv(filteredResults);
        const path = await save({
            filters: [{ name: "CSV", extensions: ["csv"] }],
            defaultPath: "results.csv"
        });
        if (path) {
            await writeTextFile(path, csv);
            alert("CSVを出力しました: " + path);
        }
    } catch (e) {
        console.error(e);
        alert("出力に失敗しました: " + e);
    }
  }

  const stats = $derived({
    total: results.length,
    ok: results.filter((r) => r.status === "OK").length,
    ng: results.filter((r) => r.status === "NG").length,
    other: results.filter((r) => r.status !== "OK" && r.status !== "NG").length,
  });

  function formatSize(bytes: number) {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  }

  const statusColors = {
    OK: "bg-emerald-500",
    NG: "bg-rose-500",
    UNKNOWN: "bg-slate-400",
    BINARY: "bg-amber-400"
  };
</script>

<div class="h-screen flex flex-col bg-background text-[10px] overflow-hidden selection:bg-primary/20">
  <!-- Ultra-Compact Toolbar -->
  <header class="flex items-center h-7 px-1 border-b bg-muted/30 gap-1 shrink-0">
    <div class="flex items-center gap-0.5 flex-1 min-w-0">
        <Button variant="ghost" size="icon" class="h-6 w-6 shrink-0" onclick={browseFolder} title="フォルダ選択">
            <FolderOpen class="h-3.5 w-3.5" />
        </Button>
        <div class="flex-1 min-w-0 relative group">
            <input
                bind:value={folderPath}
                placeholder="判定するフォルダのパス..."
                class="w-full h-5.5 bg-transparent border-none focus:ring-0 text-[10px] px-1 truncate"
            />
            <div class="absolute bottom-0 left-1 right-1 h-px bg-primary/20 group-focus-within:bg-primary transition-colors"></div>
        </div>
    </div>

    <div class="flex items-center h-5 gap-1.5 px-2 border-l border-r border-muted-foreground/20">
        <div class="flex items-center gap-1">
            <span class="text-muted-foreground whitespace-nowrap">ENC:</span>
            <Select.Root type="single" bind:value={expectedEncoding}>
                <Select.Trigger class="h-5 text-[9px] py-0 min-w-[65px] border-none bg-transparent hover:bg-muted/50 transition-colors px-1">{expectedEncoding}</Select.Trigger>
                <Select.Content>
                    <Select.Item value="UTF-8" class="text-[10px]">UTF-8</Select.Item>
                    <Select.Item value="Shift_JIS" class="text-[10px]">Shift_JIS</Select.Item>
                    <Select.Item value="EUC-JP" class="text-[10px]">EUC-JP</Select.Item>
                </Select.Content>
            </Select.Root>
        </div>
        <div class="flex items-center gap-1">
            <span class="text-muted-foreground whitespace-nowrap">NL:</span>
            <Select.Root type="single" bind:value={expectedNewline}>
                <Select.Trigger class="h-5 text-[9px] py-0 min-w-[50px] border-none bg-transparent hover:bg-muted/50 transition-colors px-1">{expectedNewline}</Select.Trigger>
                <Select.Content>
                    <Select.Item value="LF" class="text-[10px]">LF</Select.Item>
                    <Select.Item value="CRLF" class="text-[10px]">CRLF</Select.Item>
                </Select.Content>
            </Select.Root>
        </div>
    </div>

    <div class="flex items-center gap-0.5 shrink-0">
      <Button variant="ghost" size="icon" class="h-6 w-6" onclick={() => showSettings = true} title="設定">
        <Settings class="h-3.5 w-3.5" />
      </Button>
      <Button variant="default" size="sm" class="h-5.5 px-2 font-bold text-[10px] gap-1" onclick={checkFolder} disabled={isLoading}>
        {#if isLoading}
            <div class="h-2.5 w-2.5 animate-spin border-2 border-white border-t-transparent rounded-full"></div>
        {:else}
            <Play class="h-2.5 w-2.5 fill-current" />
        {/if}
        <span>実行</span>
      </Button>
    </div>
  </header>

  <!-- Filter & Stats Bar -->
  <div class="flex items-center h-6 px-2 bg-muted/10 border-b shrink-0 gap-3">
    <div class="flex items-center gap-2 border-r pr-3">
        <div class="flex items-center gap-1"><span class="text-[8px] opacity-60">合計:</span><span class="font-bold">{stats.total}</span></div>
        <div class="flex items-center gap-1"><span class="text-emerald-600/80 text-[8px]">OK:</span><span class="font-bold text-emerald-600">{stats.ok}</span></div>
        <div class="flex items-center gap-1"><span class="text-rose-600/80 text-[8px]">NG:</span><span class="font-bold text-rose-600">{stats.ng}</span></div>
    </div>

    <div class="flex items-center flex-1 gap-2">
        <div class="flex items-center h-4.5 bg-background border rounded px-1 flex-1 max-w-[200px]">
            <Search class="h-2.5 w-2.5 text-muted-foreground mr-1" />
            <input bind:value={globalFilter} placeholder="フィルタ..." class="bg-transparent border-none focus:ring-0 text-[9px] w-full p-0" />
        </div>
        <div class="flex items-center gap-2">
            <label class="flex items-center gap-1 cursor-pointer hover:opacity-70 transition-opacity">
                <input type="checkbox" bind:checked={ngOnly} class="h-2.5 w-2.5 rounded border-muted-foreground/30 text-rose-500 focus:ring-rose-500/20" />
                <span class="text-[9px]">NGのみ</span>
            </label>
            <label class="flex items-center gap-1 cursor-pointer hover:opacity-70 transition-opacity">
                <input type="checkbox" bind:checked={hideBinary} class="h-2.5 w-2.5 rounded border-muted-foreground/30 text-amber-500 focus:ring-amber-500/20" />
                <span class="text-[9px]">バイナリ隠す</span>
            </label>
        </div>
    </div>

    <Button variant="outline" size="sm" class="h-4.5 text-[8px] px-1.5 font-normal gap-1" onclick={exportCsv} disabled={results.length === 0}>
        <Download class="h-2.5 w-2.5" />
        <span>CSV</span>
    </Button>
  </div>

  <!-- Data Grid -->
  <main class="flex-1 overflow-auto relative">
    <table class="w-full text-left border-separate border-spacing-0">
        <thead class="bg-muted/50 border-b sticky top-0 z-10 backdrop-blur-sm">
        <tr class="h-6">
            <th class="pl-3 pr-2 font-semibold border-b text-muted-foreground/80 w-12">状態</th>
            <th class="px-2 font-semibold border-b text-muted-foreground/80">パス</th>
            <th class="px-2 font-semibold border-b text-muted-foreground/80 w-24">エンコード</th>
            <th class="px-2 font-semibold border-b text-muted-foreground/80 w-16">改行</th>
            <th class="px-2 pr-3 font-semibold border-b text-muted-foreground/80 text-right w-20">サイズ</th>
        </tr>
        </thead>
        <tbody class="divide-y divide-muted/30">
        {#each filteredResults as row}
            <tr class="h-5 hover:bg-muted/20 transition-colors group">
            <td class="pl-3 pr-2 relative">
                <div class="absolute left-0 top-0.5 bottom-0.5 w-1 {statusColors[row.status] || 'bg-slate-300'} rounded-r-full shadow-sm"></div>
                <span class="font-bold text-[8px] px-1 rounded-sm uppercase tracking-tighter {row.status === 'NG' ? 'text-rose-600' : 'text-muted-foreground'}">
                    {row.status}
                </span>
            </td>
            <td class="px-2 font-mono text-[9px] text-foreground/80 truncate max-w-0" title={row.path}>{row.path}</td>
            <td class="px-2 text-foreground/70">{row.encoding}</td>
            <td class="px-2 text-foreground/70">{row.newline}</td>
            <td class="px-2 pr-3 text-right text-muted-foreground font-mono tabular-nums">{formatSize(row.size)}</td>
            </tr>
        {:else}
            <tr><td colspan="5" class="py-12 text-center text-muted-foreground/50 italic">データがありません。スキャンを実行してください。</td></tr>
        {/each}
        </tbody>
    </table>
  </main>

  <!-- Settings Overlay -->
  {#if showSettings}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="fixed inset-0 z-50 bg-background/40 backdrop-blur-[2px] flex items-center justify-center p-4" onclick={() => showSettings = false}>
        <div class="w-full max-w-xs bg-card shadow-2xl border rounded-lg overflow-hidden animate-in zoom-in-95 duration-150" onclick={(e) => e.stopPropagation()}>
            <div class="flex items-center justify-between px-3 py-2 border-b bg-muted/20">
                <h2 class="text-xs font-bold">環境設定</h2>
                <Button variant="ghost" size="icon" class="h-5 w-5" onclick={() => showSettings = false}>
                    <X class="h-3 w-3" />
                </Button>
            </div>
            <div class="p-3 space-y-3">
                <div class="space-y-1">
                    <Label class="text-[9px] text-muted-foreground uppercase font-bold tracking-wider">除外ディレクトリ</Label>
                    <textarea
                        bind:value={excludeDirs}
                        class="w-full h-12 bg-muted/10 border border-muted-foreground/20 rounded p-1.5 text-[10px] focus:ring-1 focus:ring-primary outline-none transition-all resize-none"
                    ></textarea>
                    <p class="text-[8px] text-muted-foreground/60 italic">※カンマ区切りで入力</p>
                </div>
                <div class="space-y-1">
                    <Label class="text-[9px] text-muted-foreground uppercase font-bold tracking-wider">除外拡張子</Label>
                    <input
                        bind:value={excludeExts}
                        class="w-full h-7 bg-muted/10 border border-muted-foreground/20 rounded px-1.5 text-[10px] focus:ring-1 focus:ring-primary outline-none transition-all"
                    />
                    <p class="text-[8px] text-muted-foreground/60 italic">※例: .exe, .xlsx, .pdf</p>
                </div>
            </div>
            <div class="p-2 border-t flex justify-end bg-muted/5">
                <Button variant="default" size="sm" class="h-6 px-4 text-[10px]" onclick={() => showSettings = false}>適用して閉じる</Button>
            </div>
        </div>
    </div>
  {/if}
</div>

<style>
    :global(body) {
        user-select: none;
    }
    input::placeholder {
        opacity: 0.4;
    }
    /* Custom Scrollbar for a more refined look */
    ::-webkit-scrollbar {
        width: 6px;
        height: 6px;
    }
    ::-webkit-scrollbar-track {
        background: transparent;
    }
    ::-webkit-scrollbar-thumb {
        background: rgba(0,0,0,0.1);
        border-radius: 10px;
    }
    ::-webkit-scrollbar-thumb:hover {
        background: rgba(0,0,0,0.2);
    }
</style>
