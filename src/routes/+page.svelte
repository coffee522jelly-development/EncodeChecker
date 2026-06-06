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
  import { Settings } from "lucide-svelte";

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
</script>

<div class="container mx-auto p-1 space-y-1 text-[10px]">
  <!-- Top Bar: Folder, Config, Actions -->
  <div class="flex items-center space-x-2 border-b pb-1">
    <div class="flex items-center space-x-1 flex-1">
        <Input bind:value={folderPath} placeholder="フォルダ..." class="h-6 text-[10px] py-0" />
        <Button variant="secondary" size="sm" class="h-6 px-1.5 text-[10px]" onclick={browseFolder}>参照</Button>
    </div>

    <div class="flex items-center space-x-2 border-l pl-2">
        <div class="flex items-center space-x-1">
            <span class="whitespace-nowrap opacity-70">文字コード:</span>
            <Select.Root type="single" bind:value={expectedEncoding}>
                <Select.Trigger class="h-5 text-[10px] py-0 min-w-[70px]">{expectedEncoding}</Select.Trigger>
                <Select.Content>
                    <Select.Item value="UTF-8" class="text-[10px]">UTF-8</Select.Item>
                    <Select.Item value="Shift_JIS" class="text-[10px]">Shift_JIS</Select.Item>
                    <Select.Item value="EUC-JP" class="text-[10px]">EUC-JP</Select.Item>
                </Select.Content>
            </Select.Root>
        </div>
        <div class="flex items-center space-x-1">
            <span class="whitespace-nowrap opacity-70">改行:</span>
            <Select.Root type="single" bind:value={expectedNewline}>
                <Select.Trigger class="h-5 text-[10px] py-0 min-w-[60px]">{expectedNewline}</Select.Trigger>
                <Select.Content>
                    <Select.Item value="LF" class="text-[10px]">LF</Select.Item>
                    <Select.Item value="CRLF" class="text-[10px]">CRLF</Select.Item>
                </Select.Content>
            </Select.Root>
        </div>
    </div>

    <div class="flex items-center space-x-1 border-l pl-2">
      <Button variant="ghost" size="icon" class="h-6 w-6" onclick={() => showSettings = true}>
        <Settings class="h-3.5 w-3.5" />
      </Button>
      <Button onclick={checkFolder} disabled={isLoading} size="sm" class="h-6 px-3 font-bold text-[10px]">
        {isLoading ? "実行中..." : "実行"}
      </Button>
    </div>
  </div>

  <!-- Summary Row (Compact) -->
  <div class="flex space-x-2 py-0.5 px-1 border-b items-center justify-between">
    <div class="flex space-x-4">
        <div class="flex items-center space-x-1">
            <span class="text-muted-foreground text-[8px]">合計:</span>
            <span class="font-bold">{stats.total}</span>
        </div>
        <div class="flex items-center space-x-1">
            <span class="text-green-700/60 text-[8px]">OK:</span>
            <span class="font-bold text-green-600">{stats.ok}</span>
        </div>
        <div class="flex items-center space-x-1">
            <span class="text-destructive/60 text-[8px]">NG:</span>
            <span class="font-bold text-destructive">{stats.ng}</span>
        </div>
        <div class="flex items-center space-x-1">
            <span class="text-orange-700/60 text-[8px]">その他:</span>
            <span class="font-bold text-orange-500">{stats.other}</span>
        </div>
    </div>

    <div class="flex items-center space-x-3">
        <Input placeholder="検索..." class="w-32 h-5 text-[10px] py-0" bind:value={globalFilter} />
        <div class="flex items-center space-x-1">
          <Checkbox id="ng-only" bind:checked={ngOnly} class="h-2.5 w-2.5" />
          <Label for="ng-only" class="text-[9px]">NGのみ</Label>
        </div>
        <div class="flex items-center space-x-1">
          <Checkbox id="hide-binary" bind:checked={hideBinary} class="h-2.5 w-2.5" />
          <Label for="hide-binary" class="text-[9px]">バイナリ隠す</Label>
        </div>
        <Button variant="outline" size="sm" class="h-5 text-[9px] px-1.5" onclick={exportCsv} disabled={results.length === 0}>CSV</Button>
    </div>
  </div>

  <!-- Results Table -->
  <div class="rounded-md border overflow-x-auto max-h-[calc(100vh-100px)] overflow-y-auto">
    <table class="w-full text-[9px]">
        <thead class="bg-muted/50 border-b sticky top-0 z-10">
        <tr>
            <th class="p-1 text-left font-medium w-14">状態</th>
            <th class="p-1 text-left font-medium">パス</th>
            <th class="p-1 text-left font-medium w-20">文字コード</th>
            <th class="p-1 text-left font-medium w-14">改行</th>
            <th class="p-1 text-right font-medium w-16">サイズ</th>
        </tr>
        </thead>
        <tbody>
        {#each filteredResults as row}
            <tr class="border-b last:border-0 hover:bg-muted/50 transition-colors">
            <td class="p-0.5">
                <Badge variant={row.status === "OK" ? "secondary" : row.status === "NG" ? "destructive" : "outline"} class="text-[8px] px-1 py-0 h-3.5 leading-none font-normal">
                {row.status}
                </Badge>
            </td>
            <td class="p-0.5 font-mono text-[9px] truncate max-w-[500px]" title={row.path}>{row.path}</td>
            <td class="p-0.5">{row.encoding}</td>
            <td class="p-0.5">{row.newline}</td>
            <td class="p-0.5 text-right text-muted-foreground">{formatSize(row.size)}</td>
            </tr>
        {:else}
            <tr><td colspan="5" class="p-2 text-center text-muted-foreground">結果なし</td></tr>
        {/each}
        </tbody>
    </table>
  </div>

  <!-- Settings Overlay -->
  {#if showSettings}
    <div class="fixed inset-0 z-50 bg-background/60 backdrop-blur-[1px] flex items-center justify-center p-4">
        <Card.Root class="w-full max-w-sm shadow-xl border">
            <Card.Header class="py-2 px-3">
                <Card.Title class="text-xs">設定 (除外項目)</Card.Title>
            </Card.Header>
            <Card.Content class="space-y-3 py-2 px-3">
                <div class="space-y-1">
                    <Label class="text-[10px] font-medium">除外フォルダ</Label>
                    <Input bind:value={excludeDirs} class="h-7 text-[10px]" />
                    <p class="text-[9px] text-muted-foreground">カンマ区切り (.git, node_modules)</p>
                </div>
                <div class="space-y-1">
                    <Label class="text-[10px] font-medium">除外拡張子</Label>
                    <Input bind:value={excludeExts} class="h-7 text-[10px]" />
                    <p class="text-[9px] text-muted-foreground">カンマ区切り (.exe, .xlsx)</p>
                </div>
            </Card.Content>
            <Card.Footer class="flex justify-end py-2 px-3 border-t bg-muted/10">
                <Button size="sm" class="h-7 text-[10px]" onclick={() => showSettings = false}>閉じる</Button>
            </Card.Footer>
        </Card.Root>
    </div>
  {/if}
</div>

<style>
    :global(body) {
        overflow: hidden;
    }
</style>
