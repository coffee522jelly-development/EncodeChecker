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
  let excludeExts = $state(".exe, .dll, .zip, .png, .jpg");

  let results = $state<FileResult[]>([]);
  let isLoading = $state(false);
  let globalFilter = $state("");
  let ngOnly = $state(false);
  let hideBinary = $state(false);

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

<div class="container mx-auto p-2 space-y-3">
  <div class="flex justify-end items-center border-b pb-2">
    <Button onclick={checkFolder} disabled={isLoading} size="sm" class="px-6 font-bold">
      {isLoading ? "実行中..." : "実行"}
    </Button>
  </div>

  <div class="grid grid-cols-1 lg:grid-cols-3 gap-3">
    <div class="space-y-3 lg:col-span-2">
        <Card.Root>
          <Card.Header class="py-2 px-4"><Card.Title class="text-sm">フォルダ</Card.Title></Card.Header>
          <Card.Content class="py-2 px-4 space-y-2">
            <div class="flex space-x-2">
              <Input bind:value={folderPath} placeholder="C:\Project" class="h-8 text-sm" />
              <Button variant="secondary" size="sm" onclick={browseFolder}>参照</Button>
            </div>
          </Card.Content>
        </Card.Root>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
            <Card.Root>
              <Card.Header class="py-2 px-4"><Card.Title class="text-sm">期待する文字コード</Card.Title></Card.Header>
              <Card.Content class="py-2 px-4">
                <Select.Root type="single" bind:value={expectedEncoding}>
                  <Select.Trigger class="w-full h-8 text-sm">{expectedEncoding}</Select.Trigger>
                  <Select.Content>
                    <Select.Item value="UTF-8">UTF-8</Select.Item>
                    <Select.Item value="Shift_JIS">Shift_JIS</Select.Item>
                    <Select.Item value="EUC-JP">EUC-JP</Select.Item>
                  </Select.Content>
                </Select.Root>
              </Card.Content>
            </Card.Root>

            <Card.Root>
              <Card.Header class="py-2 px-4"><Card.Title class="text-sm">期待する改行コード</Card.Title></Card.Header>
              <Card.Content class="py-2 px-4">
                <Select.Root type="single" bind:value={expectedNewline}>
                  <Select.Trigger class="w-full h-8 text-sm">{expectedNewline}</Select.Trigger>
                  <Select.Content>
                    <Select.Item value="LF">LF</Select.Item>
                    <Select.Item value="CRLF">CRLF</Select.Item>
                  </Select.Content>
                </Select.Root>
              </Card.Content>
            </Card.Root>
        </div>
    </div>

    <Card.Root>
      <Card.Header class="py-2 px-4"><Card.Title class="text-sm">除外設定</Card.Title></Card.Header>
      <Card.Content class="py-2 px-4 space-y-2">
        <div class="space-y-1">
            <Label class="text-xs text-muted-foreground">除外フォルダ</Label>
            <Input bind:value={excludeDirs} class="h-8 text-sm" />
        </div>
        <div class="space-y-1">
            <Label class="text-xs text-muted-foreground">除外拡張子</Label>
            <Input bind:value={excludeExts} class="h-8 text-sm" />
        </div>
      </Card.Content>
    </Card.Root>
  </div>

  <div class="grid grid-cols-2 md:grid-cols-4 gap-2 text-center">
    <Card.Root><Card.Content class="p-2"><div class="text-lg font-bold">{stats.total}</div><div class="text-[10px] text-muted-foreground uppercase">合計</div></Card.Content></Card.Root>
    <Card.Root><Card.Content class="p-2"><div class="text-lg font-bold text-green-600">{stats.ok}</div><div class="text-[10px] text-muted-foreground uppercase">OK</div></Card.Content></Card.Root>
    <Card.Root><Card.Content class="p-2"><div class="text-lg font-bold text-destructive">{stats.ng}</div><div class="text-[10px] text-muted-foreground uppercase">NG</div></Card.Content></Card.Root>
    <Card.Root><Card.Content class="p-2"><div class="text-lg font-bold text-orange-500">{stats.other}</div><div class="text-[10px] text-muted-foreground uppercase">その他</div></Card.Content></Card.Root>
  </div>

  <Card.Root>
    <Card.Header class="py-2 px-4 flex flex-col md:flex-row items-start md:items-center justify-between gap-2">
      <Card.Title class="text-sm">実行結果</Card.Title>
      <div class="flex flex-wrap items-center gap-3">
        <Input placeholder="パスで検索..." class="max-w-xs h-8 text-sm" bind:value={globalFilter} />
        <div class="flex items-center space-x-1">
          <Checkbox id="ng-only" bind:checked={ngOnly} />
          <Label for="ng-only" class="text-xs">NGのみ</Label>
        </div>
        <div class="flex items-center space-x-1">
          <Checkbox id="hide-binary" bind:checked={hideBinary} />
          <Label for="hide-binary" class="text-xs">バイナリ非表示</Label>
        </div>
        <Button variant="outline" size="sm" class="h-8 text-xs" onclick={exportCsv} disabled={results.length === 0}>CSV出力</Button>
      </div>
    </Card.Header>
    <Card.Content class="p-0">
      <div class="rounded-md border-t overflow-x-auto max-h-[400px] overflow-y-auto">
        <table class="w-full text-xs">
          <thead class="bg-muted/50 border-b sticky top-0">
            <tr><th class="p-2 text-left font-medium">状態</th><th class="p-2 text-left font-medium">パス</th><th class="p-2 text-left font-medium">文字コード</th><th class="p-2 text-left font-medium">改行</th><th class="p-2 text-left font-medium">サイズ</th></tr>
          </thead>
          <tbody>
            {#each filteredResults as row}
              <tr class="border-b last:border-0 hover:bg-muted/50 transition-colors">
                <td class="p-2"><Badge variant={row.status === "OK" ? "secondary" : row.status === "NG" ? "destructive" : "outline"} class="text-[10px] px-1 py-0">{row.status}</Badge></td>
                <td class="p-2 font-mono text-[10px] truncate max-w-[300px]" title={row.path}>{row.path}</td><td class="p-2">{row.encoding}</td><td class="p-2">{row.newline}</td><td class="p-2 text-muted-foreground">{formatSize(row.size)}</td>
              </tr>
            {:else}
              <tr><td colspan="5" class="p-8 text-center text-muted-foreground">結果が見つかりません。</td></tr>
            {/each}
          </tbody>
        </table>
      </div>
    </Card.Content>
  </Card.Root>
</div>
