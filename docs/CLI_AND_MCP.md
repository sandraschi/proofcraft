# CLI, MCP Server & Launchers Reference

## 1. Launchers
- **Double-click:** `start.bat` (or `mcp-central-docs/starts/proofcraft-start.bat`)
- **PowerShell:**
  - `.\start.ps1` (incremental build + launch desktop app)
  - `.\start.ps1 -Smoke` (headless smoke check on throwaway DB)
  - `.\start.ps1 -Mcp` (launch MCP stdio server)
  - `.\start.ps1 -Release` (run release binary)

---

## 2. CLI (`proofcraft-mcp`)
- `proofcraft-mcp status`
- `proofcraft-mcp list-problems`
- `proofcraft-mcp get-problem --problem-id <id>`
- `proofcraft-mcp create-problem --title "..." --domain "..." --statement "..." --max-calls 200`
- `proofcraft-mcp add-direction --problem-id <id> --name "..." --claim "..."`
- `proofcraft-mcp add-literature --problem-id <id> --arxiv-id 2609.40324`
- `proofcraft-mcp import-url --url https://erdosproblems.com/42`
- `proofcraft-mcp load-pack --pack lean_warmup` (`lean_warmup`, `erdos_combinatorics`, `cogentic_paper`)
- `proofcraft-mcp run-round --problem-id <id> --provers 2`
- `proofcraft-mcp lean-status`
- `proofcraft-mcp check-lean --code "theorem t : 1 + 1 = 2 := rfl"`
- `proofcraft-mcp formalize-lemma --problem-id <id> --ledger-id <lem>`
- `proofcraft-mcp consolidate --problem-id <id> --format tex`
- `proofcraft-mcp export-lake --problem-id <id> --out-dir ./out_lake`

---

## 3. MCP Stdio Server (`2026-07-28`, 15 Tools)

Run `proofcraft-mcp mcp` (pure JSON-RPC 2.0 on `stdout`, diagnostics on `stderr`):
- `proof_status`, `proof_list_problems`, `proof_get_problem`, `proof_create_problem`
- `proof_ledger`, `proof_export_manuscript`, `proof_consolidate`, `proof_export_lake`
- `proof_run_round`, `proof_add_literature`, `proof_import_url`, `proof_load_pack`
- `proof_lean_status`, `proof_check_lean`, `proof_formalize_lemma`
- Resources: `skill://proofcraft/SKILL.md`
