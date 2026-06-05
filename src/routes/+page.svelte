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
      alert("Error: " + e);
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
            alert("Exported to " + path);
        }
    } catch (e) {
        console.error(e);
        alert("Export failed: " + e);
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

<div class="container mx-auto p-4 space-y-6">
  <div class="flex justify-between items-center border-b pb-4">
    <h1 class="text-2xl font-bold">Code Inspector</h1>
    <Button onclick={checkFolder} disabled={isLoading}>
      {isLoading ? "Checking..." : "Check"}
    </Button>
  </div>

  <div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
    <div class="space-y-4 lg:col-span-2">
        <Card.Root>
          <Card.Header><Card.Title>Folder</Card.Title></Card.Header>
          <Card.Content class="space-y-4">
            <div class="flex space-x-2">
              <Input bind:value={folderPath} placeholder="C:\Project" />
              <Button variant="secondary" onclick={browseFolder}>Browse</Button>
            </div>
          </Card.Content>
        </Card.Root>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <Card.Root>
              <Card.Header><Card.Title>Expected Encoding</Card.Title></Card.Header>
              <Card.Content>
                <Select.Root type="single" bind:value={expectedEncoding}>
                  <Select.Trigger class="w-full">{expectedEncoding}</Select.Trigger>
                  <Select.Content>
                    <Select.Item value="UTF-8">UTF-8</Select.Item>
                    <Select.Item value="Shift_JIS">Shift_JIS</Select.Item>
                    <Select.Item value="EUC-JP">EUC-JP</Select.Item>
                  </Select.Content>
                </Select.Root>
              </Card.Content>
            </Card.Root>

            <Card.Root>
              <Card.Header><Card.Title>Expected Newline</Card.Title></Card.Header>
              <Card.Content>
                <Select.Root type="single" bind:value={expectedNewline}>
                  <Select.Trigger class="w-full">{expectedNewline}</Select.Trigger>
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
      <Card.Header><Card.Title>Exclusions</Card.Title></Card.Header>
      <Card.Content class="space-y-4">
        <div class="space-y-2">
            <Label>Exclude Folders (comma separated)</Label>
            <Input bind:value={excludeDirs} />
        </div>
        <div class="space-y-2">
            <Label>Exclude Extensions (comma separated)</Label>
            <Input bind:value={excludeExts} />
        </div>
      </Card.Content>
    </Card.Root>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-4 gap-4 text-center">
    <Card.Root><Card.Content class="pt-6"><div class="text-2xl font-bold">{stats.total}</div><div class="text-sm text-muted-foreground">Total</div></Card.Content></Card.Root>
    <Card.Root><Card.Content class="pt-6"><div class="text-2xl font-bold text-green-600">{stats.ok}</div><div class="text-sm text-muted-foreground">OK</div></Card.Content></Card.Root>
    <Card.Root><Card.Content class="pt-6"><div class="text-2xl font-bold text-destructive">{stats.ng}</div><div class="text-sm text-muted-foreground">NG</div></Card.Content></Card.Root>
    <Card.Root><Card.Content class="pt-6"><div class="text-2xl font-bold text-orange-500">{stats.other}</div><div class="text-sm text-muted-foreground">Unknown / Binary</div></Card.Content></Card.Root>
  </div>

  <Card.Root>
    <Card.Header class="flex flex-col md:flex-row items-start md:items-center justify-between gap-4">
      <Card.Title>Results</Card.Title>
      <div class="flex flex-wrap items-center gap-4">
        <Input placeholder="Search Path..." class="max-w-xs" bind:value={globalFilter} />
        <div class="flex items-center space-x-2">
          <Checkbox id="ng-only" bind:checked={ngOnly} />
          <Label for="ng-only">NG Only</Label>
        </div>
        <div class="flex items-center space-x-2">
          <Checkbox id="hide-binary" bind:checked={hideBinary} />
          <Label for="hide-binary">Hide Binary</Label>
        </div>
        <Button variant="outline" size="sm" onclick={exportCsv} disabled={results.length === 0}>Export CSV</Button>
      </div>
    </Card.Header>
    <Card.Content>
      <div class="rounded-md border overflow-x-auto">
        <table class="w-full text-sm">
          <thead class="bg-muted/50 border-b">
            <tr><th class="p-2 text-left">Status</th><th class="p-2 text-left">Path</th><th class="p-2 text-left">Encoding</th><th class="p-2 text-left">Newline</th><th class="p-2 text-left">Size</th></tr>
          </thead>
          <tbody>
            {#each filteredResults as row}
              <tr class="border-b last:border-0 hover:bg-muted/50 transition-colors">
                <td class="p-2"><Badge variant={row.status === "OK" ? "secondary" : row.status === "NG" ? "destructive" : "outline"}>{row.status}</Badge></td>
                <td class="p-2">{row.path}</td><td class="p-2">{row.encoding}</td><td class="p-2">{row.newline}</td><td class="p-2">{formatSize(row.size)}</td>
              </tr>
            {:else}
              <tr><td colspan="5" class="p-8 text-center text-muted-foreground">No results found.</td></tr>
            {/each}
          </tbody>
        </table>
      </div>
    </Card.Content>
  </Card.Root>
</div>
