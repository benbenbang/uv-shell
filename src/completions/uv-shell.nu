module uv-shell-completions {
    def "nu-complete uv-shell commands" [] {
        [
            { value: "anchor", description: "Print export commands for shell rc eval" }
            { value: "completions", description: "Generate shell completions" }
        ]
    }

    def "nu-complete uv-shell shells" [] {
        ["bash" "zsh" "fish" "nushell" "powershell"]
    }

    def "nu-complete uv-shell anchor-shells" [] {
        ["bash" "fish" "nushell" "powershell" "cmd"]
    }

    def "nu-complete uv-shell index-strategy" [] {
        ["first-index" "unsafe-first-match" "unsafe-best-match"]
    }

    def "nu-complete uv-shell keyring-provider" [] {
        ["disabled" "subprocess"]
    }

    def "nu-complete uv-shell link-mode" [] {
        ["clone" "copy" "hardlink" "symlink"]
    }

    def "nu-complete uv-shell color" [] {
        ["auto" "always" "never"]
    }

    export extern "uv-shell" [
        command?: string@"nu-complete uv-shell commands"
        --python(-p): string          # Python interpreter to use
        --seed                        # Install seed packages (pip, setuptools, wheel)
        --clear(-c)                   # Re-create the venv even if .venv already exists
        --allow-existing              # Preserve existing files at the target path
        --prompt: string              # Custom prompt prefix (skips auto-prompt)
        --system-site-packages        # Access system site packages
        --relocatable                 # Make the venv relocatable
        --no-project                  # Avoid discovering a project or workspace
        --index-strategy: string@"nu-complete uv-shell index-strategy"  # Strategy for resolving against multiple indexes
        --keyring-provider: string@"nu-complete uv-shell keyring-provider"  # Keyring provider for authentication
        --exclude-newer: string       # Limit packages to those uploaded before date
        --link-mode: string@"nu-complete uv-shell link-mode"  # Method for installing from cache
        --index: string               # Additional index URLs
        --default-index: string       # Default package index URL
        --find-links(-f): string      # Locations to search for distributions
        --no-index                    # Ignore the registry index
        --refresh                     # Refresh all cached data
        --no-cache(-n)                # Avoid reading from or writing to the cache
        --cache-dir: path             # Path to the cache directory
        --quiet(-q)                   # Quiet output
        --verbose(-v)                 # Verbose output
        --color: string@"nu-complete uv-shell color"  # Control use of color
        --native-tls                  # Load TLS certificates from platform native store
        --offline                     # Disable network access
        --allow-insecure-host: string # Allow insecure connections to a host
        --no-progress                 # Hide all progress outputs
        --directory: path             # Change to given directory before running
        --project: path               # Discover a project in given directory
        --config-file: path           # Path to uv.toml
        --no-config                   # Avoid discovering configuration files
        --help(-h)                    # Show help message
    ]

    export extern "uv-shell anchor" [
        --shell: string@"nu-complete uv-shell anchor-shells"  # Override shell detection
    ]

    export extern "uv-shell completions" [
        shell?: string@"nu-complete uv-shell shells"
    ]
}

use uv-shell-completions *
