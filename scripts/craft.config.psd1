# craft.config.psd1 - per-repo settings for start.ps1 and scripts\release.ps1.
# Standard: mcp-central-docs\standards\rust-artcraft\CRAFT_START_AND_RELEASE.md
@{
    Name        = 'proofcraft'
    Display     = 'ProofCraft'
    AppPackage  = 'proofcraft-app'
    AppBin      = 'proofcraft-app'
    AppSmoke    = @('--smoke-test')
    CliPackage  = 'proofcraft-mcp'
    CliBin      = 'proofcraft-mcp'
    CliSmoke    = @('--db', '{db}', 'status')
    McpPackage  = 'proofcraft-mcp'
    McpBin      = 'proofcraft-mcp'
    McpArgs     = @('mcp')
    McpSmoke    = @('mcp')
    Installer   = $true
    Mcpb        = $false
}
