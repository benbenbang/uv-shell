Register-ArgumentCompleter -Native -CommandName uv-shell -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commands = @(
        [CompletionResult]::new('anchor', 'anchor', [CompletionResultType]::ParameterValue, 'Print export commands for shell rc eval')
        [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Generate shell completions')
    )

    $shells = @(
        [CompletionResult]::new('bash', 'bash', [CompletionResultType]::ParameterValue, 'Bash completions')
        [CompletionResult]::new('zsh', 'zsh', [CompletionResultType]::ParameterValue, 'Zsh completions')
        [CompletionResult]::new('fish', 'fish', [CompletionResultType]::ParameterValue, 'Fish completions')
        [CompletionResult]::new('nushell', 'nushell', [CompletionResultType]::ParameterValue, 'Nushell completions')
        [CompletionResult]::new('powershell', 'powershell', [CompletionResultType]::ParameterValue, 'PowerShell completions')
    )

    $anchorShells = @(
        [CompletionResult]::new('bash', 'bash', [CompletionResultType]::ParameterValue, 'Bash/Zsh export syntax')
        [CompletionResult]::new('fish', 'fish', [CompletionResultType]::ParameterValue, 'Fish set -gx syntax')
        [CompletionResult]::new('nushell', 'nushell', [CompletionResultType]::ParameterValue, 'Nushell $env syntax')
        [CompletionResult]::new('powershell', 'powershell', [CompletionResultType]::ParameterValue, 'PowerShell $env: syntax')
        [CompletionResult]::new('cmd', 'cmd', [CompletionResultType]::ParameterValue, 'CMD set syntax')
    )

    $options = @(
        [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Python interpreter to use')
        [CompletionResult]::new('--python', '--python', [CompletionResultType]::ParameterName, 'Python interpreter to use')
        [CompletionResult]::new('--seed', '--seed', [CompletionResultType]::ParameterName, 'Install seed packages (pip, setuptools, wheel)')
        [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'Re-create the venv even if .venv already exists')
        [CompletionResult]::new('--clear', '--clear', [CompletionResultType]::ParameterName, 'Re-create the venv even if .venv already exists')
        [CompletionResult]::new('--allow-existing', '--allow-existing', [CompletionResultType]::ParameterName, 'Preserve existing files at the target path')
        [CompletionResult]::new('--prompt', '--prompt', [CompletionResultType]::ParameterName, 'Custom prompt prefix (skips auto-prompt)')
        [CompletionResult]::new('--system-site-packages', '--system-site-packages', [CompletionResultType]::ParameterName, 'Access system site packages')
        [CompletionResult]::new('--relocatable', '--relocatable', [CompletionResultType]::ParameterName, 'Make the venv relocatable')
        [CompletionResult]::new('--no-project', '--no-project', [CompletionResultType]::ParameterName, 'Avoid discovering a project or workspace')
        [CompletionResult]::new('--no-config', '--no-config', [CompletionResultType]::ParameterName, 'Avoid discovering configuration files')
        [CompletionResult]::new('--index-strategy', '--index-strategy', [CompletionResultType]::ParameterName, 'Strategy for resolving against multiple indexes')
        [CompletionResult]::new('--keyring-provider', '--keyring-provider', [CompletionResultType]::ParameterName, 'Keyring provider for authentication')
        [CompletionResult]::new('--exclude-newer', '--exclude-newer', [CompletionResultType]::ParameterName, 'Limit packages to those uploaded before date')
        [CompletionResult]::new('--link-mode', '--link-mode', [CompletionResultType]::ParameterName, 'Method for installing from cache')
        [CompletionResult]::new('--no-index', '--no-index', [CompletionResultType]::ParameterName, 'Ignore the registry index')
        [CompletionResult]::new('--refresh', '--refresh', [CompletionResultType]::ParameterName, 'Refresh all cached data')
        [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Avoid reading from or writing to the cache')
        [CompletionResult]::new('--no-cache', '--no-cache', [CompletionResultType]::ParameterName, 'Avoid reading from or writing to the cache')
        [CompletionResult]::new('--cache-dir', '--cache-dir', [CompletionResultType]::ParameterName, 'Path to the cache directory')
        [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Quiet output')
        [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Quiet output')
        [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Verbose output')
        [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Verbose output')
        [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'Control use of color')
        [CompletionResult]::new('--native-tls', '--native-tls', [CompletionResultType]::ParameterName, 'Load TLS certificates from platform native store')
        [CompletionResult]::new('--offline', '--offline', [CompletionResultType]::ParameterName, 'Disable network access')
        [CompletionResult]::new('--no-progress', '--no-progress', [CompletionResultType]::ParameterName, 'Hide all progress outputs')
        [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Show help message')
        [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Show help message')
    )

    $elements = $commandAst.CommandElements
    $count = $elements.Count

    # Determine context
    if ($count -le 2) {
        # First argument: offer commands and options
        $all = $commands + $options
        $all | Where-Object { $_.CompletionText -like "$wordToComplete*" }
    }
    elseif ($elements[1].ToString() -eq 'completions') {
        $shells | Where-Object { $_.CompletionText -like "$wordToComplete*" }
    }
    elseif ($elements[1].ToString() -eq 'anchor') {
        if ($elements[-1].ToString() -eq '--shell' -or $elements[-2].ToString() -eq '--shell') {
            $anchorShells | Where-Object { $_.CompletionText -like "$wordToComplete*" }
        }
        else {
            [CompletionResult]::new('--shell', '--shell', [CompletionResultType]::ParameterName, 'Override shell detection')
        }
    }
    else {
        $options | Where-Object { $_.CompletionText -like "$wordToComplete*" }
    }
}
